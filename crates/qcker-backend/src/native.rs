use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use qcker_common::error::{QckerError, Result};
use qcker_runtime::process::ContainerProcess;
use qcker_runtime::spec::{OciConfig, ProcessConfig, RootConfig, UserConfig};

use crate::config::BackendConfig;
use crate::types::*;
use crate::RuntimeBackend;

fn env_to_vec(env: &HashMap<String, String>) -> Vec<String> {
    env.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
}

pub struct NativeBackend {
    config: Option<BackendConfig>,
    containers_dir: PathBuf,
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeBackend {
    pub fn new() -> Self {
        Self {
            config: None,
            containers_dir: PathBuf::new(),
        }
    }

    fn container_dir(&self, id: &str) -> PathBuf {
        self.containers_dir.join(id)
    }

    fn load_container_state(&self, id: &str) -> Result<ContainerInfo> {
        let dir = self.container_dir(id);
        let state_path = dir.join("state.json");
        if !state_path.exists() {
            return Err(QckerError::container_not_found(id.to_string()));
        }
        let content = fs::read_to_string(&state_path)
            .map_err(|e| QckerError::internal(format!("Read state error: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| QckerError::internal(format!("Parse state error: {}", e)))
    }

    fn save_container_state(&self, container: &ContainerInfo) -> Result<()> {
        let dir = self.container_dir(&container.id);
        fs::create_dir_all(&dir)
            .map_err(|e| QckerError::internal(format!("Create dir error: {}", e)))?;
        let state_path = dir.join("state.json");
        let content = serde_json::to_string_pretty(container)
            .map_err(|e| QckerError::internal(format!("Serialize error: {}", e)))?;
        fs::write(&state_path, content)
            .map_err(|e| QckerError::internal(format!("Write error: {}", e)))?;
        Ok(())
    }

    async fn build_oci_config(&self, id: &str, spec: &ContainerSpec) -> OciConfig {
        let rootfs = self.container_dir(id).join("rootfs");
        OciConfig {
            oci_version: "1.0.0".to_string(),
            root: RootConfig { path: rootfs, readonly: false },
            process: Some(ProcessConfig {
                terminal: false,
                user: UserConfig { uid: 0, gid: 0 },
                args: spec.command.clone(),
                env: env_to_vec(&spec.env),
                cwd: "/".to_string(),
                capabilities: None,
                rlimits: vec![],
                no_new_privileges: true,
            }),
            hostname: Some(spec.hostname.clone()),
            mounts: vec![],
            linux: None,
        }
    }
}

#[async_trait]
impl RuntimeBackend for NativeBackend {
    fn backend_name(&self) -> &str {
        "native"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }

    async fn initialize(&mut self, config: &BackendConfig) -> Result<()> {
        self.containers_dir = config.containers_dir();
        fs::create_dir_all(&self.containers_dir)
            .map_err(|e| QckerError::internal(format!("Create containers dir error: {}", e)))?;
        self.config = Some(config.clone());
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.config.is_some()
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo> {
        let container_dir = self.container_dir(id);
        let rootfs = container_dir.join("rootfs");
        fs::create_dir_all(&rootfs)
            .map_err(|e| QckerError::internal(format!("Create rootfs error: {}", e)))?;
        let oci_config = self.build_oci_config(id, spec).await;
        let mut cp = ContainerProcess::new(id, &rootfs, oci_config, self.containers_dir.clone())?;
        cp.create()?;
        let info = ContainerInfo {
            id: id.to_string(),
            name: None,
            image: spec.image.clone(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            labels: spec.labels.clone(),
        };
        self.save_container_state(&info)?;
        Ok(info)
    }

    async fn start_container(&self, id: &str) -> Result<()> {
        let mut ci = self.load_container_state(id)?;
        match ci.status {
            ContainerStatus::Created => {}
            ContainerStatus::Stopped => {}
            _ => return Err(QckerError::process(format!("Invalid state: {:?}", ci.status))),
        }
        let runtime_container = ContainerProcess::load_state(&self.containers_dir, id)?;
        let oci_config = runtime_container.config.clone();
        let rootfs = runtime_container.rootfs.clone();
        let mut cp = ContainerProcess::new(id, &rootfs, oci_config, self.containers_dir.clone())?;
        cp.create()?;
        cp.start()?;
        ci.pid = cp.container.pid.map(|p| p as u32);
        ci.status = ContainerStatus::Running;
        self.save_container_state(&ci)?;
        Ok(())
    }

    async fn kill_container(&self, id: &str, signal: i32) -> Result<()> {
        let mut ci = self.load_container_state(id)?;
        if let Some(pid) = ci.pid {
            unsafe { libc::kill(pid as libc::pid_t, signal) };
        }
        ci.status = ContainerStatus::Stopped;
        self.save_container_state(&ci)?;
        Ok(())
    }

    async fn delete_container(&self, id: &str, force: bool) -> Result<()> {
        let mut ci = self.load_container_state(id)?;
        if ci.status == ContainerStatus::Running {
            if force {
                let _ = self.kill_container(id, libc::SIGKILL).await;
            } else {
                return Err(QckerError::process("Container running, use force=true".to_string()));
            }
        }
        let dir = self.container_dir(id);
        if dir.exists() { fs::remove_dir_all(&dir)?; }
        Ok(())
    }

    async fn exec_in_container(&self, id: &str, command: &[String], tty: bool, _env: &HashMap<String, String>) -> Result<ExecResult> {
        let ci = self.load_container_state(id)?;
        if ci.status != ContainerStatus::Running {
            return Err(QckerError::process("Not running".to_string()));
        }
        let pid = ci.pid.ok_or_else(|| QckerError::process("No PID".to_string()))?;
        let mut args = vec![
            "--target".to_string(), pid.to_string(),
            "--mount".into(), "--uts".into(), "--ipc".into(),
            "--net".into(), "--pid".into(), "--".into(),
        ];
        if tty { args.push("-t".into()); }
        args.extend_from_slice(command);
        let output = Command::new("nsenter").args(&args).output()
            .map_err(|e| QckerError::process(format!("nsenter failed: {}", e)))?;
        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    async fn container_stats(&self, id: &str) -> Result<ContainerStats> {
        let ci = self.load_container_state(id)?;
        let pid = ci.pid.ok_or_else(|| QckerError::process("No PID".to_string()))?;
        let stat = fs::read_to_string(format!("/proc/{}/stat", pid))
            .map_err(|e| QckerError::process(format!("Read stat failed: {}", e)))?;
        let fields: Vec<&str> = stat.trim().split_whitespace().collect();
        if fields.len() < 24 {
            return Err(QckerError::process("Invalid stat format".to_string()));
        }
        let utime: u64 = fields[22].parse().unwrap_or(0);
        let stime: u64 = fields[23].parse().unwrap_or(0);
        let jiffies = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as u64;
        let cpu = ((utime + stime) * 1_000_000_000) / jiffies;
        let status = fs::read_to_string(format!("/proc/{}/status", pid))
            .map_err(|e| QckerError::process(format!("Read status failed: {}", e)))?;
        let mut mem = 0u64;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 { mem = parts[1].parse().unwrap_or(0) * 1024; }
                break;
            }
        }
        Ok(ContainerStats {
            cpu_usage_ns: cpu,
            memory_usage_bytes: mem,
            memory_limit_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            block_read_bytes: 0,
            block_write_bytes: 0,
            pids: 0,
        })
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let mut v = Vec::new();
        if self.containers_dir.exists() {
            for entry in fs::read_dir(&self.containers_dir)
                .map_err(|e| QckerError::internal(format!("Read dir error: {}", e)))?
            {
                let entry = entry.map_err(|e| QckerError::internal(format!("Entry error: {}", e)))?;
                if entry.path().join("state.json").exists() {
                    if let Ok(c) = self.load_container_state(&entry.file_name().to_string_lossy()) {
                        v.push(c);
                    }
                }
            }
        }
        Ok(v)
    }

    async fn container_logs(&self, _id: &str, _opts: &LogReadOpts) -> Result<Vec<LogEntry>> {
        Ok(Vec::new())
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.config = None;
        Ok(())
    }

    async fn list_files(&self, id: &str, path: &str) -> Result<Vec<FileInfo>> {
        let root = self.container_dir(id).join("rootfs").join(path.trim_start_matches('/'));
        if !root.exists() {
            return Err(QckerError::process(format!("Path not found: {}", path)));
        }
        let mut files = Vec::new();
        for entry in fs::read_dir(root)
            .map_err(|e| QckerError::process(format!("Read dir error: {}", e)))?
        {
            let entry = entry.map_err(|e| QckerError::process(format!("Entry error: {}", e)))?;
            let m = entry.metadata()
                .map_err(|e| QckerError::process(format!("Metadata error: {}", e)))?;
            let name = entry.file_name().to_string_lossy().to_string();
            files.push(FileInfo {
                name: name.clone(),
                path: format!("{}/{}", path.trim_end_matches('/'), name),
                is_dir: m.is_dir(),
                size: m.len(),
                permissions: if m.permissions().readonly() { "r--" } else { "rw-" }.into(),
                modified: m.modified().map(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                }).unwrap_or_else(|_| "N/A".to_string()),
            });
        }
        files.sort_by(|a, b| {
            if a.is_dir == b.is_dir { a.name.cmp(&b.name) }
            else if a.is_dir { std::cmp::Ordering::Less }
            else { std::cmp::Ordering::Greater }
        });
        Ok(files)
    }

    async fn read_file(&self, id: &str, path: &str) -> Result<Vec<u8>> {
        let f = self.container_dir(id).join("rootfs").join(path.trim_start_matches('/'));
        if !f.exists() {
            return Err(QckerError::process(format!("File not found: {}", path)));
        }
        fs::read(&f).map_err(|e| QckerError::process(format!("Read error: {}", e)))
    }

    async fn write_file(&self, id: &str, path: &str, content: &[u8]) -> Result<()> {
        let f = self.container_dir(id).join("rootfs").join(path.trim_start_matches('/'));
        if let Some(p) = f.parent() {
            fs::create_dir_all(p)
                .map_err(|e| QckerError::process(format!("Create dir error: {}", e)))?;
        }
        fs::write(f, content).map_err(|e| QckerError::process(format!("Write error: {}", e)))
    }

    async fn delete_file(&self, id: &str, path: &str) -> Result<()> {
        let f = self.container_dir(id).join("rootfs").join(path.trim_start_matches('/'));
        if !f.exists() {
            return Err(QckerError::process(format!("File not found: {}", path)));
        }
        if f.is_dir() {
            fs::remove_dir_all(&f)
                .map_err(|e| QckerError::process(format!("Remove dir error: {}", e)))?;
        } else {
            fs::remove_file(&f)
                .map_err(|e| QckerError::process(format!("Remove file error: {}", e)))?;
        }
        Ok(())
    }

    async fn create_dir(&self, id: &str, path: &str) -> Result<()> {
        let d = self.container_dir(id).join("rootfs").join(path.trim_start_matches('/'));
        fs::create_dir_all(d)
            .map_err(|e| QckerError::process(format!("Create dir error: {}", e)))
    }

    async fn upload_file(&self, id: &str, host_path: &str, container_path: &str) -> Result<()> {
        let content = fs::read(host_path)
            .map_err(|e| QckerError::process(format!("Host read error: {}", e)))?;
        self.write_file(id, container_path, &content).await
    }

    async fn download_file(&self, id: &str, container_path: &str, host_path: &str) -> Result<()> {
        let content = self.read_file(id, container_path).await?;
        fs::write(host_path, content)
            .map_err(|e| QckerError::process(format!("Host write error: {}", e)))
    }
}
