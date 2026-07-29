use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum HostToVm {
    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "container.create")]
    ContainerCreate {
        id: String,
        spec: ContainerSpec,
    },

    #[serde(rename = "container.start")]
    ContainerStart {
        id: String,
    },

    #[serde(rename = "container.kill")]
    ContainerKill {
        id: String,
        signal: i32,
    },

    #[serde(rename = "container.delete")]
    ContainerDelete {
        id: String,
        force: bool,
    },

    #[serde(rename = "container.exec")]
    ContainerExec {
        id: String,
        command: Vec<String>,
        tty: bool,
        interactive: bool,
        env: HashMap<String, String>,
    },

    #[serde(rename = "container.stats")]
    ContainerStats {
        id: String,
    },

    #[serde(rename = "vm.shutdown")]
    VmShutdown,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum VmToHost {
    #[serde(rename = "vm.ready")]
    VmReady {
        version: String,
        arch: String,
    },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "container.created")]
    ContainerCreated {
        id: String,
        pid: u32,
    },

    #[serde(rename = "container.started")]
    ContainerStarted {
        id: String,
    },

    #[serde(rename = "container.exited")]
    ContainerExited {
        id: String,
        exit_code: i32,
        timestamp: String,
    },

    #[serde(rename = "container.stats")]
    ContainerStatsResponse {
        id: String,
        stats: ContainerStats,
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
    let json = serde_json::to_vec(msg)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;
    let len = (json.len() as u32).to_be_bytes();
    let mut result = Vec::with_capacity(4 + json.len());
    result.extend_from_slice(&len);
    result.extend_from_slice(&json);
    Ok(result)
}

pub fn deserialize_message<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T, String> {
    serde_json::from_slice(data)
        .map_err(|e| format!("Failed to deserialize message: {}", e))
}

pub fn parse_length_prefix(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() >= 4 + len {
        Some((4, len))
    } else {
        None
    }
}
