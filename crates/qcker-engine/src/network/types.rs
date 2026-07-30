use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkDriver {
    Bridge,
    Host,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub id: String,
    pub name: String,
    pub driver: NetworkDriver,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub ip_range: Option<String>,
    pub internal: bool,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkConfig {
    pub container_id: String,
    pub network_id: String,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub ports: Vec<PortMapping>,
    pub dns: Vec<String>,
    pub hostname: Option<String>,
    pub extra_hosts: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_ip: Option<String>,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: NetworkDriver,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub container_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetail {
    pub config: NetworkConfig,
    pub containers: Vec<ContainerNetworkConfig>,
    pub created_at: String,
}

impl NetworkConfig {
    pub fn new_bridge(name: &str, subnet: &str) -> Self {
        let gateway = compute_gateway(subnet);
        Self {
            id: qcker_common::id::generate_network_id(),
            name: name.to_string(),
            driver: NetworkDriver::Bridge,
            subnet: Some(subnet.to_string()),
            gateway: Some(gateway),
            ip_range: None,
            internal: false,
            labels: std::collections::HashMap::new(),
        }
    }

    pub fn new_host() -> Self {
        Self {
            id: "host".to_string(),
            name: "host".to_string(),
            driver: NetworkDriver::Host,
            subnet: None,
            gateway: None,
            ip_range: None,
            internal: false,
            labels: std::collections::HashMap::new(),
        }
    }

    pub fn new_none() -> Self {
        Self {
            id: "none".to_string(),
            name: "none".to_string(),
            driver: NetworkDriver::None,
            subnet: None,
            gateway: None,
            ip_range: None,
            internal: false,
            labels: std::collections::HashMap::new(),
        }
    }
}

fn compute_gateway(subnet: &str) -> String {
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.is_empty() {
        return "172.17.0.1".to_string();
    }

    let ip_parts: Vec<u8> = parts[0]
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    if ip_parts.len() == 4 {
        format!("{}.{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3] + 1)
    } else {
        "172.17.0.1".to_string()
    }
}

pub fn parse_port_mapping(port_str: &str) -> Result<PortMapping, String> {
    let parts: Vec<&str> = port_str.split('/').collect();
    let protocol = if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "tcp".to_string()
    };

    let port_part = parts[0];
    let port_parts: Vec<&str> = port_part.split(':').collect();

    match port_parts.len() {
        2 => {
            let host_port: u16 = port_parts[0].parse().map_err(|_| "Invalid host port")?;
            let container_port: u16 = port_parts[1].parse().map_err(|_| "Invalid container port")?;
            Ok(PortMapping {
                host_ip: None,
                host_port,
                container_port,
                protocol,
            })
        }
        3 => {
            let host_ip = port_parts[0].to_string();
            let host_port: u16 = port_parts[1].parse().map_err(|_| "Invalid host port")?;
            let container_port: u16 = port_parts[2].parse().map_err(|_| "Invalid container port")?;
            Ok(PortMapping {
                host_ip: Some(host_ip),
                host_port,
                container_port,
                protocol,
            })
        }
        _ => Err("Invalid port format".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config() {
        let config = NetworkConfig::new_bridge("test-net", "172.20.0.0/16");
        assert_eq!(config.name, "test-net");
        assert_eq!(config.driver, NetworkDriver::Bridge);
        assert_eq!(config.subnet, Some("172.20.0.0/16".to_string()));
        assert_eq!(config.gateway, Some("172.20.0.1".to_string()));
    }

    #[test]
    fn test_parse_port_mapping() {
        let mapping = parse_port_mapping("8080:80").unwrap();
        assert_eq!(mapping.host_port, 8080);
        assert_eq!(mapping.container_port, 80);
        assert_eq!(mapping.protocol, "tcp");

        let mapping = parse_port_mapping("127.0.0.1:8080:80/tcp").unwrap();
        assert_eq!(mapping.host_ip, Some("127.0.0.1".to_string()));
        assert_eq!(mapping.host_port, 8080);
        assert_eq!(mapping.container_port, 80);
        assert_eq!(mapping.protocol, "tcp");
    }

    #[test]
    fn test_compute_gateway() {
        assert_eq!(compute_gateway("172.20.0.0/16"), "172.20.0.1");
        assert_eq!(compute_gateway("10.0.0.0/24"), "10.0.0.1");
    }
}
