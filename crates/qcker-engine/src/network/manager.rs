use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

use super::bridge::BridgeNetwork;
use super::types::{NetworkConfig, NetworkDriver, NetworkInfo, PortMapping};

pub struct NetworkManager {
    pub data_dir: PathBuf,
    pub networks: HashMap<String, NetworkConfig>,
}

impl NetworkManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            networks: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let networks_dir = self.data_dir.join("networks");
        fs::create_dir_all(&networks_dir)
            .map_err(|e| QckerError::network(format!("Failed to create networks dir: {}", e)))?;

        self.load_networks()?;

        self.ensure_default_networks()?;

        Ok(())
    }

    fn load_networks(&mut self) -> Result<()> {
        let networks_dir = self.data_dir.join("networks");

        if !networks_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&networks_dir)
            .map_err(|e| QckerError::network(format!("Failed to read networks dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::network(format!("Failed to read entry: {}", e)))?;
            let path = entry.path().join("config.json");

            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| QckerError::network(format!("Failed to read network config: {}", e)))?;
                let config: NetworkConfig = serde_json::from_str(&content)
                    .map_err(|e| QckerError::network(format!("Failed to parse network config: {}", e)))?;
                self.networks.insert(config.id.clone(), config);
            }
        }

        Ok(())
    }

    fn ensure_default_networks(&mut self) -> Result<()> {
        if !self.networks.values().any(|n| n.name == "bridge") {
            let config = NetworkConfig::new_bridge("bridge", "172.17.0.0/16");
            self.create_network(config)?;
        }

        if !self.networks.values().any(|n| n.name == "host") {
            let config = NetworkConfig::new_host();
            self.create_network(config)?;
        }

        if !self.networks.values().any(|n| n.name == "none") {
            let config = NetworkConfig::new_none();
            self.create_network(config)?;
        }

        Ok(())
    }

    pub fn create_network(&mut self, config: NetworkConfig) -> Result<()> {
        let network_dir = self.data_dir.join("networks").join(&config.id);
        fs::create_dir_all(&network_dir)
            .map_err(|e| QckerError::network(format!("Failed to create network dir: {}", e)))?;

        let config_path = network_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| QckerError::network(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, config_json)
            .map_err(|e| QckerError::network(format!("Failed to write config: {}", e)))?;

        if config.driver == NetworkDriver::Bridge {
            let bridge = BridgeNetwork::new(&config)?;
            bridge.setup()?;
        }

        self.networks.insert(config.id.clone(), config);

        Ok(())
    }

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

    pub fn get_network(&self, id_or_name: &str) -> Result<&NetworkConfig> {
        if let Some(config) = self.networks.get(id_or_name) {
            return Ok(config);
        }

        self.networks
            .values()
            .find(|n| n.name == id_or_name)
            .ok_or_else(|| QckerError::network(format!("Network not found: {}", id_or_name)))
    }

    pub fn remove_network(&mut self, id: &str) -> Result<()> {
        let config = self.networks.get(id)
            .ok_or_else(|| QckerError::network(format!("Network not found: {}", id)))?
            .clone();

        if config.name == "bridge" || config.name == "host" || config.name == "none" {
            return Err(QckerError::network(format!(
                "Cannot remove default network: {}",
                config.name
            )));
        }

        if config.driver == NetworkDriver::Bridge {
            let bridge = BridgeNetwork::new(&config)?;
            bridge.remove()?;
        }

        let network_dir = self.data_dir.join("networks").join(id);
        if network_dir.exists() {
            fs::remove_dir_all(&network_dir)
                .map_err(|e| QckerError::network(format!("Failed to remove network dir: {}", e)))?;
        }

        self.networks.remove(id);

        Ok(())
    }

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
                tracing::info!("Container {} using host network", container_id);
            }
            NetworkDriver::None => {
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

        let _ = manager.init();

        let networks = manager.list_networks();
        println!("Found {} networks", networks.len());
    }
}
