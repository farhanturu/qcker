use std::path::PathBuf;

use qcker_common::error::Result;

use super::parser::{ComposeFile, ServiceConfig};

pub struct ComposeProject {
    pub name: String,
    pub file: ComposeFile,
    pub work_dir: PathBuf,
    pub data_dir: PathBuf,
}

#[derive(Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub state: String,
    pub container_id: Option<String>,
}

impl ComposeProject {
    pub fn new(name: &str, file: ComposeFile, work_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            file,
            work_dir,
            data_dir,
        }
    }

    pub fn network_name(&self) -> String {
        format!("{}_default", self.name)
    }

    pub fn container_name(&self, service: &str) -> String {
        format!("{}_{}_1", self.name, service)
    }

    pub fn up(&self, services: Option<&[String]>) -> Result<()> {
        let service_order = self.file.get_service_order()?;
        let services_to_start: Vec<String> = if let Some(services) = services {
            services.iter().map(|s| s.to_string()).collect()
        } else {
            service_order
        };

        tracing::info!("Starting project {}", self.name);

        for service_name in &services_to_start {
            if let Some(service_config) = self.file.services.get(service_name) {
                self.start_service(service_name, service_config)?;
            }
        }

        tracing::info!("Project {} started", self.name);
        Ok(())
    }

    fn start_service(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        let container_name = self.container_name(name);
        let image = config.image.clone().unwrap_or_else(|| format!("{}:latest", name));

        tracing::info!("Starting service {} ({})", name, container_name);
        tracing::info!("  Image: {}", image);

        if let Some(ref ports) = config.ports {
            tracing::info!("  Ports: {:?}", ports);
        }

        if let Some(ref volumes) = config.volumes {
            tracing::info!("  Volumes: {:?}", volumes);
        }

        Ok(())
    }

    pub fn down(&self, _remove_volumes: bool) -> Result<()> {
        tracing::info!("Stopping project {}", self.name);

        let service_order = self.file.get_service_order()?;
        for service_name in service_order.iter().rev() {
            let container_name = self.container_name(service_name);
            tracing::info!("Stopping service {} ({})", service_name, container_name);
        }

        tracing::info!("Removing network {}", self.network_name());
        tracing::info!("Project {} stopped", self.name);

        Ok(())
    }

    pub fn ps(&self) -> Result<Vec<ServiceStatus>> {
        let mut statuses = Vec::new();

        for name in self.file.services.keys() {
            statuses.push(ServiceStatus {
                name: name.clone(),
                state: "stopped".to_string(),
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
