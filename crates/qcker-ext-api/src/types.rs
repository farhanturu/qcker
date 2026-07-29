use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extension capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionCapability {
    NetworkAccess,
    FileSystemAccess,
    ContainerLifecycle,
    ImageAccess,
    Privileged,
    SystemInfo,
    RegistryAccess,
}

/// Extension metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<ExtensionCapability>,
}

/// Extension status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionStatus {
    Loaded,
    Active,
    Error(String),
    Disabled,
}

/// Extension info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub metadata: ExtensionMetadata,
    pub status: ExtensionStatus,
    pub path: String,
}

/// IPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// IPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<IpcError>,
}

/// IPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// IPC event (host to extension)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// Container info for extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub pid: Option<i32>,
    pub networks: Vec<String>,
    pub ports: Vec<PortMapping>,
}

/// Port mapping info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

/// Image info for extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub created_at: String,
}

/// Extension configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

impl IpcRequest {
    /// Create a new IPC request
    pub fn new(id: u64, method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }
}

impl IpcResponse {
    /// Create a success response
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: u64, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(IpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

impl IpcEvent {
    /// Create a new IPC event
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_request() {
        let req = IpcRequest::new(1, "container.list", serde_json::json!({}));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.id, 1);
        assert_eq!(req.method, "container.list");
    }

    #[test]
    fn test_ipc_response() {
        let resp = IpcResponse::success(1, serde_json::json!({"containers": []}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());

        let resp = IpcResponse::error(2, -32600, "Invalid Request");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
