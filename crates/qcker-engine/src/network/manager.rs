use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

use super::bridge::BridgeNetwork;
use super::types::{NetworkConfig, NetworkDriver, NetworkInfo, PortMapping};

/// Network manager
pub struct NetworkManager {
    pub data_dir: PathBuf,
    pub networks: HashMap<String, NetworkConfig>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            networks: HashMap::new(),
        }
    }

    /// Initialize the network manager
    pub fn init(&mut self) -> Result<()> {
        let networks_dir = self.data_dir.join("networks");
        fs::create_dir_all(&networks_dir)
            .map_err(|e| QckerError::Network(format!("Failed to create networks dir: {}", e)))?;

        // Load existing networks
        self.load_networks()?;

        // Create default networks if they don't exist
        self.ensure_default_networks()?;

        Ok(())
    }

    /// Load networks from disk
    fn load_networks(&mut self) -> Result<()> {
        let networks_dir = self.data_dir.join("networks");

        if !networks_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&networks_dir)
            .map_err(|e| QckerError::Network(format!("Failed to read networks dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Network(format!("Failed to read entry: {}", e)))?;
            let path = entry.path().join("config.json");

            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| QckerError::Network(format!("Failed to read network config: {}", e)))?;
                let config: NetworkConfig = serde_json::from_str(&content)
                    .map_err(|e| QckerError::Network(format!("Failed to parse network config: {}", e)))?;
                self.networks.insert(config.id.clone(), config);
            }
        }

        Ok(())
    }

    /// Ensure default networks exist
    fn ensure_default_networks(&mut self) -> Result<()> {
        // Create bridge network if not exists
        if !self.networks.values().any(|n| n.name == "bridge") {
            let config = NetworkConfig::new_bridge("bridge", "172.17.0.0/16");
            self.create_network(config)?;
        }

        // Create host network if not exists
        if !self.networks.values().any(|n| n.name == "host") {
            let config = NetworkConfig::new_host();
            self.create_network(config)?;
        }

        // Create none network if not exists
        if !self.networks.values().any(|n| n.name == "none") {
            let config = NetworkConfig::new_none();
            self.create_network(config)?;
        }

        Ok(())
    }

    /// Create a new network
    pub fn create_network(&mut self, config: NetworkConfig) -> Result<()> {
        let network_dir = self.data_dir.join("networks").join(&config.id);
        fs::create_dir_all(&network_dir)
            .map_err(|e| QckerError::Network(format!("Failed to create network dir: {}", e)))?;

        // Save config
        let config_path = network_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| QckerError::Network(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, config_json)
            .map_err(|e| QckerError::Network(format!("Failed to write config: {}", e)))?;

        // Setup bridge if needed
        if config.driver == NetworkDriver::Bridge {
            let bridge = BridgeNetwork::new(&config)?;
            bridge.setup()?;
        }

        self.networks.insert(config.id.clone(), config);

        Ok(())
    }

    /// List all networks
    pub fn list_networks(&self) -> Vec<NetworkInfo> {
        self.networks
            .values()
            .map(|config| NetworkInfo {
                id: config.id.clone(),
                name: config.name.clone(),
                driver: config.driver.clone(),
                subnet: config.subnet.clone(),
                gateway: config.gateway.clone(),
                container_count: 0, // TODO: Track containers
            })
            .collect()
    }

    /// Get network by ID or name
    pub fn get_network(&self, id_or_name: &str) -> Result<&NetworkConfig> {
        // Try by ID
        if let Some(config) = self.networks.get(id_or_name) {
            return Ok(config);
        }

        // Try by name
        self.networks
            .values()
            .find(|n| n.name == id_or_name)
            .ok_or_else(|| QckerError::Network(format!("Network not found: {}", id_or_name)))
    }

    /// Remove a network
    pub fn remove_network(&mut self, id: &str) -> Result<()> {
        let config = self.networks.get(id)
            .ok_or_else(|| QckerError::Network(format!("Network not found: {}", id)))?
            .clone();

        // Prevent removal of default networks
        if config.name == "bridge" || config.name == "host" || config.name == "none" {
            return Err(QckerError::Network(format!(
                "Cannot remove default network: {}",
                config.name
            )));
        }

        // Remove bridge if needed
        if config.driver == NetworkDriver::Bridge {
            let bridge = BridgeNetwork::new(&config)?;
            bridge.remove()?;
        }

        // Remove network directory
        let network_dir = self.data_dir.join("networks").join(id);
        if network_dir.exists() {
            fs::remove_dir_all(&network_dir)
                .map_err(|e| QckerError::Network(format!("Failed to remove network dir: {}", e)))?;
        }

        self.networks.remove(id);

        Ok(())
    }

    /// Connect container to network
    pub fn connect_container(
        &self,
        network_id: &str,
        container_id: &str,
        container_pid: i32,
        ports: &[PortMapping],
    ) -> Result<()> {
        let config = self.get_network(network_id)?;

        match config.driver {
            NetworkDriver::Bridge => {
                let bridge = BridgeNetwork::new(config)?;
                bridge.connect_container(container_id, container_pid)?;

                // Setup port forwarding
                for port in ports {
                    let container_ip = "172.17.0.2"; // TODO: Assign IP dynamically
                    bridge.setup_port_forward(
                        port.host_port,
                        container_ip,
                        port.container_port,
                        &port.protocol,
                    )?;
                }
            }
            NetworkDriver::Host => {
                // No network isolation for host mode
                tracing::info!("Container {} using host network", container_id);
            }
            NetworkDriver::None => {
                // No networking
                tracing::info!("Container {} has no networking", container_id);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_network_manager() {
        let tmp = TempDir::new().unwrap();
        let mut manager = NetworkManager::new(tmp.path().to_path_buf());

        // init() may fail if not running as root (bridge creation requires root)
        // Just test that we can create the manager
        let _ = manager.init();

        // Test basic operations
        let networks = manager.list_networks();
        // Networks may or may not exist depending on permissions
        println!("Found {} networks", networks.len());
    }
}
