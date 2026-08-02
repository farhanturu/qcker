use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use qcker_runtime::cgroup;
use qcker_runtime::process::ContainerProcess;
use qcker_runtime::spec::{
    LinuxConfig, NamespaceConfig, NamespaceType, OciConfig, ProcessConfig,
    RootConfig, UserConfig,
};

use crate::config::BackendConfig;
use crate::types::*;
use crate::RuntimeBackend;

pub struct NativeBackend {
    config: Option<BackendConfig>,
    containers_dir: PathBuf,
    data_dir: PathBuf,
    running_pids: Arc<Mutex<HashMap<String, u32>>>,
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
            data_dir: PathBuf::new(),
            running_pids: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn container_dir(&self, id: &str) -> PathBuf {
        self.containers_dir.join(id)
    }

    fn container_state_path(&self, id: &str) -> PathBuf {
        self.container_dir(id).join("state.json")
    }

    fn container_log_path(&self, id: &str) -> PathBuf {
        self.container_dir(id).join("container.log")
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

    fn build_oci_config(&self, container: &ContainerInfo, spec_path: &PathBuf) -> OciConfig {
        let spec_content = fs::read_to_string(spec_path).ok();
        let spec: Option<ContainerSpec> = spec_content
            .and_then(|c| serde_json::from_str(&c).ok());

        let _rootfs_path = container.rootfs_path.clone().unwrap_or_else(|| {
            self.container_dir(&container.id)
                .join("rootfs")
                .to_str()
                .unwrap_or("rootfs")
                .to_string()
        });

        let (args, env_vec, cwd) = if let Some(ref s) = spec {
            let env_vec: Vec<String> = s
                .env
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            (
                if s.command.is_empty() {
                    vec!["/bin/sh".to_string()]
                } else {
                    s.command.clone()
                },
                env_vec,
                s.working_dir.clone().unwrap_or_else(|| "/".to_string()),
            )
        } else {
            (vec!["/bin/sh".to_string()], Vec::new(), "/".to_string())
        };

        let namespaces = vec![
            NamespaceConfig {
                r#type: NamespaceType::Pid,
                path: None,
            },
            NamespaceConfig {
                r#type: NamespaceType::Mount,
                path: None,
            },
            NamespaceConfig {
                r#type: NamespaceType::Uts,
                path: None,
            },
            NamespaceConfig {
                r#type: NamespaceType::Ipc,
                path: None,
            },
            NamespaceConfig {
                r#type: NamespaceType::Network,
                path: None,
            },
        ];

        OciConfig {
            oci_version: "1.0.0".to_string(),
            root: RootConfig {
                path: PathBuf::from("rootfs"),
                readonly: false,
            },
            process: Some(ProcessConfig {
                terminal: false,
                user: UserConfig { uid: 0, gid: 0 },
                args,
                env: env_vec,
                cwd,
                capabilities: None,
                rlimits: vec![],
                no_new_privileges: false,
            }),
            hostname: Some(container.id.chars().take(12).collect()),
            mounts: vec![],
            linux: Some(LinuxConfig {
                namespaces,
                resources: None,
                uid_mappings: vec![],
                gid_mappings: vec![],
                seccomp: None,
            }),
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

    async fn initialize(&mut self, config: &BackendConfig) -> Result<(), String> {
        self.containers_dir = config.containers_dir();
        self.data_dir = config.data_dir.clone();
        fs::create_dir_all(&self.containers_dir)
            .map_err(|e| format!("Failed to create containers dir: {}", e))?;
        self.config = Some(config.clone());
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.config.is_some()
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo, String> {
        let container_dir = self.container_dir(id);
        fs::create_dir_all(&container_dir)
            .map_err(|e| format!("Failed to create container dir: {}", e))?;

        let spec_path = container_dir.join("spec.json");
        let spec_content = serde_json::to_string_pretty(spec)
            .map_err(|e| format!("Failed to serialize spec: {}", e))?;
        fs::write(&spec_path, spec_content)
            .map_err(|e| format!("Failed to write spec: {}", e))?;

        let rootfs_path = spec.rootfs_path.clone().or_else(|| {
            container_dir.join("rootfs").to_str().map(|s| s.to_string())
        });

        let log_path = self.container_log_path(id).to_str().map(|s| s.to_string());

        let container = ContainerInfo {
            id: id.to_string(),
            name: None,
            image: spec.image.clone(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            labels: spec.labels.clone(),
            rootfs_path,
            log_path,
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

        let container_dir = self.container_dir(id);
        let spec_path = container_dir.join("spec.json");
        let oci_config = self.build_oci_config(&container, &spec_path);

        let rootfs_path = container.rootfs_path.clone().unwrap_or_else(|| {
            container_dir.join("rootfs").to_str().unwrap_or("rootfs").to_string()
        });
        let rootfs = PathBuf::from(&rootfs_path);

        if !rootfs.exists() {
            return Err(format!("Container rootfs not found: {}", rootfs.display()));
        }

        let bundle = rootfs.parent().unwrap_or(&rootfs).to_path_buf();
        let log_path = self.container_log_path(id);
        let data_dir = self.data_dir.clone();
        let id_owned = id.to_string();
        let _containers_dir = self.containers_dir.clone();
        let running_pids = self.running_pids.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut proc = ContainerProcess::new(&id_owned, &bundle, oci_config, data_dir)
                .map_err(|e| format!("Failed to create container process: {}", e))?;

            proc.set_log_path(log_path);

            proc.start()
                .map_err(|e| format!("Failed to start container process: {}", e))?;

            let pid = proc.container.pid.unwrap_or(0) as u32;

            if let Ok(mut pids) = running_pids.lock() {
                pids.insert(id_owned.clone(), pid);
            }

            if let Ok(cgroup_path) = cgroup::create_cgroup(&id_owned) {
                let _ = cgroup::add_process(&cgroup_path, pid as i32);
            }

            Ok::<(u32, String), String>((pid, id_owned))
        })
        .await
        .map_err(|e| format!("Spawn blocking task failed: {}", e))??;

        container.pid = Some(result.0);
        container.status = ContainerStatus::Running;
        self.save_container_state(&container)?;

        let monitor_id = id.to_string();
        let monitor_containers_dir = self.containers_dir.clone();
        let monitor_running_pids = self.running_pids.clone();

        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                let pid = result.0 as i32;
                loop {
                    let check = unsafe { libc::kill(pid, 0) };
                    if check != 0 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                let state_path = monitor_containers_dir
                    .join(&monitor_id)
                    .join("state.json");
                if state_path.exists() {
                    if let Ok(content) = fs::read_to_string(&state_path) {
                        if let Ok(mut info) = serde_json::from_str::<ContainerInfo>(&content) {
                            info.status = ContainerStatus::Stopped;
                            info.exit_code = Some(0);
                            if let Ok(updated) = serde_json::to_string_pretty(&info) {
                                let _ = fs::write(&state_path, updated);
                            }
                        }
                    }
                }

                if let Ok(mut pids) = monitor_running_pids.lock() {
                    pids.remove(&monitor_id);
                }
            })
            .await
        });

        Ok(())
    }

    async fn kill_container(&self, id: &str, signal: i32) -> Result<(), String> {
        let mut container = self.load_container_state(id)?;

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

        container.status = ContainerStatus::Stopped;
        if signal == libc::SIGKILL || signal == libc::SIGTERM {
            container.exit_code = Some(128 + signal);
        }
        self.save_container_state(&container)?;

        if let Ok(mut pids) = self.running_pids.lock() {
            pids.remove(id);
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

        if let Ok(cgroup_path) = cgroup::create_cgroup(id) {
            let _ = cgroup::remove_cgroup(&cgroup_path);
        }

        self.delete_container_state(id)?;
        Ok(())
    }

    async fn exec_in_container(&self, id: &str, command: &[String], _tty: bool, _env: &HashMap<String, String>) -> Result<ExecResult, String> {
        let container = self.load_container_state(id)?;

        if container.status != ContainerStatus::Running {
            return Err("Container is not running".to_string());
        }

        let container_dir = self.containers_dir.join(id);
        let rootfs = container_dir.join("rootfs");

        if !rootfs.exists() {
            return Err(format!("Container rootfs not found: {}", rootfs.display()));
        }

        let mut nsenter_args = vec![
            "--target".to_string(),
            container.pid.unwrap_or(0).to_string(),
            "--mount".to_string(),
            "--uts".to_string(),
            "--ipc".to_string(),
            "--net".to_string(),
            "--pid".to_string(),
            "--".to_string(),
        ];
        nsenter_args.extend_from_slice(command);

        let output = Command::new("nsenter")
            .args(&nsenter_args)
            .output()
            .map_err(|e| format!("Failed to exec in container: {}", e))?;

        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    async fn container_stats(&self, id: &str) -> Result<ContainerStats, String> {
        let _container = self.load_container_state(id)?;

        let cgroup_path = std::path::Path::new("/sys/fs/cgroup/qcker").join(id);
        if cgroup_path.exists() {
            if let Ok(cgroup_stats) = cgroup::get_stats(&cgroup_path) {
                return Ok(ContainerStats {
                    cpu_usage_ns: cgroup_stats.cpu_usage_usec * 1000,
                    memory_usage_bytes: cgroup_stats.memory_current,
                    memory_limit_bytes: cgroup_stats.memory_limit,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    block_read_bytes: 0,
                    block_write_bytes: 0,
                    pids: cgroup_stats.pids_current as u64,
                });
            }
        }

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

    async fn container_logs(&self, id: &str, opts: &LogReadOpts) -> Result<Vec<LogEntry>, String> {
        let container = self.load_container_state(id)?;

        let log_path = container.log_path.as_deref().map(PathBuf::from)
            .unwrap_or_else(|| self.container_log_path(id));

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_path)
            .map_err(|e| format!("Failed to read log file: {}", e))?;

        let mut entries: Vec<LogEntry> = Vec::new();

        for line in content.lines() {
            let entry = LogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                stream: LogStream::Stdout,
                message: line.to_string(),
            };
            entries.push(entry);
        }

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
        self.config = None;
        Ok(())
    }

    async fn list_files(&self, id: &str, path: &str) -> Result<Vec<FileInfo>, String> {
        let _container = self.load_container_state(id)?;
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
