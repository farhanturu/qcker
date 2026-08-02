use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{ContainerSpec, LogStream};

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

