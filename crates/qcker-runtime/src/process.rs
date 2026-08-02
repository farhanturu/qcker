use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::fs;
use std::path::{Path, PathBuf};

use crate::capability;
use crate::cgroup;
use crate::mount as mount_module;
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
            .map_err(|e| QckerError::process(format!("Failed to create container dir: {}", e)))?;

        let state_path = container_dir.join("state.json");
        let state_json = serde_json::to_string_pretty(&container)
            .map_err(|e| QckerError::process(format!("Failed to serialize state: {}", e)))?;
        fs::write(&state_path, state_json)
            .map_err(|e| QckerError::process(format!("Failed to write state: {}", e)))?;

        Ok(Self { container, data_dir })
    }

    pub fn create(&mut self) -> Result<()> {
        let container_dir = self.data_dir.join("containers").join(&self.container.id);
        fs::create_dir_all(&container_dir)
            .map_err(|e| QckerError::process(format!("Failed to create container dir: {}", e)))?;

        self.container.state = ContainerState::Created;
        self.save_state()?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.container.state != ContainerState::Created {
            return Err(QckerError::process(format!(
                "Container is in state {:?}, expected Created",
                self.container.state
            )));
        }

        let rootfs = self.container.rootfs.clone();
        let process_config = self.container.config.process.clone()
            .ok_or_else(|| QckerError::process("No process configuration".to_string()))?;

        let (pipe_read, pipe_write) = nix::unistd::pipe()
            .map_err(|e| QckerError::process(format!("Failed to create pipe: {}", e)))?;

        match unsafe { fork() }.map_err(|e| QckerError::process(format!("Fork failed: {}", e)))? {
            ForkResult::Parent { child } => {
                nix::unistd::close(pipe_write)
                    .map_err(|e| QckerError::process(format!("Failed to close pipe: {}", e)))?;

                let mut buf = [0u8; 1];
                let _ = nix::unistd::read(pipe_read, &mut buf);
                nix::unistd::close(pipe_read).ok();

                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, code)) if code == 0 => {
                        self.container.pid = Some(child.as_raw());
                        self.container.state = ContainerState::Running;
                        self.save_state()?;
                        Ok(())
                    }
                    Ok(WaitStatus::Exited(_, code)) => {
                        self.container.state = ContainerState::Stopped;
                        self.save_state()?;
                        Err(QckerError::process(format!("Child exited with code {}", code)))
                    }
                    Ok(WaitStatus::Signaled(_, sig, _)) => {
                        self.container.state = ContainerState::Stopped;
                        self.save_state()?;
                        Err(QckerError::process(format!("Child killed by signal {}", sig)))
                    }
                    Err(_) => {
                        self.container.state = ContainerState::Stopped;
                        self.save_state()?;
                        Err(QckerError::process("Child process failed".to_string()))
                    }
                    _ => Ok(()),
                }
            }
            ForkResult::Child => {
                nix::unistd::close(pipe_read)
                    .map_err(|e| QckerError::process(format!("Failed to close pipe: {}", e)))?;

                let log_dir = self.data_dir.join("containers").join(&self.container.id);
                fs::create_dir_all(&log_dir).ok();
                let log_path = log_dir.join("container.log");
                if let Ok(log_file) = std::fs::File::create(&log_path) {
                    use std::os::unix::io::AsRawFd;
                    unsafe {
                        libc::dup2(log_file.as_raw_fd(), 1);
                        libc::dup2(log_file.as_raw_fd(), 2);
                    }
                }

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

        let is_root = unsafe { libc::getuid() == 0 };
        let needs_user_ns = !is_root;

        if needs_user_ns {
            match unshare(CloneFlags::CLONE_NEWUSER) {
                Ok(()) => {
                    let host_uid = unsafe { libc::getuid() };
                    let host_gid = unsafe { libc::getgid() };

                    if let Err(e) = fs::write("/proc/self/setgroups", "deny") {
                        tracing::warn!("Failed to write setgroups: {}", e);
                    }
                    if let Err(e) = fs::write("/proc/self/uid_map", format!("0 {} 1", host_uid)) {
                        tracing::warn!("Failed to write uid_map: {}", e);
                    }
                    if let Err(e) = fs::write("/proc/self/gid_map", format!("0 {} 1", host_gid)) {
                        tracing::warn!("Failed to write gid_map: {}", e);
                    }
                    tracing::info!("User namespace created (rootless mode)");
                }
                Err(e) => {
                    tracing::warn!("Cannot create user namespace ({}), continuing without it", e);
                }
            }
        }

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
                match unshare(flags) {
                    Ok(()) => {}
                    Err(e) => {
                        if is_root {
                            return Err(QckerError::process(format!("Failed to unshare namespaces: {}", e)));
                        } else {
                            tracing::warn!("Some namespaces cannot be created ({}), continuing", e);
                        }
                    }
                }
            }
        }

        let mut mount_requested = false;
        if let Some(ref linux) = self.container.config.linux {
            for ns in &linux.namespaces {
                if ns.r#type == crate::spec::NamespaceType::Mount {
                    mount_requested = true;
                }
            }
        }

        if mount_requested {
            mount_module::bind_mount(rootfs, rootfs, true)?;
        }

        let old_root = rootfs.join(".old_root");
        mount_module::pivot_root(rootfs, &old_root)?;
        std::env::set_current_dir("/")?;

        mount_module::mount_proc(rootfs)?;
        mount_module::mount_sys(rootfs)?;
        mount_module::mount_dev(rootfs)?;

        let hostname = self.container.config.hostname.as_deref().unwrap_or(&self.container.id[..12.min(self.container.id.len())]);
        if let Err(e) = nix::unistd::sethostname(hostname) {
            tracing::warn!("Failed to set hostname (expected in rootless): {}", e);
        }

        if is_root || matches!(seccomp::apply_default_profile(), Ok(())) {
            ()
        } else {
            tracing::warn!("Seccomp not applied (rootless mode requires capabilities)");
        }

        if is_root || matches!(capability::drop_all_capabilities(), Ok(())) {
            ()
        } else {
            tracing::warn!("Capability drop failed (rootless mode)");
        }

        for env in &process_config.env {
            let parts: Vec<&str> = env.splitn(2, '=').collect();
            if parts.len() == 2 {
                std::env::set_var(parts[0], parts[1]);
            }
        }

        std::env::set_current_dir(&process_config.cwd)?;

        if process_config.args.is_empty() {
            return Err(QckerError::process("No command specified".to_string()));
        }

        use std::ffi::CString;
        let cmd = &process_config.args[0];
        let c_cmd = CString::new(cmd.as_str())
            .map_err(|e| QckerError::process(format!("Invalid command: {}", e)))?;

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

        Err(QckerError::process(format!("Failed to execute {}: {}", cmd, std::io::Error::last_os_error())))
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
            .map_err(|e| QckerError::process(format!("Failed to create dev dir: {}", e)))?;

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
                        .map_err(|e| QckerError::process(format!("Failed to copy GPU device: {}", e)))?;
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
                        return Err(QckerError::process(format!("Failed to send signal: {}", err)));
                    }
                }
            }
            Ok(())
        } else {
            Err(QckerError::process("Container has no PID".to_string()))
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
            Err(QckerError::process("Container has no PID".to_string()))
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
                .map_err(|e| QckerError::process(format!("Failed to remove container dir: {}", e)))?;
        }

        self.container.state = ContainerState::Stopped;
        Ok(())
    }

    fn save_state(&self) -> Result<()> {
        let container_dir = self.data_dir.join("containers").join(&self.container.id);
        let state_path = container_dir.join("state.json");
        let state_json = serde_json::to_string_pretty(&self.container)
            .map_err(|e| QckerError::process(format!("Failed to serialize state: {}", e)))?;
        fs::write(&state_path, state_json)
            .map_err(|e| QckerError::process(format!("Failed to write state: {}", e)))?;
        Ok(())
    }

    pub fn load_state(data_dir: &Path, container_id: &str) -> Result<Container> {
        let state_path = data_dir
            .join("containers")
            .join(container_id)
            .join("state.json");
        let state_json = fs::read_to_string(&state_path)
            .map_err(|e| QckerError::process(format!("Failed to read state: {}", e)))?;
        let container: Container = serde_json::from_str(&state_json)
            .map_err(|e| QckerError::process(format!("Failed to parse state: {}", e)))?;
        Ok(container)
    }

    pub fn exec(&self, command: &[String], terminal: bool, interactive: bool) -> Result<()> {
        let rootfs = self.container.rootfs.clone();
        let container_pid = self.container.pid;

        let (pipe_read, pipe_write) = nix::unistd::pipe()
            .map_err(|e| QckerError::process(format!("Failed to create pipe: {}", e)))?;

        match unsafe { fork() }.map_err(|e| QckerError::process(format!("Fork failed: {}", e)))? {
            ForkResult::Parent { child } => {
                nix::unistd::close(pipe_write).ok();

                let mut buf = [0u8; 1];
                let _ = nix::unistd::read(pipe_read, &mut buf);
                nix::unistd::close(pipe_read).ok();

                let status = waitpid(child, None);
                match status {
                    Ok(WaitStatus::Exited(_, code)) => {
                        if code != 0 {
                            return Err(QckerError::process(format!("Exec exited with code {}", code)));
                        }
                    }
                    Ok(WaitStatus::Signaled(_, signal, _)) => {
                        return Err(QckerError::process(format!("Exec killed by signal {:?}", signal)));
                    }
                    Err(nix::errno::Errno::ECHILD) => {}
                    _ => {}
                }
                Ok(())
            }
            ForkResult::Child => {
                nix::unistd::close(pipe_read).ok();

                if let Some(pid) = container_pid {
                    let ns_types = vec![
                        ("user", crate::spec::NamespaceType::User),
                        ("mnt", crate::spec::NamespaceType::Mount),
                        ("pid", crate::spec::NamespaceType::Pid),
                        ("net", crate::spec::NamespaceType::Network),
                        ("uts", crate::spec::NamespaceType::Uts),
                        ("ipc", crate::spec::NamespaceType::Ipc),
                        ("cgroup", crate::spec::NamespaceType::Cgroup),
                    ];

                    for (ns_name, _ns_type) in &ns_types {
                        let ns_path = format!("/proc/{}/ns/{}", pid, ns_name);
                        if std::path::Path::new(&ns_path).exists() {
                            let ns_file = std::fs::File::open(&ns_path);
                            if let Ok(ns_file) = ns_file {
                                use std::os::unix::io::AsRawFd;
                                let fd = ns_file.as_raw_fd();
                                unsafe {
                                    libc::setns(fd, 0);
                                }
                            }
                        }
                    }
                }

                use nix::unistd::chroot;
                chroot(rootfs.as_path())
                    .map_err(|e| QckerError::process(format!("Failed to chroot in exec: {}", e)))?;
                std::env::set_current_dir("/")
                    .map_err(|e| QckerError::process(format!("Failed to chdir in exec: {}", e)))?;

                seccomp::apply_default_profile()
                    .map_err(|e| QckerError::process(format!("Failed to apply seccomp in exec: {}", e)))?;
                capability::drop_all_capabilities()
                    .map_err(|e| QckerError::process(format!("Failed to drop capabilities in exec: {}", e)))?;

                nix::unistd::close(pipe_write).ok();

                if terminal || interactive {
                    use nix::unistd::{dup2, setsid};
                    use nix::fcntl::{open, OFlag};
                    use nix::sys::stat::Mode;
                    use std::os::unix::io::AsRawFd;
                    let tty_path = "/dev/tty";
                    if std::path::Path::new(tty_path).exists() {
                        let tty_fd = open(tty_path, OFlag::O_RDWR, Mode::empty())
                            .map_err(|e| QckerError::process(format!("Failed to open tty: {}", e)))?;
                        dup2(tty_fd.as_raw_fd(), 0)
                            .map_err(|e| QckerError::process(format!("Failed to dup2 stdin: {}", e)))?;
                        dup2(tty_fd.as_raw_fd(), 1)
                            .map_err(|e| QckerError::process(format!("Failed to dup2 stdout: {}", e)))?;
                        dup2(tty_fd.as_raw_fd(), 2)
                            .map_err(|e| QckerError::process(format!("Failed to dup2 stderr: {}", e)))?;
                        setsid()
                            .map_err(|e| QckerError::process(format!("Failed to setsid: {}", e)))?;
                    }
                }

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

