use super::types::*;
use std::collections::HashMap;

/// Base extension trait - all extensions must implement
pub trait Extension: Send + Sync {
    /// Unique extension identifier
    fn id(&self) -> &str;

    /// Extension version
    fn version(&self) -> &str;

    /// Extension name
    fn name(&self) -> &str;

    /// Extension description
    fn description(&self) -> &str;

    /// Author
    fn author(&self) -> &str;

    /// Required API version
    fn api_version(&self) -> &str;

    /// List of capabilities this extension requires
    fn capabilities(&self) -> Vec<ExtensionCapability>;

    /// Called when extension is loaded
    fn on_load(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Called when extension is unloaded
    fn on_unload(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// Network driver extension trait
pub trait NetworkDriver: Extension {
    /// Initialize the network driver
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    /// Create a network
    fn create_network(&self, name: &str, config: &HashMap<String, String>) -> Result<String, String>;

    /// Delete a network
    fn delete_network(&self, network_id: &str) -> Result<(), String>;

    /// Connect a container to a network
    fn connect(&self, network_id: &str, container_id: &str, config: &HashMap<String, String>) -> Result<(), String>;

    /// Disconnect a container from a network
    fn disconnect(&self, network_id: &str, container_id: &str) -> Result<(), String>;

    /// List networks
    fn list_networks(&self) -> Result<Vec<NetworkInfo>, String>;

    /// Return driver name
    fn driver_name(&self) -> &str;
}

/// Storage driver extension trait
pub trait StorageDriver: Extension {
    /// Initialize the storage driver
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    /// Prepare a snapshot
    fn prepare(&self, key: &str, parent: &str) -> Result<String, String>;

    /// Commit a snapshot
    fn commit(&self, key: &str, name: &str) -> Result<(), String>;

    /// Remove a snapshot
    fn remove(&self, key: &str) -> Result<(), String>;

    /// Return driver name
    fn driver_name(&self) -> &str;
}

/// Security scanner extension trait
pub trait SecurityScanner: Extension {
    /// Initialize the scanner
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    /// Scan an image for vulnerabilities
    fn scan_image(&self, image_id: &str, config: &HashMap<String, String>) -> Result<ScanResult, String>;

    /// Return scanner name
    fn scanner_name(&self) -> &str;
}

/// Scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: ScanSummary,
}

/// Vulnerability
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub severity: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub description: String,
}

/// Scan summary
#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Log driver extension trait
pub trait LogDriver: Extension {
    /// Initialize the log driver
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    /// Start capturing logs
    fn start_capture(&self, container_id: &str, config: &HashMap<String, String>) -> Result<String, String>;

    /// Stop capturing logs
    fn stop_capture(&self, handle: &str) -> Result<(), String>;

    /// Read logs
    fn read_logs(&self, container_id: &str, config: &HashMap<String, String>) -> Result<Vec<LogEntry>, String>;

    /// Return driver name
    fn driver_name(&self) -> &str;
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub stream: String,
    pub message: String,
}

/// Build strategy extension trait
pub trait BuildStrategy: Extension {
    /// Initialize the build strategy
    fn init(&mut self, config: &HashMap<String, String>) -> Result<(), String>;

    /// Build an image
    fn build(&self, context_path: &str, config: &HashMap<String, String>) -> Result<String, String>;

    /// Return strategy name
    fn strategy_name(&self) -> &str;
}

/// Command extension trait - adds custom CLI subcommands
pub trait CommandExtension: Extension {
    /// Return the subcommand name
    fn command_name(&self) -> &str;

    /// Return help text
    fn help(&self) -> &str;

    /// Execute the command
    fn execute(&self, args: &[String]) -> Result<(), String>;
}

/// Hook extension trait - lifecycle hooks
pub trait HookExtension: Extension {
    /// Called before container starts
    fn pre_start(&self, container: &ContainerInfo) -> Result<HookDecision, String>;

    /// Called after container starts
    fn post_start(&self, container: &ContainerInfo) -> Result<(), String>;

    /// Called before container stops
    fn pre_stop(&self, container: &ContainerInfo) -> Result<HookDecision, String>;

    /// Called after container stops
    fn post_stop(&self, container: &ContainerInfo) -> Result<(), String>;
}

/// Hook decision
#[derive(Debug, Clone)]
pub enum HookDecision {
    Allow,
    Deny(String),
}

/// Network info
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
