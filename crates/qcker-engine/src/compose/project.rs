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

        use qcker_runtime::process::ContainerProcess;
        use qcker_runtime::spec::{LinuxConfig, NamespaceConfig, NamespaceType, OciConfig, ProcessConfig, RootConfig, UserConfig};

        let container_dir = self.data_dir.join(&container_name);
        let rootfs = container_dir.join("rootfs");
        let rootfs_clone = rootfs.clone();
        std::fs::create_dir_all(&rootfs)?;

        let args = ComposeFile::get_command(&config.command).unwrap_or_else(|| vec!["/bin/sh".to_string()]);
        let env = ComposeFile::get_env(&config.environment);
        let cwd = config.working_dir.clone().unwrap_or_else(|| "/".to_string());

        let config_oci = OciConfig {
            oci_version: "1.0.0".to_string(),
            root: RootConfig { path: rootfs, readonly: false },
            process: Some(ProcessConfig {
                terminal: false,
                user: UserConfig { uid: 0, gid: 0 },
                args,
                env,
                cwd,
                capabilities: None,
                rlimits: vec![],
                no_new_privileges: true,
            }),
            hostname: Some(container_name.clone()),
            mounts: vec![],
            linux: Some(LinuxConfig {
                namespaces: vec![
                    NamespaceConfig { r#type: NamespaceType::Pid, path: None },
                    NamespaceConfig { r#type: NamespaceType::Network, path: None },
                    NamespaceConfig { r#type: NamespaceType::Mount, path: None },
                    NamespaceConfig { r#type: NamespaceType::Uts, path: None },
                    NamespaceConfig { r#type: NamespaceType::Ipc, path: None },
                ],
                resources: None,
                uid_mappings: vec![],
                gid_mappings: vec![],
                seccomp: None,
            }),
        };

        let mut container = ContainerProcess::new(
            &container_name,
            &rootfs_clone,
            config_oci,
            self.data_dir.clone(),
        )?;
        container.create()?;
        container.start()?;

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

        for (name, _) in &self.file.services {
            statuses.push(ServiceStatus {
                name: name.clone(),
                state: "stopped".to_string(),
                container_id: None,
            });
        }

        Ok(statuses)
    }

    pub fn build(&self, services: Option<&[String]>) -> Result<()> {
        tracing::info!("Building project {}", self.name);
        let service_order = self.file.get_service_order()?;
        let services_to_build: Vec<String> = if let Some(services) = services {
            services.iter().map(|s| s.to_string()).collect()
        } else {
            service_order
        };

        for service_name in &services_to_build {
            if let Some(service_config) = self.file.services.get(service_name) {
                if let Some(ref build_config) = service_config.build {
                    tracing::info!("Building service {}: {:?}", service_name, build_config);
                }
            }
        }
        Ok(())
    }

    pub async fn pull(&self, services: Option<&[String]>) -> Result<()> {
        use crate::registry::client::RegistryClient;
        tracing::info!("Pulling project {}", self.name);
        let service_order = self.file.get_service_order()?;
        let services_to_pull: Vec<String> = if let Some(services) = services {
            services.iter().map(|s| s.to_string()).collect()
        } else {
            service_order
        };

        let client = RegistryClient::new("registry-1.docker.io");
        for service_name in &services_to_pull {
            if let Some(service_config) = self.file.services.get(service_name) {
                if let Some(ref image) = service_config.image {
                    tracing::info!("Pulling image {} for service {}", image, service_name);
                    let _ = client.pull_image(image, self.data_dir.clone()).await?;
                }
            }
        }
        Ok(())
    }

    pub fn logs(&self, services: Option<&[String]>, follow: bool) -> Result<()> {
        tracing::info!("Showing logs for project {}", self.name);
        let service_order = self.file.get_service_order()?;
        let services_to_log: Vec<String> = if let Some(services) = services {
            services.iter().map(|s| s.to_string()).collect()
        } else {
            service_order
        };

        for service_name in &services_to_log {
            tracing::info!("Logs for service {} (follow={})", service_name, follow);
        }
        Ok(())
    }
}

