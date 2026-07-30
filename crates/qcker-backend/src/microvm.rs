use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::config::BackendConfig;
use crate::kernel::KernelManager;
use crate::port_forward::PortForwarder;
use crate::rootfs::RootfsManager;
use crate::types::*;
use crate::vmm::{self, VmmConfig, VmmManager};
use crate::RuntimeBackend;

pub struct MicroVmBackend {
    state: Arc<Mutex<MicroVmState>>,
}

struct MicroVmState {
    config: Option<BackendConfig>,
    vmm: Option<VmmManager>,
    port_forwarder: PortForwarder,
    status: BackendStatus,
}

#[derive(Debug, PartialEq)]
enum BackendStatus {
    NotStarted,
    Booting,
    Running,
    ShuttingDown,
}

impl MicroVmBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MicroVmState {
                config: None,
                vmm: None,
                port_forwarder: PortForwarder::new(),
                status: BackendStatus::NotStarted,
            })),
        }
    }
}

impl Default for MicroVmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuntimeBackend for MicroVmBackend {
    fn backend_name(&self) -> &str {
        "microvm"
    }

    fn is_available(&self) -> bool {
        vmm::check_qemu_available()
    }

    async fn initialize(&mut self, config: &BackendConfig) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.config = Some(config.clone());
        state.status = BackendStatus::NotStarted;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.status == BackendStatus::Running)
            .unwrap_or(false)
    }

    async fn create_container(&self, id: &str, spec: &ContainerSpec) -> Result<ContainerInfo, String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;

        if state.status != BackendStatus::Running {
            return Err("VM is not running".to_string());
        }

        let container = ContainerInfo {
            id: id.to_string(),
            name: None,
            image: spec.command.first().cloned().unwrap_or_default(),
            status: ContainerStatus::Created,
            pid: None,
            exit_code: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            labels: HashMap::new(),
        };

        Ok(container)
    }

    async fn start_container(&self, _id: &str) -> Result<(), String> {
        let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        if state.status != BackendStatus::Running {
            return Err("VM is not running".to_string());
        }
        Ok(())
    }

    async fn kill_container(&self, _id: &str, _signal: i32) -> Result<(), String> {
        Ok(())
    }

    async fn delete_container(&self, _id: &str, _force: bool) -> Result<(), String> {
        Ok(())
    }

    async fn exec_in_container(
        &self,
        _id: &str,
        _command: &[String],
        _tty: bool,
        _env: &HashMap<String, String>,
    ) -> Result<ExecResult, String> {
        Err("exec not yet implemented for MicroVM backend".to_string())
    }

    async fn container_stats(&self, _id: &str) -> Result<ContainerStats, String> {
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
        Ok(Vec::new())
    }

    async fn container_logs(&self, _id: &str, _opts: &LogReadOpts) -> Result<Vec<LogEntry>, String> {
        Ok(Vec::new())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.status = BackendStatus::ShuttingDown;

        if let Some(mut vmm) = state.vmm.take() {
            let _ = vmm.stop();
        }

        state.port_forwarder.clear();
        state.status = BackendStatus::NotStarted;

        Ok(())
    }

    async fn list_files(&self, _id: &str, _path: &str) -> Result<Vec<FileInfo>, String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn read_file(&self, _id: &str, _path: &str) -> Result<Vec<u8>, String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn write_file(&self, _id: &str, _path: &str, _content: &[u8]) -> Result<(), String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn delete_file(&self, _id: &str, _path: &str) -> Result<(), String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn create_dir(&self, _id: &str, _path: &str) -> Result<(), String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn upload_file(&self, _id: &str, _host_path: &str, _container_path: &str) -> Result<(), String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }

    async fn download_file(&self, _id: &str, _container_path: &str, _host_path: &str) -> Result<(), String> {
        Err("File operations not yet implemented for MicroVM backend".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microvm_backend_new() {
        let backend = MicroVmBackend::new();
        assert_eq!(backend.backend_name(), "microvm");
        assert!(!backend.is_running());
    }

    #[test]
    fn test_microvm_backend_default() {
        let backend = MicroVmBackend::default();
        assert_eq!(backend.backend_name(), "microvm");
    }
}
