use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tracing::{debug, error, info, warn};

use crate::config::BackendConfig;
use crate::kernel::KernelManager;
use crate::port_forward::PortForwarder;
use crate::protocol::{
    self, ContainerSpec as VmContainerSpec, HostToVm, LogStream as VmLogStream, QCKER_VSOCK_PORT,
    VmToHost,
};
use crate::rootfs::RootfsManager;
use crate::types::*;
use crate::vmm::{self, VmmConfig, VmmManager};
use crate::vsock::SyncVsockChannel;
use crate::RuntimeBackend;

/// MicroVM backend using QEMU as the VMM.
///
/// Architecture:
/// ┌─────────────────────────────────────────┐
/// │  Host (qcker-cli)                       │
/// │  ┌─────────────────────────────────┐    │
/// │  │  MicroVmBackend                 │    │
/// │  │  - Manages VM lifecycle         │    │
/// │  │  - Sends commands via vsock     │    │
/// │  │  - Tracks container state       │    │
/// │  └──────────┬──────────────────────┘    │
/// │             │ vsock (CID:port)           │
/// └─────────────┼───────────────────────────┘
///               │
/// ┌─────────────┼───────────────────────────┐
/// │  Guest VM   │                           │
/// │  ┌──────────▼──────────────────────┐    │
/// │  │  qcker-guest-agent              │    │
/// │  │  - Listens on vsock port 7421   │    │
/// │  │  - Creates/runs containers      │    │
/// │  │  - Reports stats via vsock      │    │
/// │  └─────────────────────────────────┘    │
/// └─────────────────────────────────────────┘
pub struct MicroVmBackend {
    state: Arc<Mutex<MicroVmState>>,
}

