use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use qcker_common::error::{QckerError, Result};

pub struct PortForwarder {
    forwards: HashMap<u16, PortForward>,
}

struct PortForward {
    host_port: u16,
    guest_ip: Ipv4Addr,
    guest_port: u16,
    protocol: String,
}

impl PortForwarder {
    pub fn new() -> Self {
        Self {
            forwards: HashMap::new(),
        }
    }

    pub fn add_forward(
        &mut self,
        host_port: u16,
        guest_ip: Ipv4Addr,
        guest_port: u16,
        protocol: &str,
    ) -> Result<()> {
        self.forwards.insert(
            host_port,
            PortForward {
                host_port,
                guest_ip,
                guest_port,
                protocol: protocol.to_string(),
            },
        );
        Ok(())
    }

    pub fn remove_forward(&mut self, host_port: u16) {
        self.forwards.remove(&host_port);
    }

    pub fn list_forwards(&self) -> Vec<(u16, Ipv4Addr, u16, String)> {
        self.forwards
            .values()
            .map(|f| (f.host_port, f.guest_ip, f.guest_port, f.protocol.clone()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.forwards.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_forwarder() {
        let mut pf = PortForwarder::new();
        pf.add_forward(8080, Ipv4Addr::new(10, 0, 2, 15), 80, "tcp").unwrap();
        assert_eq!(pf.list_forwards().len(), 1);
        pf.remove_forward(8080);
        assert_eq!(pf.list_forwards().len(), 0);
    }
}
