use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerState {
    Creating,
    Created,
    Running,
    Stopped,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    pub oci_version: String,
    pub root: RootConfig,
    pub process: Option<ProcessConfig>,
    pub hostname: Option<String>,
    pub mounts: Vec<MountConfig>,
    pub linux: Option<LinuxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub path: PathBuf,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub terminal: bool,
    pub user: UserConfig,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub cwd: String,
    pub capabilities: Option<CapabilitiesConfig>,
    pub rlimits: Vec<RlimitConfig>,
    pub no_new_privileges: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    pub bounding: Vec<String>,
    pub effective: Vec<String>,
    pub inheritable: Vec<String>,
    pub permitted: Vec<String>,
    pub ambient: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlimitConfig {
    pub r#type: String,
    pub hard: u64,
    pub soft: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub destination: PathBuf,
    pub source: Option<PathBuf>,
    pub r#type: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfig {
    pub namespaces: Vec<NamespaceConfig>,
    pub resources: Option<ResourcesConfig>,
    pub uid_mappings: Vec<IdMapping>,
    pub gid_mappings: Vec<IdMapping>,
    pub seccomp: Option<SeccompConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    pub r#type: NamespaceType,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NamespaceType {
    Pid,
    Network,
    Mount,
    Uts,
    Ipc,
    User,
    Cgroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdMapping {
    pub container_id: u32,
    pub host_id: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesConfig {
    pub memory: Option<MemoryConfig>,
    pub cpu: Option<CpuConfig>,
    pub pids: Option<PidsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub limit: Option<i64>,
    pub reservation: Option<i64>,
    pub swap: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    pub shares: Option<u64>,
    pub quota: Option<i64>,
    pub period: Option<u64>,
    pub cpus: Option<String>,
    pub mems: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsConfig {
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompConfig {
    pub default_action: String,
    pub architectures: Vec<String>,
    pub syscalls: Vec<SeccompSyscall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompSyscall {
    pub names: Vec<String>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub state: ContainerState,
    pub bundle: PathBuf,
    pub pid: Option<i32>,
    pub rootfs: PathBuf,
    pub config: OciConfig,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciContainerState {
    pub oci_version: String,
    pub id: String,
    pub status: String,
    pub pid: Option<i32>,
    pub bundle: PathBuf,
    pub annotations: std::collections::HashMap<String, String>,
}

impl Container {
    pub fn new(id: String, bundle: PathBuf, config: OciConfig) -> Self {
        let rootfs = bundle.join(&config.root.path);
        Self {
            id,
            state: ContainerState::Created,
            bundle,
            pid: None,
            rootfs,
            config,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn oci_state(&self) -> OciContainerState {
        OciContainerState {
            oci_version: self.config.oci_version.clone(),
            id: self.id.clone(),
            status: match self.state {
                ContainerState::Creating => "creating".to_string(),
                ContainerState::Created => "created".to_string(),
                ContainerState::Running => "running".to_string(),
                ContainerState::Stopped => "stopped".to_string(),
                ContainerState::Paused => "paused".to_string(),
            },
            pid: self.pid,
            bundle: self.bundle.clone(),
            annotations: std::collections::HashMap::new(),
        }
    }
}