struct MicroVmState {
    config: Option<BackendConfig>,
    vmm: Option<VmmManager>,
    vsock: Option<SyncVsockChannel>,
    port_forwarder: PortForwarder,
    status: BackendStatus,
    containers: HashMap<String, ContainerInfo>,
    vsock_cid: u32,
    data_dir: PathBuf,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
enum BackendStatus {
    NotStarted,
    Starting,
    Running,
    ShuttingDown,
}

impl MicroVmBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MicroVmState {
                config: None,
                vmm: None,
                vsock: None,
                port_forwarder: PortForwarder::new(),
                status: BackendStatus::NotStarted,
                containers: HashMap::new(),
                vsock_cid: 0,
                data_dir: PathBuf::new(),
            })),
        }
    }

    /// Allocate a unique context ID for vsock communication.
    /// CIDs 0-2 are reserved; we use 3+ for our VMs.
    #[allow(dead_code)]
    fn allocate_cid(&self) -> u32 {
        use std::time::SystemTime;
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Use modulo to keep CID in reasonable range (3-4294967294)
        3 + ((ts % 4294967291) as u32)
    }

    /// Connect to the VM's guest agent via vsock.
    fn connect_vsock(cid: u32) -> Result<SyncVsockChannel, String> {
        SyncVsockChannel::connect(cid, QCKER_VSOCK_PORT)
    }

    /// Send a command to the guest agent and wait for a response.
    /// The caller must ensure the VM is running before calling this.
    fn send_vm_command(state: &MicroVmState, msg: &HostToVm) -> Result<VmToHost, String> {
        if let Some(ref vsock) = state.vsock {
            vsock.send(msg)?;
            let response: VmToHost = vsock.recv_timeout(Duration::from_secs(30))
                .map_err(|e| format!("vsock recv timeout: {}", e))?;
            Ok(response)
        } else {
            Err("No vsock connection to VM".to_string())
        }
    }

    /// Start the VM and wait for the guest agent to be ready.
    fn start_vm_inner(config: &BackendConfig) -> Result<(VmmManager, SyncVsockChannel, u32), String> {
        let cid = {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            3 + ((ts % 4294967291) as u32)
        };

        let kernel_manager = KernelManager::new(config.data_dir.clone());
        let kernel_path = kernel_manager
            .get_or_download()
            .map_err(|e| format!("Kernel not available: {}", e))?;

        let rootfs_manager = RootfsManager::new(config.data_dir.clone());
        let rootfs_path = rootfs_manager
            .get_or_build()
            .map_err(|e| format!("Rootfs not available: {}", e))?;

        let vmm_config = VmmConfig {
            kernel_path,
            rootfs_path,
            vcpu_count: config.vcpu_count,
            memory_mb: config.memory_mb,
            use_acceleration: config.use_acceleration,
            kernel_cmdline_extra: config.kernel_cmdline_extra.clone(),
            vsock_cid: cid,
            fs_shares: vec![],
        };

        info!("Starting MicroVM with CID={}, vcpu={}, mem={}MB", cid, config.vcpu_count, config.memory_mb);

        let mut vmm = VmmManager::start(vmm_config)
            .map_err(|e| format!("Failed to start VMM: {}", e))?;

        // Wait for the VM to boot and the guest agent to be ready.
        let mut vsock = None;
        let max_retries = 30;
        let retry_delay = Duration::from_millis(200);

        for attempt in 1..=max_retries {
            debug!("vsock connect attempt {}/{}", attempt, max_retries);

            if !vmm.is_running() {
                return Err("VM process exited unexpectedly during startup".to_string());
            }

            match Self::connect_vsock(cid) {
                Ok(channel) => {
                    // Verify the guest agent is ready by sending a ping
                    match channel.send(&HostToVm::Ping) {
                        Ok(()) => match channel.recv_timeout(Duration::from_secs(5)) {
                            Ok(VmToHost::Pong) => {
                                info!("Guest agent is ready (attempt {})", attempt);
                                vsock = Some(channel);
                                break;
                            }
                            Ok(VmToHost::VmReady { version, arch }) => {
                                info!(
                                    "Guest agent ready: version={}, arch={} (attempt {})",
                                    version, arch, attempt
                                );
                                vsock = Some(channel);
                                break;
                            }
                            Ok(other) => {
                                warn!("Unexpected response from guest agent: {:?}", other);
                                vsock = Some(channel);
                                break;
                            }
                            Err(e) => {
                                debug!("vsock recv timeout on attempt {}: {}", attempt, e);
                                drop(channel);
                                std::thread::sleep(retry_delay);
                            }
                        },
                        Err(e) => {
                            debug!("vsock send failed on attempt {}: {}", attempt, e);
                            drop(channel);
                            std::thread::sleep(retry_delay);
                        }
                    }
                }
                Err(e) => {
                    debug!("vsock connect failed on attempt {}: {}", attempt, e);
                    std::thread::sleep(retry_delay);
                }
            }
        }

        let vsock = vsock.ok_or_else(|| {
            let _ = vmm.stop();
            "Timed out waiting for guest agent to be ready".to_string()
        })?;

        Ok((vmm, vsock, cid))
    }

    /// Convert a host-side ContainerSpec to the protocol's ContainerSpec for the VM.
    fn to_vm_spec(spec: &ContainerSpec) -> VmContainerSpec {
        VmContainerSpec {
            layers: spec.layers.clone(),
            command: spec.command.clone(),
            env: spec.env.clone(),
            working_dir: spec.working_dir.clone(),
            user: spec.user.clone(),
            group: spec.group.clone(),
            mounts: spec.mounts.iter().map(|m| protocol::MountSpec {
                source: m.source.clone(),
                target: m.target.clone(),
                mount_type: match m.mount_type {
                    MountType::Bind => protocol::MountType::Bind,
                    MountType::Volume => protocol::MountType::Volume,
                    MountType::Tmpfs => protocol::MountType::Tmpfs,
                },
                read_only: m.read_only,
            }).collect(),
            port_forwards: spec.port_forwards.iter().map(|p| protocol::PortForwardSpec {
                host_port: p.host_port,
                guest_port: p.guest_port,
                protocol: match p.protocol {
                    Protocol::Tcp => protocol::Protocol::Tcp,
                    Protocol::Udp => protocol::Protocol::Udp,
                },
            }).collect(),
            resources: protocol::ResourceSpec {
                cpu_shares: spec.resources.cpu_shares,
                memory_mb: spec.resources.memory_mb,
                pids_limit: spec.resources.pids_limit,
            },
            rootful: spec.rootful,
            seccomp_profile: spec.seccomp_profile.clone(),
            capabilities: spec.capabilities.clone(),
            network_mode: match &spec.network_mode {
                NetworkMode::Bridge => protocol::NetworkMode::Bridge,
                NetworkMode::Host => protocol::NetworkMode::Host,
                NetworkMode::None => protocol::NetworkMode::None,
                NetworkMode::Custom(name) => protocol::NetworkMode::Custom(name.clone()),
            },
            hostname: spec.hostname.clone(),
            dns: spec.dns.clone(),
            labels: spec.labels.clone(),
        }
    }

    /// Persist container state to disk.
    fn save_container_state_to_dir(data_dir: &PathBuf, container: &ContainerInfo) -> Result<(), String> {
        let container_dir = data_dir.join("containers").join(&container.id);
        fs::create_dir_all(&container_dir)
            .map_err(|e| format!("Failed to create container dir: {}", e))?;

        let state_path = container_dir.join("state.json");
        let content = serde_json::to_string_pretty(container)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        fs::write(&state_path, content)
            .map_err(|e| format!("Failed to write state: {}", e))?;
        Ok(())
    }

    /// Load container state from disk.
    fn load_container_state_from_dir(data_dir: &PathBuf, id: &str) -> Result<ContainerInfo, String> {
        let state_path = data_dir.join("containers").join(id).join("state.json");
        if !state_path.exists() {
            return Err(format!("Container not found: {}", id));
        }
        let content = fs::read_to_string(&state_path)
            .map_err(|e| format!("Failed to read state: {}", e))?;
        let container: ContainerInfo = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse state: {}", e))?;
        Ok(container)
    }

    /// Delete container state from disk.
    fn delete_container_state_from_dir(data_dir: &PathBuf, id: &str) -> Result<(), String> {
        let container_dir = data_dir.join("containers").join(id);
        if container_dir.exists() {
            fs::remove_dir_all(&container_dir)
                .map_err(|e| format!("Failed to remove container dir: {}", e))?;
        }
        Ok(())
    }

    /// Ensure the VM is running. Starts it if not.
    fn ensure_vm_running(&self) -> Result<(), String> {
        let needs_start = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.status != BackendStatus::Running
        };

        if needs_start {
            let config = {
                let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                state.config.clone().ok_or("Backend not initialized")?
            };

            let (vmm, vsock, cid) = Self::start_vm_inner(&config)?;

            let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.vmm = Some(vmm);
            state.vsock = Some(vsock);
            state.vsock_cid = cid;
            state.status = BackendStatus::Running;
            state.data_dir = config.data_dir.clone();
        }

        Ok(())
    }

    /// Check if VM is running (without holding the lock).
    fn is_vm_running(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.status == BackendStatus::Running)
            .unwrap_or(false)
    }
}

