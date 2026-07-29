use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub image: String,
    pub layers: Vec<String>,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub mounts: Vec<MountSpec>,
    pub port_forwards: Vec<PortForwardSpec>,
    pub resources: ResourceSpec,
    pub rootful: bool,
    pub seccomp_profile: Option<String>,
    pub capabilities: Vec<String>,
    pub network_mode: NetworkMode,
    pub hostname: String,
    pub dns: Vec<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSpec {
    pub source: String,
    pub target: String,
    pub mount_type: MountType,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountType {
    Bind,
    Volume,
    Tmpfs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardSpec {
    pub host_port: u16,
    pub guest_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_shares: Option<u64>,
    pub memory_mb: Option<u64>,
    pub pids_limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: Option<String>,
    pub image: String,
    pub status: ContainerStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Paused,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_usage_ns: u64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub pids: u64,
}

#[derive(Debug)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct LogReadOpts {
    pub follow: bool,
    pub tail: Option<usize>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub timestamps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub stream: LogStream,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: String,
}

impl Default for ContainerSpec {
    fn default() -> Self {
        Self {
            image: String::new(),
            layers: Vec::new(),
            command: vec!["/bin/sh".to_string()],
            env: HashMap::new(),
            working_dir: Some("/".to_string()),
            user: Some("root".to_string()),
            group: None,
            mounts: Vec::new(),
            port_forwards: Vec::new(),
            resources: ResourceSpec {
                cpu_shares: None,
                memory_mb: None,
                pids_limit: None,
            },
            rootful: false,
            seccomp_profile: None,
            capabilities: Vec::new(),
            network_mode: NetworkMode::Bridge,
            hostname: "container".to_string(),
            dns: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            labels: HashMap::new(),
        }
    }
}
