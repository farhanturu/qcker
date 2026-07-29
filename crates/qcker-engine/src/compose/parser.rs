use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use qcker_common::error::{QckerError, Result};

/// Docker Compose file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFile {
    pub version: Option<String>,
    pub services: HashMap<String, ServiceConfig>,
    pub networks: Option<HashMap<String, NetworkConfig>>,
    pub volumes: Option<HashMap<String, VolumeConfig>>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub image: Option<String>,
    pub build: Option<BuildConfig>,
    pub command: Option<CommandConfig>,
    pub entrypoint: Option<CommandConfig>,
    pub environment: Option<EnvConfig>,
    pub ports: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub networks: Option<Vec<String>>,
    pub depends_on: Option<DependsOnConfig>,
    pub restart: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub hostname: Option<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub privileged: Option<bool>,
    pub dns: Option<Vec<String>>,
    pub extra_hosts: Option<Vec<String>>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BuildConfig {
    Simple(String),
    Detailed {
        context: String,
        dockerfile: Option<String>,
        args: Option<EnvConfig>,
    },
}

/// Command configuration (string or array)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandConfig {
    Simple(String),
    Array(Vec<String>),
}

/// Environment configuration (list or map)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvConfig {
    List(Vec<String>),
    Map(HashMap<String, String>),
}

/// Depends on configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependsOnConfig {
    Simple(Vec<String>),
    Detailed(HashMap<String, DependencyConfig>),
}

/// Dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub condition: Option<String>,
}

/// Network configuration in compose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub driver: Option<String>,
    pub external: Option<bool>,
    pub name: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

/// Volume configuration in compose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub driver: Option<String>,
    pub external: Option<bool>,
    pub name: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

impl ComposeFile {
    /// Parse a compose file from YAML
    pub fn parse(content: &str) -> Result<Self> {
        serde_yaml::from_str(content)
            .map_err(|e| QckerError::InvalidArgument(format!("Failed to parse compose file: {}", e)))
    }

    /// Parse a compose file from path
    pub fn parse_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| QckerError::Internal(format!("Failed to read compose file: {}", e)))?;
        Self::parse(&content)
    }

    /// Get service names in dependency order
    pub fn get_service_order(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for name in self.services.keys() {
            if !visited.contains(name) {
                self.visit_service(name, &mut order, &mut visited, &mut visiting)?;
            }
        }

        Ok(order)
    }

    /// Visit service for topological sort
    fn visit_service(
        &self,
        name: &str,
        order: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visiting.contains(name) {
            return Err(QckerError::InvalidArgument(format!(
                "Circular dependency detected: {}",
                name
            )));
        }

        if visited.contains(name) {
            return Ok(());
        }

        visiting.insert(name.to_string());

        // Visit dependencies first
        if let Some(service) = self.services.get(name) {
            if let Some(ref deps) = service.depends_on {
                match deps {
                    DependsOnConfig::Simple(deps) => {
                        for dep in deps {
                            self.visit_service(dep, order, visited, visiting)?;
                        }
                    }
                    DependsOnConfig::Detailed(deps) => {
                        for dep in deps.keys() {
                            self.visit_service(dep, order, visited, visiting)?;
                        }
                    }
                }
            }
        }

        visiting.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());

        Ok(())
    }

    /// Get command as vec of strings
    pub fn get_command(cmd: &Option<CommandConfig>) -> Option<Vec<String>> {
        match cmd {
            Some(CommandConfig::Simple(s)) => Some(vec!["/bin/sh".to_string(), "-c".to_string(), s.clone()]),
            Some(CommandConfig::Array(arr)) => Some(arr.clone()),
            None => None,
        }
    }

    /// Get environment as vec of KEY=VALUE strings
    pub fn get_env(env: &Option<EnvConfig>) -> Vec<String> {
        match env {
            Some(EnvConfig::List(list)) => list.clone(),
            Some(EnvConfig::Map(map)) => {
                map.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
            }
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_compose() {
        let yaml = r#"
version: "3"
services:
  web:
    image: nginx:latest
    ports:
      - "8080:80"
  app:
    image: node:18
    depends_on:
      - web
"#;
        let compose = ComposeFile::parse(yaml).unwrap();
        assert_eq!(compose.services.len(), 2);
        assert!(compose.services.contains_key("web"));
        assert!(compose.services.contains_key("app"));
    }

    #[test]
    fn test_service_order() {
        let yaml = r#"
services:
  db:
    image: postgres:15
  app:
    image: node:18
    depends_on:
      - db
  web:
    image: nginx:latest
    depends_on:
      - app
"#;
        let compose = ComposeFile::parse(yaml).unwrap();
        let order = compose.get_service_order().unwrap();

        // db should come before app, app before web
        let db_pos = order.iter().position(|n| n == "db").unwrap();
        let app_pos = order.iter().position(|n| n == "app").unwrap();
        let web_pos = order.iter().position(|n| n == "web").unwrap();

        assert!(db_pos < app_pos);
        assert!(app_pos < web_pos);
    }

    #[test]
    fn test_get_env() {
        let env = Some(EnvConfig::Map({
            let mut map = HashMap::new();
            map.insert("DB_HOST".to_string(), "localhost".to_string());
            map.insert("DB_PORT".to_string(), "5432".to_string());
            map
        }));

        let result = ComposeFile::get_env(&env);
        assert_eq!(result.len(), 2);
    }
}
