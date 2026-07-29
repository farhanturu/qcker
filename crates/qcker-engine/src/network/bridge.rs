use std::fs;
use std::process::Command;

use qcker_common::error::{QckerError, Result};

use super::types::NetworkConfig;

/// Bridge network implementation
pub struct BridgeNetwork {
    pub bridge_name: String,
    pub subnet: String,
    pub gateway: String,
    pub mtu: u32,
}

impl BridgeNetwork {
    /// Create a new bridge network
    pub fn new(config: &NetworkConfig) -> Result<Self> {
        let bridge_name = format!("qcker-{}", &config.id[..8]);
        let subnet = config.subnet.clone().unwrap_or_else(|| "172.17.0.0/16".to_string());
        let gateway = config.gateway.clone().unwrap_or_else(|| "172.17.0.1".to_string());

        Ok(Self {
            bridge_name,
            subnet,
            gateway,
            mtu: 1500,
        })
    }

    /// Setup the bridge network
    pub fn setup(&self) -> Result<()> {
        // Check if bridge already exists
        if self.bridge_exists() {
            return Ok(());
        }

        // Create bridge interface
        let output = Command::new("ip")
            .args(["link", "add", &self.bridge_name, "type", "bridge"])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to create bridge: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to create bridge: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Set bridge IP
        let output = Command::new("ip")
            .args([
                "addr",
                "add",
                &format!("{}/{}", self.gateway, self.subnet.split('/').nth(1).unwrap_or("16")),
                "dev",
                &self.bridge_name,
            ])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to set bridge IP: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to set bridge IP: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Bring up bridge
        let output = Command::new("ip")
            .args(["link", "set", &self.bridge_name, "up"])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to bring up bridge: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to bring up bridge: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Enable IP forwarding
        fs::write("/proc/sys/net/ipv4/ip_forward", "1")
            .map_err(|e| QckerError::Network(format!("Failed to enable IP forwarding: {}", e)))?;

        // Setup NAT with iptables
        self.setup_nat()?;

        tracing::info!("Bridge network {} created", self.bridge_name);

        Ok(())
    }

    /// Setup NAT rules
    fn setup_nat(&self) -> Result<()> {
        // Add MASQUERADE rule for outbound traffic
        let output = Command::new("iptables")
            .args([
                "-t", "nat", "-A", "POSTROUTING",
                "-s", &self.subnet,
                "!", "-o", &self.bridge_name,
                "-j", "MASQUERADE",
            ])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to setup NAT: {}", e)))?;

        if !output.status.success() {
            tracing::warn!(
                "Failed to setup NAT: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Check if bridge exists
    fn bridge_exists(&self) -> bool {
        Command::new("ip")
            .args(["link", "show", &self.bridge_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Create veth pair and connect to bridge
    pub fn connect_container(&self, container_id: &str, container_pid: i32) -> Result<()> {
        let veth_host = format!("veth-{}", &container_id[..8]);
        let veth_container = format!("eth0");

        // Create veth pair
        let output = Command::new("ip")
            .args([
                "link",
                "add",
                &veth_host,
                "type",
                "veth",
                "peer",
                "name",
                &veth_container,
            ])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to create veth pair: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to create veth pair: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Move container end to container namespace
        let output = Command::new("ip")
            .args([
                "link",
                "set",
                &veth_container,
                "netns",
                &container_pid.to_string(),
            ])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to move veth to namespace: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to move veth to namespace: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Attach host end to bridge
        let output = Command::new("ip")
            .args(["link", "set", &veth_host, "master", &self.bridge_name])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to attach veth to bridge: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to attach veth to bridge: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Bring up host end
        let output = Command::new("ip")
            .args(["link", "set", &veth_host, "up"])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to bring up veth: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to bring up veth: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        tracing::info!(
            "Container {} connected to bridge {}",
            container_id,
            self.bridge_name
        );

        Ok(())
    }

    /// Setup port forwarding
    pub fn setup_port_forward(
        &self,
        host_port: u16,
        container_ip: &str,
        container_port: u16,
        protocol: &str,
    ) -> Result<()> {
        let output = Command::new("iptables")
            .args([
                "-t", "nat", "-A", "PREROUTING",
                "-p", protocol,
                "--dport", &host_port.to_string(),
                "-j", "DNAT",
                "--to-destination",
                &format!("{}:{}", container_ip, container_port),
            ])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to setup port forwarding: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to setup port forwarding: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        tracing::info!(
            "Port forwarding: {}:{} -> {}:{}",
            "0.0.0.0",
            host_port,
            container_ip,
            container_port
        );

        Ok(())
    }

    /// Remove bridge network
    pub fn remove(&self) -> Result<()> {
        if !self.bridge_exists() {
            return Ok(());
        }

        let output = Command::new("ip")
            .args(["link", "del", &self.bridge_name])
            .output()
            .map_err(|e| QckerError::Network(format!("Failed to remove bridge: {}", e)))?;

        if !output.status.success() {
            return Err(QckerError::Network(format!(
                "Failed to remove bridge: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        tracing::info!("Bridge network {} removed", self.bridge_name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_network() {
        let config = NetworkConfig::new_bridge("test", "172.20.0.0/16");
        let bridge = BridgeNetwork::new(&config).unwrap();
        assert!(bridge.bridge_name.starts_with("qcker-"));
        assert_eq!(bridge.subnet, "172.20.0.0/16");
        assert_eq!(bridge.gateway, "172.20.0.1");
    }
}
