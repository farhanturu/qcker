use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::fs;
use std::path::{Path, PathBuf};

use crate::capability;
use crate::cgroup;
use crate::namespace;
use crate::seccomp;
use crate::spec::{Container, ContainerState, OciConfig};
use qcker_common::error::{QckerError, Result};

pub struct ContainerProcess {
    pub container: Container,
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub memory_swap_mb: Option<u64>,
    pub pids_limit: Option<i64>,
    pub cpu_shares: Option<u64>,
    pub gpu_enabled: bool,
    pub gpu_devices: Vec<String>,
    pub vram_mb: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: None,
            memory_mb: None,
            memory_swap_mb: None,
            pids_limit: Some(256),
            cpu_shares: Some(1024),
            gpu_enabled: false,
            gpu_devices: Vec::new(),
            vram_mb: None,
        }
    }
}

impl ContainerProcess {
    pub fn new(id: &str, bundle: &Path, config: OciConfig, data_dir: PathBuf) -> Result<Self> {
        let container = Container::new(id.to_string(), bundle.to_path_buf(), config);
        let container_dir = data_dir.join("containers").join(id);

        fs::create_dir_all(&container_dir)
            .map_err(|e| QckerError::Process(format!("Failed to create container dir: {}", e)))?;

        let state_path = container_dir.join("state.json");
        let state_json = serde_json::to_string_pretty(&container)
            .map_err(|e| QckerError::Process(format!("Failed to serialize state: {}", e)))?;
        fs::write(&state_path, state_json)
            .map_err(|e| QckerError::Process(format!("Failed to write state: {}", e)))?;

        Ok(Self { container, data_dir })
    }

    pub fn create(&mut self) -> Result<()> {
        let container_dir = self.data_dir.join("containers").join(&self.container.id);
        fs::create_dir_all(&container_dir)
            .map_err(|e| QckerError::Process(format!("Failed to create container dir: {}", e)))?;

        self.container.state = ContainerState::Created;
        self.save_state()?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.container.state != ContainerState::Created {
            return Err(QckerError::Process(format!(
                "Container is in state {:?}, expected Created",
                self.container.state
            )));
        }

        let rootfs = self.container.rootfs.clone();
        let process_config = self.container.config.process.clone()
            .ok_or_else(|| QckerError::Process("No process configuration".to_string()))?;

        let (pipe_read, pipe_write) = nix::unistd::pipe()
            .map_err(|e| QckerError::Process(format!("Failed to create pipe: {}", e)))?;

        match unsafe { fork() }.map_err(|e| QckerError::Process(format!("Fork failed: {}", e)))? {
            ForkResult::Parent { child } => {
                nix::unistd::close(pipe_write)
                    .map_err(|e| QckerError::Process(format!("Failed to close pipe: {}", e)))?;

                self.container.pid = Some(child.as_raw());
                self.container.state = ContainerState::Running;
                self.save_state()?;

                let mut buf = [0u8; 1];
                let _ = nix::unistd::read(pipe_read, &mut buf);
                nix::unistd::close(pipe_read).ok();

                Ok(())
            }
            ForkResult::Child => {
                nix::unistd::close(pipe_read)
                    .map_err(|e| QckerError::Process(format!("Failed to close pipe: {}", e)))?;

                if let Err(e) = self.child_process(&rootfs, &process_config) {
                    eprintln!("qcker: {}", e);
                    nix::unistd::close(pipe_write).ok();
                    std::process::exit(1);
                }

                nix::unistd::close(pipe_write).ok();
                std::process::exit(0);
            }
        }
    }

    fn child_process(&self, rootfs: &Path, process_config: &crate::spec::ProcessConfig) -> Result<()> {
        use nix::sched::{unshare, CloneFlags};
        use nix::unistd::chroot;

        if let Some(ref linux) = self.container.config.linux {
            let mut flags = CloneFlags::empty();
            for ns in &linux.namespaces {
                match ns.r#type {
                    crate::spec::NamespaceType::Pid => flags |= CloneFlags::CLONE_NEWPID,
                    crate::spec::NamespaceType::Network => flags |= CloneFlags::CLONE_NEWNET,
                    crate::spec::NamespaceType::Mount => flags |= CloneFlags::CLONE_NEWNS,
                    crate::spec::NamespaceType::Uts => flags |= CloneFlags::CLONE_NEWUTS,
                    crate::spec::NamespaceType::Ipc => flags |= CloneFlags::CLONE_NEWIPC,
                    crate::spec::NamespaceType::Cgroup => flags |= CloneFlags::CLONE_NEWCGROUP,
                    crate::spec::NamespaceType::User => {}
                }
            }
            if !flags.is_empty() {
                let _ = unshare(flags);
            }
        }