impl Default for MicroVmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuntimeBackend for MicroVmBackend {
    fn backend_name(&self) -> &str {
        "microvm"
    }

    fn is_available(&self) -> bool {
        vmm::check_qemu_available()
    }

    async fn initialize(&mut self, config: &BackendConfig) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.config = Some(config.clone());
        state.data_dir = config.data_dir.clone();
        state.status = BackendStatus::NotStarted;

        // Ensure data directories exist
        fs::create_dir_all(&config.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        fs::create_dir_all(&config.containers_dir())
            .map_err(|e| format!("Failed to create containers dir: {}", e))?;
        fs::create_dir_all(&config.images_dir())
            .map_err(|e| format!("Failed to create images dir: {}", e))?;

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_vm_running()
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo, String> {
        // Ensure VM is running
        self.ensure_vm_running()?;

        // Set up port forwarding if specified
        {
            let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            for pf in &spec.port_forwards {
                let guest_ip = std::net::Ipv4Addr::new(10, 0, 2, 15);
                state.port_forwarder.add_forward(
                    pf.host_port,
                    guest_ip,
                    pf.guest_port,
                    match pf.protocol {
                        Protocol::Tcp => "tcp",
                        Protocol::Udp => "udp",
                    },
                ).map_err(|e| format!("Port forward error: {}", e))?;
            }
        }

        // Send container create command to the guest agent via vsock
        let vm_spec = Self::to_vm_spec(spec);
        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerCreate {
                id: id.to_string(),
                spec: vm_spec,
            })?
        };

