use std::path::PathBuf;

use qcker_common::error::Result;

use super::parser::{ComposeFile, ServiceConfig};

/// Compose project
pub struct ComposeProject {
    pub name: String,
    pub file: ComposeFile,
    pub work_dir: PathBuf,
    pub data_dir: PathBuf,
}

/// Service status
#[derive(Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub state: String,
    pub container_id: Option<String>,
}

impl ComposeProject {
    /// Create a new compose project
    pub fn new(name: &str, file: ComposeFile, work_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            file,
            work_dir,
            data_dir,
        }
    }

    /// Get project network name
    pub fn network_name(&self) -> String {
        format!("{}_default", self.name)
    }

    /// Get service container name
    pub fn container_name(&self, service: &str) -> String {
        format!("{}_{}_1", self.name, service)
    }

    /// Start all services
    pub fn up(&self, services: Option<&[String]>) -> Result<()> {
        let service_order = self.file.get_service_order()?;
        let services_to_start: Vec<String> = if let Some(services) = services {
            services.iter().map(|s| s.to_string()).collect()
        } else {
            service_order
        };

        tracing::info!("Starting project {}", self.name);

        // Create network
        let network_name = self.network_name();
        tracing::info!("Creating network {}", network_name);

        // Start services in order
        for service_name in &services_to_start {
            if let Some(service_config) = self.file.services.get(service_name) {
                self.start_service(service_name, service_config)?;
            }
        }

        tracing::info!("Project {} started", self.name);

        Ok(())
    }

    /// Start a single service
    fn start_service(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        let container_name = self.container_name(name);

        tracing::info!("Starting service {} ({})", name, container_name);

        // Get image
        let image = config.image.clone().unwrap_or_else(|| {
            format!("{}:{}", name, "latest")
        });

        // Get command
        let _command = ComposeFile::get_command(&config.command);

        // Get environment
        let _env = ComposeFile::get_env(&config.environment);

        // Get ports
        let ports = config.ports.clone().unwrap_or_default();

        // Get volumes
        let volumes = config.volumes.clone().unwrap_or_default();

        tracing::info!("  Image: {}", image);
        if !ports.is_empty() {
            tracing::info!("  Ports: {:?}", ports);
        }
        if !volumes.is_empty() {
            tracing::info!("  Volumes: {:?}", volumes);
        }

        // In a real implementation, this would:
        // 1. Pull image if not exists
        // 2. Create container with config
        // 3. Start container
        // 4. Wait for health check if configured

        Ok(())
    }

    /// Stop all services
    pub fn down(&self, _remove_volumes: bool) -> Result<()> {
        tracing::info!("Stopping project {}", self.name);

        // Stop services in reverse order
        let service_order = self.file.get_service_order()?;
        for service_name in service_order.iter().rev() {
            let container_name = self.container_name(service_name);
            tracing::info!("Stopping service {} ({})", service_name, container_name);

            // In a real implementation, this would:
            // 1. Stop container
            // 2. Remove container
            // 3. Optionally remove volumes
        }

        // Remove network
        let network_name = self.network_name();
        tracing::info!("Removing network {}", network_name);

        tracing::info!("Project {} stopped", self.name);

        Ok(())
    }

    /// Get status of all services
    pub fn ps(&self) -> Result<Vec<ServiceStatus>> {
        let mut statuses = Vec::new();

        for (name, _) in &self.file.services {
            statuses.push(ServiceStatus {
                name: name.clone(),
                state: "stopped".to_string(), // TODO: Check actual state
                container_id: None,
            });
        }

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::parser::ComposeFile;

    #[test]
    fn test_compose_project() {
        let yaml = r#"
services:
  web:
    image: nginx:latest
  app:
    image: node:18
"#;
        let file = ComposeFile::parse(yaml).unwrap();
        let tmp = tempfile::TempDir::new().unwrap();

        let project = ComposeProject::new(
            "test",
            file,
            tmp.path().to_path_buf(),
            tmp.path().to_path_buf(),
        );

        assert_eq!(project.network_name(), "test_default");
        assert_eq!(project.container_name("web"), "test_web_1");
    }
}