        chroot(rootfs)
            .map_err(|e| QckerError::Process(format!("Failed to chroot: {}", e)))?;

        std::env::set_current_dir("/")
            .map_err(|e| QckerError::Process(format!("Failed to chdir: {}", e)))?;

        nix::unistd::sethostname("container")
            .map_err(|e| QckerError::Process(format!("Failed to set hostname: {}", e)))?;

        let _ = seccomp::apply_default_profile();
        let _ = capability::drop_all_capabilities();

        for env in &process_config.env {
            let parts: Vec<&str> = env.splitn(2, '=').collect();
            if parts.len() == 2 {
                std::env::set_var(parts[0], parts[1]);
            }
        }

        let _ = std::env::set_current_dir(&process_config.cwd);

        if process_config.args.is_empty() {
            return Err(QckerError::Process("No command specified".to_string()));
        }

        use std::ffi::CString;
        let cmd = &process_config.args[0];
        let c_cmd = CString::new(cmd.as_str())
            .map_err(|e| QckerError::Process(format!("Invalid command: {}", e)))?;

        let c_args: Vec<CString> = process_config.args.iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        let mut c_args_ptrs: Vec<*const libc::c_char> = c_args.iter()
            .map(|s| s.as_ptr())
            .collect();
        c_args_ptrs.push(std::ptr::null());

        unsafe {
            libc::execvp(c_cmd.as_ptr(), c_args_ptrs.as_ptr());
        }