        match response {
            VmToHost::ContainerCreated { id: vm_id, pid } => {
                info!("Container created in VM: id={}, pid={}", vm_id, pid);

                let container = ContainerInfo {
                    id: id.to_string(),
                    name: None,
                    image: spec.image.clone(),
                    status: ContainerStatus::Created,
                    pid: Some(pid),
                    exit_code: None,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    labels: spec.labels.clone(),
                    rootfs_path: spec.rootfs_path.clone(),
                    log_path: None,
                };

                // Track in-memory
                {
                    let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    state.containers.insert(id.to_string(), container.clone());
                }

                // Persist to disk
                let data_dir = {
                    let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    state.data_dir.clone()
                };
                Self::save_container_state_to_dir(&data_dir, &container)?;

                Ok(container)
            }
            VmToHost::Error { request_type: _, message, code } => {
                error!(
                    "Guest agent error creating container: code={}, msg={}",
                    code, message
                );
                Err(format!("Guest agent error ({}): {}", code, message))
            }
            other => {
                warn!("Unexpected response from guest agent: {:?}", other);
                Err(format!("Unexpected response from guest agent: {:?}", other))
            }
        }
    }

    async fn start_container(&self, id: &str) -> Result<(), String> {
        // Ensure VM is running
        self.ensure_vm_running()?;

        // Send start command to the guest agent
        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerStart {
                id: id.to_string(),
            })?
        };

        match response {
            VmToHost::ContainerStarted { id: vm_id } => {
                info!("Container started in VM: id={}", vm_id);

                // Update in-memory state
                {
                    let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    if let Some(container) = state.containers.get_mut(id) {
                        container.status = ContainerStatus::Running;
                    }
                }

                // Persist updated state
                let data_dir = {
                    let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    state.data_dir.clone()
                };
                let mut container = Self::load_container_state_from_dir(&data_dir, id)?;
                container.status = ContainerStatus::Running;
                Self::save_container_state_to_dir(&data_dir, &container)?;

                Ok(())
            }
            VmToHost::Error { request_type: _, message, code } => {
                Err(format!("Guest agent error ({}): {}", code, message))
            }
            other => Err(format!("Unexpected response from guest agent: {:?}", other)),
        }
    }

    async fn kill_container(&self, id: &str, signal: i32) -> Result<(), String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerKill {
                id: id.to_string(),
                signal,
            })?
        };

        match response {
            VmToHost::ContainerExited { id: vm_id, exit_code, timestamp } => {
                info!(
                    "Container killed in VM: id={}, exit_code={}, at={}",
                    vm_id, exit_code, timestamp
                );

                // Update in-memory state
                {
                    let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    if let Some(container) = state.containers.get_mut(id) {
                        container.status = ContainerStatus::Stopped;
                        container.exit_code = Some(exit_code);
                    }
                }

                // Persist
                let data_dir = {
                    let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    state.data_dir.clone()
                };
                let mut container = Self::load_container_state_from_dir(&data_dir, id)?;
                container.status = ContainerStatus::Stopped;
                container.exit_code = Some(exit_code);
                Self::save_container_state_to_dir(&data_dir, &container)?;

                Ok(())
            }
            VmToHost::Error { request_type: _, message, code } => {
                Err(format!("Guest agent error ({}): {}", code, message))
            }
            other => Err(format!("Unexpected response from guest agent: {:?}", other)),
        }
    }

    async fn delete_container(&self, id: &str, force: bool) -> Result<(), String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        // If container is running, kill it first (if force)
        let data_dir = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.data_dir.clone()
        };

        let container = Self::load_container_state_from_dir(&data_dir, id)?;
        if container.status == ContainerStatus::Running {
            if force {
                self.kill_container(id, libc::SIGKILL).await?;
                std::thread::sleep(Duration::from_millis(100));
            } else {
                return Err("Container is running. Use force=true to delete".to_string());
            }
        }

        // Send delete command to the guest agent
        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerDelete {
                id: id.to_string(),
                force,
            })?
        };

        match response {
            VmToHost::Error { code, message, .. } => {
                // If the error is "not found", we can still clean up locally
                if code != "CONTAINER_NOT_FOUND" {
                    return Err(format!("Guest agent error ({}): {}", code, message));
                }
                warn!("Container not found in VM, cleaning up locally: {}", id);
            }
            other => {
                debug!("Delete response: {:?}", other);
            }
        }

        // Remove from in-memory tracking
        {
            let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.containers.remove(id);
            state.port_forwarder.clear();
        }

        // Remove from disk
        Self::delete_container_state_from_dir(&data_dir, id)?;

        Ok(())
    }

    async fn exec_in_container(
        &self,
        id: &str,
        command: &[String],
        tty: bool,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerExec {
                id: id.to_string(),
                command: command.to_vec(),
                tty,
                interactive: false,
                env: env.clone(),
            })?
        };

        match response {
            VmToHost::ContainerLog { stream, data, .. } => {
                let (stdout, stderr) = match stream {
                    VmLogStream::Stdout => (data.into_bytes(), Vec::new()),
                    VmLogStream::Stderr => (Vec::new(), data.into_bytes()),
                };
                Ok(ExecResult {
                    exit_code: 0,
                    stdout,
                    stderr,
                })
            }
            VmToHost::Error { code, message, .. } => {
                Err(format!("Guest agent error ({}): {}", code, message))
            }
            other => Err(format!("Unexpected response from guest agent: {:?}", other)),
        }
    }

    async fn container_stats(&self, id: &str) -> Result<ContainerStats, String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let response = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            Self::send_vm_command(&state, &HostToVm::ContainerStats {
                id: id.to_string(),
            })?
        };

        match response {
            VmToHost::ContainerStatsResponse {
                id: _,
                cpu_usage_ns,
                memory_usage_bytes,
                memory_limit_bytes,
                network_rx_bytes,
                network_tx_bytes,
                pids,
            } => Ok(ContainerStats {
                cpu_usage_ns,
                memory_usage_bytes,
                memory_limit_bytes,
                network_rx_bytes,
                network_tx_bytes,
                block_read_bytes: 0,
                block_write_bytes: 0,
                pids,
            }),
            VmToHost::Error { code, message, .. } => {
                Err(format!("Guest agent error ({}): {}", code, message))
            }
            other => Err(format!("Unexpected response from guest agent: {:?}", other)),
        }
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let containers: Vec<ContainerInfo> = state.containers.values().cloned().collect();
        Ok(containers)
    }

    async fn container_logs(&self, id: &str, opts: &LogReadOpts) -> Result<Vec<LogEntry>, String> {
        let data_dir = {
            let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.data_dir.clone()
        };

        let container_dir = data_dir.join("containers").join(id);
        let log_path = container_dir.join("container.log");

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_path)
            .map_err(|e| format!("Failed to read log file: {}", e))?;

        let mut entries: Vec<LogEntry> = content
            .lines()
            .map(|line| LogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                stream: LogStream::Stdout,
                message: line.to_string(),
            })
            .collect();

        if let Some(tail) = opts.tail {
            let start = if entries.len() > tail {
                entries.len() - tail
            } else {
                0
            };
            entries = entries[start..].to_vec();
        }

        Ok(entries)
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.status = BackendStatus::ShuttingDown;

        // Try to send shutdown command to the guest agent
        if let Some(ref vsock) = state.vsock {
            let _ = vsock.send(&HostToVm::VmShutdown);
            std::thread::sleep(Duration::from_millis(500));
        }

        // Close the vsock connection
        state.vsock = None;

        // Stop the VMM process
        if let Some(mut vmm) = state.vmm.take() {
            info!("Stopping VMM process...");
            let _ = vmm.stop();
        }

        // Clean up port forwards
        state.port_forwarder.clear();

        // Update container states to stopped
        for container in state.containers.values_mut() {
            if container.status == ContainerStatus::Running {
                container.status = ContainerStatus::Stopped;
            }
        }

        state.status = BackendStatus::NotStarted;
        info!("MicroVM backend shut down successfully");

        Ok(())
    }

    // ── File Operations via vsock ──────────────────────────────────
    // These are implemented by sending exec commands to the guest agent
    // which runs the equivalent of ls, cat, etc. inside the VM.

    async fn list_files(&self, id: &str, path: &str) -> Result<Vec<FileInfo>, String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        // Execute `ls -la` inside the container via vsock
        let ls_output = self
            .exec_in_container(
                id,
                &[
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    format!("ls -la --time-style=+%s {}", path),
                ],
                false,
                &HashMap::new(),
            )
            .await?;

        let stdout = String::from_utf8_lossy(&ls_output.stdout);
        let mut files = Vec::new();

        for line in stdout.lines().skip(1) {
            if line.starts_with("total") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            let perms = parts[0].to_string();
            let name = parts[8].to_string();
            if name == "." || name == ".." {
                continue;
            }

            let is_dir = perms.starts_with('d');
            let size = parts[4].parse::<u64>().unwrap_or(0);
            let modified = parts[5].to_string();

            files.push(FileInfo {
                name,
                path: format!("{}/{}", path.trim_end_matches('/'), parts[8]),
                is_dir,
                size,
                permissions: perms,
                modified,
            });
        }

        // Sort: directories first, then alphabetically
        files.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.cmp(&b.name)
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(files)
    }

    async fn read_file(&self, id: &str, path: &str) -> Result<Vec<u8>, String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let result = self
            .exec_in_container(
                id,
                &["/bin/cat".to_string(), path.to_string()],
                false,
                &HashMap::new(),
            )
            .await?;

        if result.exit_code != 0 {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to read file: {}", stderr));
        }

        Ok(result.stdout)
    }

    async fn write_file(&self, id: &str, path: &str, content: &[u8]) -> Result<(), String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        // Use hex encoding to safely transfer binary content via the command
        // This avoids the need for an external base64 crate
        let hex_content = hex::encode(content);

        // Write via xxd decode inside the container (more portable than base64)
        let cmd = format!("echo '{}' | xxd -r -p > {}", hex_content, path);
        let result = self
            .exec_in_container(
                id,
                &["/bin/sh".to_string(), "-c".to_string(), cmd],
                false,
                &HashMap::new(),
            )
            .await?;

        if result.exit_code != 0 {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to write file: {}", stderr));
        }

        Ok(())
    }

    async fn delete_file(&self, id: &str, path: &str) -> Result<(), String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let result = self
            .exec_in_container(
                id,
                &["/bin/rm".to_string(), "-rf".to_string(), path.to_string()],
                false,
                &HashMap::new(),
            )
            .await?;

        if result.exit_code != 0 {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to delete file: {}", stderr));
        }

        Ok(())
    }

    async fn create_dir(&self, id: &str, path: &str) -> Result<(), String> {
        if !self.is_vm_running() {
            return Err("MicroVM is not running".to_string());
        }

        let result = self
            .exec_in_container(
                id,
                &["/bin/mkdir".to_string(), "-p".to_string(), path.to_string()],
                false,
                &HashMap::new(),
            )
            .await?;

        if result.exit_code != 0 {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to create directory: {}", stderr));
        }

        Ok(())
    }

    async fn upload_file(&self, id: &str, host_path: &str, container_path: &str) -> Result<(), String> {
        let host_file = PathBuf::from(host_path);
        if !host_file.exists() {
            return Err(format!("Host file not found: {}", host_path));
        }

        let content = fs::read(&host_file)
            .map_err(|e| format!("Failed to read host file: {}", e))?;

        self.write_file(id, container_path, &content).await
    }

    async fn download_file(&self, id: &str, container_path: &str, host_path: &str) -> Result<(), String> {
        let content = self.read_file(id, container_path).await?;

        let host_file = PathBuf::from(host_path);
        if let Some(parent) = host_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create host directory: {}", e))?;
        }

        fs::write(&host_file, content)
            .map_err(|e| format!("Failed to write host file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microvm_backend_new() {
        let backend = MicroVmBackend::new();
        assert_eq!(backend.backend_name(), "microvm");
        assert!(!backend.is_running());
    }

    #[test]
    fn test_microvm_backend_default() {
        let backend = MicroVmBackend::default();
        assert_eq!(backend.backend_name(), "microvm");
    }

    #[test]
    fn test_allocate_cid() {
        let backend = MicroVmBackend::new();
        let cid = backend.allocate_cid();
        assert!(cid >= 3);
    }

    #[test]
    fn test_to_vm_spec() {
        let spec = ContainerSpec {
            image: "alpine:latest".to_string(),
            command: vec!["/bin/sh".to_string()],
            ..Default::default()
        };
        let vm_spec = MicroVmBackend::to_vm_spec(&spec);
        assert_eq!(vm_spec.command, vec!["/bin/sh"]);
    }

    #[test]
    fn test_backend_status() {
        let state = MicroVmState {
            config: None,
            vmm: None,
            vsock: None,
            port_forwarder: PortForwarder::new(),
            status: BackendStatus::NotStarted,
            containers: HashMap::new(),
            vsock_cid: 0,
            data_dir: PathBuf::new(),
        };
        assert_eq!(state.status, BackendStatus::NotStarted);
    }

    #[test]
    fn test_save_load_container_state() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = PathBuf::from(dir.path());

        let container = ContainerInfo {
            id: "test-container-123".to_string(),
            name: Some("test".to_string()),
            image: "alpine:latest".to_string(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            labels: HashMap::new(),
            rootfs_path: None,
            log_path: None,
        };

        MicroVmBackend::save_container_state_to_dir(&data_dir, &container).unwrap();
        let loaded = MicroVmBackend::load_container_state_from_dir(&data_dir, "test-container-123").unwrap();
        assert_eq!(loaded.id, "test-container-123");
        assert_eq!(loaded.status, ContainerStatus::Created);

        MicroVmBackend::delete_container_state_from_dir(&data_dir, "test-container-123").unwrap();
        assert!(MicroVmBackend::load_container_state_from_dir(&data_dir, "test-container-123").is_err());
    }
}
