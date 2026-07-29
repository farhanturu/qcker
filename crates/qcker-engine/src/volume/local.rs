use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

/// Volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub name: String,
    pub driver: String,
    pub mountpoint: PathBuf,
    pub labels: std::collections::HashMap<String, String>,
    pub created_at: String,
}

/// Volume manager
pub struct VolumeManager {
    pub data_dir: PathBuf,
}

impl VolumeManager {
    /// Create a new volume manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Initialize the volume manager
    pub fn init(&self) -> Result<()> {
        let volumes_dir = self.data_dir.join("volumes");
        fs::create_dir_all(&volumes_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create volumes dir: {}", e)))?;
        Ok(())
    }

    /// Create a volume
    pub fn create(&self, name: &str, driver: &str) -> Result<VolumeConfig> {
        let volumes_dir = self.data_dir.join("volumes");
        let volume_dir = volumes_dir.join(name);

        if volume_dir.exists() {
            return Err(QckerError::InvalidArgument(format!(
                "Volume already exists: {}",
                name
            )));
        }

        fs::create_dir_all(&volume_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create volume dir: {}", e)))?;

        let config = VolumeConfig {
            name: name.to_string(),
            driver: driver.to_string(),
            mountpoint: volume_dir.clone(),
            labels: std::collections::HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Save config
        let config_path = volume_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| QckerError::Internal(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, config_json)
            .map_err(|e| QckerError::Internal(format!("Failed to write config: {}", e)))?;

        tracing::info!("Volume {} created", name);

        Ok(config)
    }

    /// List all volumes
    pub fn list(&self) -> Result<Vec<VolumeConfig>> {
        let volumes_dir = self.data_dir.join("volumes");
        let mut volumes = Vec::new();

        if !volumes_dir.exists() {
            return Ok(volumes);
        }

        for entry in fs::read_dir(&volumes_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to read volumes dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            let config_path = entry.path().join("config.json");

            if config_path.exists() {
                let content = fs::read_to_string(&config_path)
                    .map_err(|e| QckerError::Internal(format!("Failed to read config: {}", e)))?;
                let config: VolumeConfig = serde_json::from_str(&content)
                    .map_err(|e| QckerError::Internal(format!("Failed to parse config: {}", e)))?;
                volumes.push(config);
            }
        }

        Ok(volumes)
    }

    /// Get a volume by name
    pub fn get(&self, name: &str) -> Result<VolumeConfig> {
        let volumes_dir = self.data_dir.join("volumes");
        let volume_dir = volumes_dir.join(name);
        let config_path = volume_dir.join("config.json");

        if !config_path.exists() {
            return Err(QckerError::InvalidArgument(format!(
                "Volume not found: {}",
                name
            )));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read config: {}", e)))?;
        let config: VolumeConfig = serde_json::from_str(&content)
            .map_err(|e| QckerError::Internal(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Remove a volume
    pub fn remove(&self, name: &str) -> Result<()> {
        let volumes_dir = self.data_dir.join("volumes");
        let volume_dir = volumes_dir.join(name);

        if !volume_dir.exists() {
            return Err(QckerError::InvalidArgument(format!(
                "Volume not found: {}",
                name
            )));
        }

        fs::remove_dir_all(&volume_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to remove volume: {}", e)))?;

        tracing::info!("Volume {} removed", name);

        Ok(())
    }

    /// Get volume mount path
    pub fn get_mount_path(&self, name: &str) -> Result<PathBuf> {
        let config = self.get(name)?;
        Ok(config.mountpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_volume_manager() {
        let tmp = TempDir::new().unwrap();
        let manager = VolumeManager::new(tmp.path().to_path_buf());
        manager.init().unwrap();

        // Create volume
        let volume = manager.create("test-vol", "local").unwrap();
        assert_eq!(volume.name, "test-vol");
        assert_eq!(volume.driver, "local");

        // List volumes
        let volumes = manager.list().unwrap();
        assert_eq!(volumes.len(), 1);

        // Get volume
        let retrieved = manager.get("test-vol").unwrap();
        assert_eq!(retrieved.name, "test-vol");

        // Remove volume
        manager.remove("test-vol").unwrap();
        assert!(manager.get("test-vol").is_err());
    }
}