        Err(QckerError::Process(format!("Failed to execute {}: {}", cmd, std::io::Error::last_os_error())))
    }

    pub fn start_interactive(&mut self) -> Result<()> {
        if self.container.state != ContainerState::Created {
            return Err(QckerError::Process(format!(
                "Container is in state {:?}, expected Created",
                self.container.state
            )));
        }

        let rootfs = self.container.rootfs.clone();
        let process_config = self.container.config.process.clone()
            .ok_or_else(|| QckerError::Process("No process configuration".to_string()))?;

        match unsafe { fork() }.map_err(|e| QckerError::Process(format!("Fork failed: {}", e)))? {
            ForkResult::Parent { child } => {
                self.container.pid = Some(child.as_raw());
                self.container.state = ContainerState::Running;
                self.save_state()?;

                let status = waitpid(child, None);
                self.container.state = ContainerState::Stopped;
                self.save_state()?;

                match status {
                    Ok(WaitStatus::Exited(_, code)) => {
                        if code != 0 {
                            return Err(QckerError::Process(format!("Container exited with code {}", code)));
                        }
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        return Err(QckerError::Process(format!("Container killed by signal {:?}", signal)));
                    }
                    Err(nix::errno::Errno::ECHILD) => {}
                    _ => {}
                }
                Ok(())
            }
            ForkResult::Child => {
                if let Err(e) = self.child_process(&rootfs, &process_config) {
                    eprintln!("qcker: {}", e);
                    std::process::exit(1);
                }
                std::process::exit(0);
            }
        }
    }

    pub fn apply_resource_limits(&self, limits: &ResourceLimits) -> Result<()> {
        let container_id = &self.container.id;

        match cgroup::create_cgroup(container_id) {
            Ok(cgroup_path) => {
                if let Some(cpu_cores) = limits.cpu_cores {
                    let period = 100000u64;
                    let quota = (cpu_cores * period as f64) as u64;
                    let path = cgroup_path.join("cpu.max");
                    if let Err(e) = fs::write(&path, format!("{} {}", quota, period)) {
                        tracing::warn!("Failed to set CPU limit (may need root): {}", e);
                    }
                }

                if let Some(shares) = limits.cpu_shares {
                    let path = cgroup_path.join("cpu.weight");
                    if let Err(e) = fs::write(&path, shares.to_string()) {
                        tracing::warn!("Failed to set CPU shares: {}", e);
                    }
                }

                if let Some(memory_mb) = limits.memory_mb {
                    let path = cgroup_path.join("memory.max");
                    let bytes = memory_mb * 1024 * 1024;
                    if let Err(e) = fs::write(&path, bytes.to_string()) {
                        tracing::warn!("Failed to set memory limit (may need root): {}", e);
                    }
                }

                if let Some(swap_mb) = limits.memory_swap_mb {
                    let path = cgroup_path.join("memory.swap.max");
                    let bytes = swap_mb * 1024 * 1024;
                    if let Err(e) = fs::write(&path, bytes.to_string()) {
                        tracing::warn!("Failed to set swap limit: {}", e);
                    }
                }

                if let Some(pids) = limits.pids_limit {
                    let path = cgroup_path.join("pids.max");
                    if let Err(e) = fs::write(&path, pids.to_string()) {
                        tracing::warn!("Failed to set PIDs limit: {}", e);
                    }
                }

                tracing::info!("Resource limits applied for container {}", container_id);
            }
            Err(e) => {
                tracing::warn!("Failed to create cgroup (may need root): {}", e);
            }
        }

        if limits.gpu_enabled {
            let _ = self.setup_gpu_access(limits);
        }

        Ok(())
    }

    fn setup_gpu_access(&self, limits: &ResourceLimits) -> Result<()> {
        let container_root = self.data_dir.join("containers").join(&self.container.id).join("rootfs");
        let dev_path = container_root.join("dev");

        fs::create_dir_all(&dev_path)
            .map_err(|e| QckerError::Process(format!("Failed to create dev dir: {}", e)))?;

        let gpu_devices = if limits.gpu_devices.is_empty() {
            self.detect_gpu_devices()?
        } else {
            limits.gpu_devices.clone()
        };

        for device in &gpu_devices {
            let device_path = Path::new(device);
            if device_path.exists() {
                let target = dev_path.join(device_path.file_name().unwrap_or_default());
                if !target.exists() {
                    fs::copy(device_path, &target)
                        .map_err(|e| QckerError::Process(format!("Failed to copy GPU device: {}", e)))?;
                }
            }
        }

        tracing::info!("GPU access enabled for container {}", self.container.id);
        Ok(())
    }

    fn detect_gpu_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();

        let nvidia_path = Path::new("/dev/nvidia0");
        if nvidia_path.exists() {
            devices.push("/dev/nvidia0".to_string());
            devices.push("/dev/nvidiactl".to_string());
            devices.push("/dev/nvidia-uvm".to_string());
        }

        let dri_path = Path::new("/dev/dri");
        if dri_path.exists() {
            if let Ok(entries) = fs::read_dir(dri_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.to_string_lossy().contains("render") || path.to_string_lossy().contains("card") {
                        devices.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(devices)
    }

    pub fn kill(&self, sig: Signal) -> Result<()> {
        if let Some(pid) = self.container.pid {
            unsafe {
                let ret = libc::kill(pid, sig as i32);
                if ret != 0 {
                    let err = std::io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::ESRCH) {
                        return Err(QckerError::Process(format!("Failed to send signal: {}", err)));
                    }
                }
            }
            Ok(())
        } else {
            Err(QckerError::Process("Container has no PID".to_string()))
        }
    }

    pub fn wait(&self) -> Result<i32> {
        if let Some(pid) = self.container.pid {
            loop {
                match waitpid(Pid::from_raw(pid), Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::Exited(_, code)) => return Ok(code),
                    Ok(WaitStatus::Signaled(_, signal, _)) => return Ok(128 + signal as i32),
                    Ok(WaitStatus::StillAlive) => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(nix::errno::Errno::ECHILD) => return Ok(0),
                    _ => return Ok(0),
                }
            }
        } else {
            Err(QckerError::Process("Container has no PID".to_string()))
        }
    }

    pub fn delete(&mut self) -> Result<()> {
        if self.container.state == ContainerState::Running {
            let _ = self.kill(Signal::SIGKILL);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if let Ok(cgroup_path) = cgroup::create_cgroup(&self.container.id) {
            let _ = cgroup::remove_cgroup(&cgroup_path);
        }

        let container_dir = self.data_dir.join("containers").join(&self.container.id);
        if container_dir.exists() {
            fs::remove_dir_all(&container_dir)
                .map_err(|e| QckerError::Process(format!("Failed to remove container dir: {}", e)))?;
        }

        self.container.state = ContainerState::Stopped;
        Ok(())
    }

    fn save_state(&self) -> Result<()> {
        let container_dir = self.data_dir.join("containers").join(&self.container.id);
        let state_path = container_dir.join("state.json");
        let state_json = serde_json::to_string_pretty(&self.container)
            .map_err(|e| QckerError::Process(format!("Failed to serialize state: {}", e)))?;
        fs::write(&state_path, state_json)
            .map_err(|e| QckerError::Process(format!("Failed to write state: {}", e)))?;
        Ok(())
    }

    pub fn load_state(data_dir: &Path, container_id: &str) -> Result<Container> {
        let state_path = data_dir
            .join("containers")
            .join(container_id)
            .join("state.json");
        let state_json = fs::read_to_string(&state_path)
            .map_err(|e| QckerError::Process(format!("Failed to read state: {}", e)))?;
        let container: Container = serde_json::from_str(&state_json)
            .map_err(|e| QckerError::Process(format!("Failed to parse state: {}", e)))?;
        Ok(container)
    }

    pub fn exec(&self, command: &[String]) -> Result<()> {
        let rootfs = self.container.rootfs.clone();

        let (pipe_read, pipe_write) = nix::unistd::pipe()
            .map_err(|e| QckerError::Process(format!("Failed to create pipe: {}", e)))?;

        match unsafe { fork() }.map_err(|e| QckerError::Process(format!("Fork failed: {}", e)))? {
            ForkResult::Parent { child } => {
                nix::unistd::close(pipe_write).ok();

                let mut buf = [0u8; 1];
                let _ = nix::unistd::read(pipe_read, &mut buf);
                nix::unistd::close(pipe_read).ok();

                let status = waitpid(child, None);
                match status {
                    Ok(WaitStatus::Exited(_, code)) => {
                        if code != 0 {
                            return Err(QckerError::Process(format!("Exec exited with code {}", code)));
                        }
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        return Err(QckerError::Process(format!("Exec killed by signal {:?}", signal)));
                    }
                    Err(nix::errno::Errno::ECHILD) => {}
                    _ => {}
                }
                Ok(())
            }
            ForkResult::Child => {
                nix::unistd::close(pipe_read).ok();

                use nix::unistd::chroot;
                let _ = chroot(rootfs.as_path());
                let _ = std::env::set_current_dir("/");

                let _ = seccomp::apply_default_profile();
                let _ = capability::drop_all_capabilities();

                nix::unistd::close(pipe_write).ok();

                use std::ffi::CString;
                if !command.is_empty() {
                    let cmd = &command[0];
                    let c_cmd = CString::new(cmd.as_str()).unwrap();
                    let c_args: Vec<CString> = command.iter()
                        .map(|s| CString::new(s.as_str()).unwrap())
                        .collect();
                    let mut c_args_ptrs: Vec<*const libc::c_char> = c_args.iter()
                        .map(|s| s.as_ptr())
                        .collect();
                    c_args_ptrs.push(std::ptr::null());

                    unsafe {
                        libc::execvp(c_cmd.as_ptr(), c_args_ptrs.as_ptr());
                    }
                }
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ProcessConfig, RootConfig, UserConfig};

    #[test]
    fn test_container_state() {
        let config = OciConfig {
            oci_version: "1.0.0".to_string(),
            root: RootConfig { path: PathBuf::from("rootfs"), readonly: false },
            process: Some(ProcessConfig {
                terminal: false,
                user: UserConfig { uid: 0, gid: 0 },
                args: vec!["echo".to_string(), "hello".to_string()],
                env: vec![],
                cwd: "/".to_string(),
                capabilities: None,
                rlimits: vec![],
                no_new_privileges: false,
            }),
            hostname: Some("test".to_string()),
            mounts: vec![],
            linux: None,
        };
        let container = Container::new("test123".to_string(), PathBuf::from("/tmp/test"), config);
        assert_eq!(container.state, ContainerState::Created);
        assert_eq!(container.id, "test123");
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.pids_limit, Some(256));
        assert_eq!(limits.cpu_shares, Some(1024));
        assert!(!limits.gpu_enabled);
    }
}
