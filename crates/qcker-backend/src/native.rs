use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::BackendConfig;
use crate::types::*;
use crate::RuntimeBackend;

pub struct NativeBackend {
    config: Option<BackendConfig>,
    containers_dir: PathBuf,
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

    fn container_state_path(&self, id: &str) -> PathBuf {
        self.container_dir(id).join("state.json")
    }

    fn load_container_state(&self, id: &str) -> Result<ContainerInfo, String> {
        let state_path = self.container_state_path(id);
        if !state_path.exists() {
            return Err(format!("Container not found: {}", id));
        }

        let content = fs::read_to_string(&state_path)
            .map_err(|e| format!("Failed to read state: {}", e))?;
        let container: ContainerInfo = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse state: {}", e))?;
        Ok(container)
    }

    fn save_container_state(&self, container: &ContainerInfo) -> Result<(), String> {
        let dir = self.container_dir(&container.id);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create container dir: {}", e))?;

        let state_path = dir.join("state.json");
        let content = serde_json::to_string_pretty(container)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        fs::write(&state_path, content)
            .map_err(|e| format!("Failed to write state: {}", e))?;
        Ok(())
    }

    fn delete_container_state(&self, id: &str) -> Result<(), String> {
        let dir = self.container_dir(id);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .map_err(|e| format!("Failed to remove container dir: {}", e))?;
        }
        Ok(())
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

    async fn initialize(&mut self, config: &BackendConfig) -> Result<(), String> {
        self.containers_dir = config.containers_dir();
        fs::create_dir_all(&self.containers_dir)
            .map_err(|e| format!("Failed to create containers dir: {}", e))?;
        self.config = Some(config.clone());
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.config.is_some()
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo, String> {
        let container = ContainerInfo {
            id: id.to_string(),
            name: None,
            image: spec.image.clone(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            labels: spec.labels.clone(),
        };

        self.save_container_state(&container)?;
        Ok(container)
    }

    async fn start_container(&self, id: &str) -> Result<(), String> {
        let mut container = self.load_container_state(id)?;

        match container.status {
            ContainerStatus::Created => {}
            ContainerStatus::Stopped => {}
            _ => return Err(format!("Container is in state {:?}, cannot start", container.status)),
        }

        container.status = ContainerStatus::Running;
        self.save_container_state(&container)?;

        Ok(())
    }

    async fn kill_container(&self, id: &str, signal: i32) -> Result<(), String> {
        let container = self.load_container_state(id)?;

        if let Some(pid) = container.pid {
            unsafe {
                let ret = libc::kill(pid as i32, signal);
                if ret != 0 {
                    let err = std::io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::ESRCH) {
                        return Err(format!("Failed to send signal: {}", err));
                    }
                }
            }
        }

        Ok(())
    }

    async fn delete_container(&self, id: &str, force: bool) -> Result<(), String> {
        let container = self.load_container_state(id)?;

        if container.status == ContainerStatus::Running {
            if force {
                let _ = self.kill_container(id, libc::SIGKILL).await;
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                return Err("Container is running. Use force=true to delete".to_string());
            }
        }

        self.delete_container_state(id)?;
        Ok(())
    }

    async fn exec_in_container(&self, id: &str, command: &[String], _tty: bool, _env: &HashMap<String, String>) -> Result<ExecResult, String> {
        let container = self.load_container_state(id)?;

        if container.status != ContainerStatus::Running {
            return Err("Container is not running".to_string());
        }

        let output = Command::new(&command[0])
            .args(&command[1..])
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    async fn container_stats(&self, id: &str) -> Result<ContainerStats, String> {
        let _container = self.load_container_state(id)?;

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

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        let mut containers = Vec::new();

        if !self.containers_dir.exists() {
            return Ok(containers);
        }

        for entry in fs::read_dir(&self.containers_dir)
            .map_err(|e| format!("Failed to read containers dir: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let state_path = entry.path().join("state.json");

            if state_path.exists() {
                if let Ok(container) = self.load_container_state(
                    &entry.file_name().to_string_lossy()
                ) {
                    containers.push(container);
                }
            }
        }

        Ok(containers)
    }

    async fn container_logs(&self, id: &str, _opts: &LogReadOpts) -> Result<Vec<LogEntry>, String> {
        let _container = self.load_container_state(id)?;
        Ok(Vec::new())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        self.config = None;
        Ok(())
    }

    async fn list_files(&self, id: &str, path: &str) -> Result<Vec<FileInfo>, String> {
        let container = self.load_container_state(id)?;
        let container_root = self.container_dir(id).join("rootfs");
        let target_path = container_root.join(path.trim_start_matches('/'));

        if !target_path.exists() {
            return Err(format!("Path not found: {}", path));
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&target_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let metadata = entry.metadata()
                .map_err(|e| format!("Failed to get metadata: {}", e))?;

            let name = entry.file_name().to_string_lossy().to_string();
            let file_path = format!("{}/{}", path.trim_end_matches('/'), name);
            let permissions = if metadata.permissions().readonly() {
                "r--".to_string()
            } else {
                "rw-".to_string()
            };
            let modified = metadata.modified()
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "N/A".to_string());

            files.push(FileInfo {
                name,
                path: file_path,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                permissions,
                modified,
            });
        }

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
        let container_root = self.container_dir(id).join("rootfs");
        let file_path = container_root.join(path.trim_start_matches('/'));

        if !file_path.exists() {
            return Err(format!("File not found: {}", path));
        }

        fs::read(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))
    }

    async fn write_file(&self, id: &str, path: &str, content: &[u8]) -> Result<(), String> {
        let container_root = self.container_dir(id).join("rootfs");
        let file_path = container_root.join(path.trim_start_matches('/'));

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))
    }

    async fn delete_file(&self, id: &str, path: &str) -> Result<(), String> {
        let container_root = self.container_dir(id).join("rootfs");
        let file_path = container_root.join(path.trim_start_matches('/'));

        if !file_path.exists() {
            return Err(format!("File not found: {}", path));
        }

        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)
                .map_err(|e| format!("Failed to remove directory: {}", e))?;
        } else {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to remove file: {}", e))?;
        }

        Ok(())
    }

    async fn create_dir(&self, id: &str, path: &str) -> Result<(), String> {
        let container_root = self.container_dir(id).join("rootfs");
        let dir_path = container_root.join(path.trim_start_matches('/'));

        fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create directory: {}", e))
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
