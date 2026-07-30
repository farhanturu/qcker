use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const QCKER_VSOCK_PORT: u32 = 7421;
pub const VMADDR_CID_HOST: u32 = 2;
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum HostToVm {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "container.create")]
    ContainerCreate { id: String, spec: ContainerSpec },
    #[serde(rename = "container.start")]
    ContainerStart { id: String },
    #[serde(rename = "container.kill")]
    ContainerKill { id: String, signal: i32 },
    #[serde(rename = "container.delete")]
    ContainerDelete { id: String, force: bool },
    #[serde(rename = "container.exec")]
    ContainerExec {
        id: String,
        command: Vec<String>,
        tty: bool,
        interactive: bool,
        env: HashMap<String, String>,
    },
    #[serde(rename = "container.attach")]
    ContainerAttach { id: String },
    #[serde(rename = "container.stats")]
    ContainerStats { id: String },
    #[serde(rename = "container.resize")]
    ContainerResize { id: String, width: u16, height: u16 },
    #[serde(rename = "vm.shutdown")]
    VmShutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum VmToHost {
    #[serde(rename = "vm.ready")]
    VmReady { version: String, arch: String },
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "container.created")]
    ContainerCreated { id: String, pid: u32 },
    #[serde(rename = "container.started")]
    ContainerStarted { id: String },
    #[serde(rename = "container.exited")]
    ContainerExited {
        id: String,
        exit_code: i32,
        timestamp: String,
    },
    #[serde(rename = "container.stats")]
    ContainerStatsResponse {
        id: String,
        cpu_usage_ns: u64,
        memory_usage_bytes: u64,
        memory_limit_bytes: u64,
        network_rx_bytes: u64,
        network_tx_bytes: u64,
        pids: u64,
    },
    #[serde(rename = "container.log")]
    ContainerLog {
        id: String,
        stream: LogStream,
        data: String,
        timestamp: String,
    },
    #[serde(rename = "error")]
    Error {
        request_type: String,
        message: String,
        code: String,
    },
    #[serde(rename = "vm.shutdown_complete")]
    VmShutdownComplete,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogStream {
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "stderr")]
    Stderr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerSpec {
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

impl Default for ContainerSpec {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            command: vec!["/bin/sh".to_string()],
            env: HashMap::new(),
            working_dir: Some("/".to_string()),
            user: Some("root".to_string()),
            group: None,
            mounts: Vec::new(),
            port_forwards: Vec::new(),
            resources: ResourceSpec::default(),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MountSpec {
    pub source: String,
    pub target: String,
    pub mount_type: MountType,
    pub read_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MountType {
    #[serde(rename = "bind")]
    Bind,
    #[serde(rename = "volume")]
    Volume,
    #[serde(rename = "tmpfs")]
    Tmpfs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortForwardSpec {
    pub host_port: u16,
    pub guest_port: u16,
    pub protocol: Protocol,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Protocol {
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResourceSpec {
    pub cpu_shares: Option<u64>,
    pub memory_mb: Option<u64>,
    pub pids_limit: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMode {
    #[serde(rename = "bridge")]
    Bridge,
    #[serde(rename = "host")]
    Host,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "custom")]
    Custom(String),
}

pub fn serialize_message<T: Serialize>(msg: &T) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(msg).map_err(|e| format!("Serialize failed: {}", e))?;
    if json.len() > MAX_MESSAGE_SIZE {
        return Err(format!("Message too large: {} bytes", json.len()));
    }
    let len = (json.len() as u32).to_be_bytes();
    let mut result = Vec::with_capacity(4 + json.len());
    result.extend_from_slice(&len);
    result.extend_from_slice(&json);
    Ok(result)
}

pub fn deserialize_message<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T, String> {
    serde_json::from_slice(data).map_err(|e| format!("Deserialize failed: {}", e))
}

pub fn parse_length_prefix(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if len > MAX_MESSAGE_SIZE {
        return None;
    }
    if data.len() >= 4 + len {
        Some((4, len))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_ping() {
        let msg = HostToVm::Ping;
        let data = serialize_message(&msg).unwrap();
        assert!(data.len() > 4);
        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(len as usize, data.len() - 4);
    }

    #[test]
    fn test_roundtrip_container_create() {
        let spec = ContainerSpec::default();
        let msg = HostToVm::ContainerCreate {
            id: "test123".to_string(),
            spec,
        };
        let data = serialize_message(&msg).unwrap();
        let parsed: HostToVm = deserialize_message(&data[4..]).unwrap();
        match parsed {
            HostToVm::ContainerCreate { id, .. } => assert_eq!(id, "test123"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_vm_ready() {
        let msg = VmToHost::VmReady {
            version: "0.1.0".to_string(),
            arch: "x86_64".to_string(),
        };
        let data = serialize_message(&msg).unwrap();
        let parsed: VmToHost = deserialize_message(&data[4..]).unwrap();
        match parsed {
            VmToHost::VmReady { version, arch } => {
                assert_eq!(version, "0.1.0");
                assert_eq!(arch, "x86_64");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_too_large() {
        let huge_spec = ContainerSpec {
            layers: vec!["x".repeat(MAX_MESSAGE_SIZE); 2],
            ..Default::default()
        };
        let msg = HostToVm::ContainerCreate {
            id: "test".to_string(),
            spec: huge_spec,
        };
        let result = serialize_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_length_prefix() {
        let data = [0, 0, 0, 5, 1, 2, 3, 4, 5];
        let result = parse_length_prefix(&data);
        assert_eq!(result, Some((4, 5)));
    }

    #[test]
    fn test_parse_length_prefix_too_short() {
        let data = [0, 0, 0];
        assert!(parse_length_prefix(&data).is_none());
    }
}
