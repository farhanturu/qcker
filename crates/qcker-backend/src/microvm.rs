use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use qcker_common::error::{QckerError, Result};

use crate::config::BackendConfig;
use crate::port_forward::PortForwarder;
use crate::types::*;
use crate::vmm::{self, VmmManager};
use crate::RuntimeBackend;

pub struct MicroVmBackend {
    state: Arc<Mutex<MicroVmState>>,
}

struct MicroVmState {
    config: Option<BackendConfig>,
    vmm: Option<VmmManager>,
    port_forwarder: PortForwarder,
    status: BackendStatus,
}

#[derive(Debug, PartialEq)]
enum BackendStatus {
    NotStarted,
    Booting,
    Running,
    ShuttingDown,
}

impl MicroVmBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MicroVmState {
                config: None,
                vmm: None,
                port_forwarder: PortForwarder::new(),
                status: BackendStatus::NotStarted,
            })),
        }
    }
}

impl Default for MicroVmBackend {
    fn default() -> Self {
        Self::new()
    }
}


fn resolve_kernel_path(path: Option<&std::path::PathBuf>) -> std::path::PathBuf {
    match path {
        Some(p) => p.clone(),
        None => std::path::Path::new("/boot/vmlinuz").to_path_buf(),
    }
}

fn resolve_rootfs_path(path: Option<&std::path::PathBuf>) -> std::path::PathBuf {
    match path {
        Some(p) => p.clone(),
        None => std::path::Path::new("/tmp/rootfs").to_path_buf(),
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

    async fn initialize(&mut self, config: &BackendConfig) -> Result<()> {
        let mut state = self.state.lock()
            .map_err(|e| QckerError::internal(format!("Lock error: {}", e)))?;
        state.config = Some(config.clone());
        state.status = BackendStatus::Booting;
        let vmm_config = vmm::VmmConfig {
            kernel_path: resolve_kernel_path(config.kernel_path.as_ref()),
            rootfs_path: resolve_rootfs_path(config.rootfs_path.as_ref()),
            vcpu_count: 2,
            memory_mb: 256,
            use_acceleration: true,
            kernel_cmdline_extra: vec![],
            vsock_cid: 2,
            fs_shares: vec![],
        };
        let vmm = vmm::VmmManager::start(vmm_config)
            .map_err(|e| QckerError::internal(format!("Failed to start VMM: {}", e)))?;
        state.vmm = Some(vmm);
        state.status = BackendStatus::Running;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.status == BackendStatus::Running)
            .unwrap_or(false)
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo> {
        let state = self.state.lock()
            .map_err(|e| QckerError::internal(format!("Lock error: {}", e)))?;

        if state.status != BackendStatus::Running {
            return Err(QckerError::process("VM is not running".to_string()));
        }

        let container = ContainerInfo {
            id: id.to_string(),
            name: None,
            image: spec.command.first().cloned().unwrap_or_default(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            labels: HashMap::new(),
        };

        Ok(container)
    }

    async fn start_container(&self, _id: &str) -> Result<()> {
        let state = self.state.lock()
            .map_err(|e| QckerError::internal(format!("Lock error: {}", e)))?;
        if state.status != BackendStatus::Running {
            return Err(QckerError::process("VM is not running".to_string()));
        }
        Ok(())
    }

    async fn kill_container(&self, _id: &str, _signal: i32) -> Result<()> {
        Ok(())
    }

    async fn delete_container(&self, _id: &str, _force: bool) -> Result<()> {
        Ok(())
    }

    async fn exec_in_container(
        &self,
        _id: &str,
        _command: &[String],
        _tty: bool,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult> {
        Err(QckerError::not_supported("exec not yet implemented for MicroVM backend".to_string()))
    }

    async fn container_stats(&self, _id: &str) -> Result<ContainerStats> {
        Ok(ContainerStats {
            cpu_usage_ns: 0,
            memory_usage_bytes: 0,
            memory_limit_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            block_read_bytes: 0,
            block_write_bytes: 0,
            pids: 0,
        })
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        Ok(Vec::new())
    }

    async fn container_logs(&self, _id: &str, _opts: &LogReadOpts) -> Result<Vec<LogEntry>> {
        Ok(Vec::new())
    }

    async fn shutdown(&mut self) -> Result<()> {
        let mut state = self.state.lock()
            .map_err(|e| QckerError::internal(format!("Lock error: {}", e)))?;
        state.status = BackendStatus::ShuttingDown;

        if let Some(mut vmm) = state.vmm.take() {
            let _ = vmm.stop();
        }

        state.port_forwarder.clear();
        state.status = BackendStatus::NotStarted;

        Ok(())
    }

    async fn list_files(&self, _id: &str, _path: &str) -> Result<Vec<FileInfo>> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn read_file(&self, _id: &str, _path: &str) -> Result<Vec<u8>> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn write_file(&self, _id: &str, _path: &str, _content: &[u8]) -> Result<()> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn delete_file(&self, _id: &str, _path: &str) -> Result<()> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn create_dir(&self, _id: &str, _path: &str) -> Result<()> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn upload_file(&self, _id: &str, _host_path: &str, _container_path: &str) -> Result<()> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }

    async fn download_file(&self, _id: &str, _container_path: &str, _host_path: &str) -> Result<()> {
        Err(QckerError::not_supported("File operations not yet implemented for MicroVM backend".to_string()))
    }
}

