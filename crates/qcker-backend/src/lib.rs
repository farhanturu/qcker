pub mod config;
pub mod fs_share;
pub mod kernel;
pub mod microvm;
pub mod native;
pub mod port_forward;
pub mod protocol;
pub mod rootfs;
pub mod selector;
pub mod types;
pub mod vmm;
pub mod vsock;

use async_trait::async_trait;
use std::collections::HashMap;

use config::BackendConfig;
use qcker_common::error::Result;
use types::*;

#[async_trait]
pub trait RuntimeBackend: Send + Sync {
    fn backend_name(&self) -> &str;
    fn is_available(&self) -> bool;
    async fn initialize(&mut self, config: &BackendConfig) -> Result<()>;
    fn is_running(&self) -> bool;
    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo>;
    async fn start_container(&self, id: &str) -> Result<()>;
    async fn kill_container(&self, id: &str, signal: i32) -> Result<()>;
    async fn delete_container(&self, id: &str, force: bool) -> Result<()>;
    async fn exec_in_container(&self, id: &str, command: &[String], tty: bool, env: &HashMap<String, String>) -> Result<ExecResult>;
    async fn container_stats(&self, id: &str) -> Result<ContainerStats>;
    async fn list_containers(&self) -> Result<Vec<ContainerInfo>>;
    async fn container_logs(&self, id: &str, opts: &LogReadOpts) -> Result<Vec<LogEntry>>;
    async fn shutdown(&mut self) -> Result<()>;

    async fn list_files(&self, id: &str, path: &str) -> Result<Vec<FileInfo>>;
    async fn read_file(&self, id: &str, path: &str) -> Result<Vec<u8>>;
    async fn write_file(&self, id: &str, path: &str, content: &[u8]) -> Result<()>;
    async fn delete_file(&self, id: &str, path: &str) -> Result<()>;
    async fn create_dir(&self, id: &str, path: &str) -> Result<()>;
    async fn upload_file(&self, id: &str, host_path: &str, container_path: &str) -> Result<()>;
    async fn download_file(&self, id: &str, container_path: &str, host_path: &str) -> Result<()>;
}
