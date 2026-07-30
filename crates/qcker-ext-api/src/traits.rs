use super::types::*;
use std::collections::HashMap;

pub trait Extension: Send + Sync {
    fn id(&self) -> &str;

    fn version(&self) -> &str;

    fn name(&self) -> &str;

    fn description(&self) -> &str;

    fn author(&self) -> &str;

    fn api_version(&self) -> &str;

    fn capabilities(&self) -> Vec<ExtensionCapability>;

    fn on_load(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), String> {
        Ok(())
    }
}

pub trait NetworkDriver: Extension {
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    fn create_network(&self, name: &str, config: &HashMap<String, String>) -> Result<String, String>;

    fn delete_network(&self, network_id: &str) -> Result<(), String>;

    fn connect(&self, network_id: &str, container_id: &str, config: &HashMap<String, String>) -> Result<(), String>;

    fn disconnect(&self, network_id: &str, container_id: &str) -> Result<(), String>;

    fn list_networks(&self) -> Result<Vec<NetworkInfo>, String>;

    fn driver_name(&self) -> &str;
}

pub trait StorageDriver: Extension {
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    fn prepare(&self, key: &str, parent: &str) -> Result<String, String>;

    fn commit(&self, key: &str, name: &str) -> Result<(), String>;

    fn remove(&self, key: &str) -> Result<(), String>;

    fn driver_name(&self) -> &str;
}

pub trait SecurityScanner: Extension {
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    fn scan_image(&self, image_id: &str, config: &HashMap<String, String>) -> Result<ScanResult, String>;

    fn scanner_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: ScanSummary,
}

#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub severity: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

pub trait LogDriver: Extension {
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    fn start_capture(&self, container_id: &str, config: &HashMap<String, String>) -> Result<String, String>;

    fn stop_capture(&self, handle: &str) -> Result<(), String>;

    fn read_logs(&self, container_id: &str, config: &HashMap<String, String>) -> Result<Vec<LogEntry>, String>;

    fn driver_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub stream: String,
    pub message: String,
}

pub trait BuildStrategy: Extension {
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    fn build(&self, context_path: &str, config: &HashMap<String, String>) -> Result<String, String>;

    fn strategy_name(&self) -> &str;
}

pub trait CommandExtension: Extension {
    fn command_name(&self) -> &str;

    fn help(&self) -> &str;

    fn execute(&self, args: &[String]) -> Result<(), String>;
}

pub trait HookExtension: Extension {
    fn pre_start(&self, container: &ContainerInfo) -> Result<HookDecision, String>;

    fn post_start(&self, container: &ContainerInfo) -> Result<(), String>;

    fn pre_stop(&self, container: &ContainerInfo) -> Result<HookDecision, String>;

    fn post_stop(&self, container: &ContainerInfo) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub enum HookDecision {
    Allow,
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtension;

    impl Extension for TestExtension {
        fn id(&self) -> &str { "com.test.ext" }
        fn version(&self) -> &str { "1.0.0" }
        fn name(&self) -> &str { "Test Extension" }
        fn description(&self) -> &str { "A test extension" }
        fn author(&self) -> &str { "Test" }
        fn api_version(&self) -> &str { "1.0.0" }
        fn capabilities(&self) -> Vec<ExtensionCapability> { vec![] }
    }

    #[test]
    fn test_extension_trait() {
        let ext = TestExtension;
        assert_eq!(ext.id(), "com.test.ext");
        assert_eq!(ext.name(), "Test Extension");
    }
}
