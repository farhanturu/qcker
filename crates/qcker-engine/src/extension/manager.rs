use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};
use qcker_ext_api::types::{ExtensionInfo, ExtensionMetadata, ExtensionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionMetadata,
    pub config: Option<HashMap<String, ConfigField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub field_type: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

pub struct ExtensionManager {
    pub data_dir: PathBuf,
    pub extensions: HashMap<String, ExtensionInfo>,
}

impl ExtensionManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            extensions: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let extensions_dir = self.data_dir.join("extensions");
        fs::create_dir_all(&extensions_dir)
            .map_err(|e| QckerError::internal(format!("Failed to create extensions dir: {}", e)))?;

        self.scan_extensions()?;

        Ok(())
    }

    fn scan_extensions(&mut self) -> Result<()> {
        let extensions_dir = self.data_dir.join("extensions");

        if !extensions_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&extensions_dir)
            .map_err(|e| QckerError::internal(format!("Failed to read extensions dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::internal(format!("Failed to read entry: {}", e)))?;
            let manifest_path = entry.path().join("manifest.json");

            if manifest_path.exists() {
                let content = fs::read_to_string(&manifest_path)
                    .map_err(|e| QckerError::internal(format!("Failed to read manifest: {}", e)))?;
                let manifest: ExtensionManifest = serde_json::from_str(&content)
                    .map_err(|e| QckerError::internal(format!("Failed to parse manifest: {}", e)))?;

                let info = ExtensionInfo {
                    metadata: manifest.extension,
                    status: ExtensionStatus::Loaded,
                    path: entry.path().to_string_lossy().to_string(),
                };

                self.extensions.insert(info.metadata.id.clone(), info);
            }
        }

        Ok(())
    }

    pub fn list(&self) -> Vec<&ExtensionInfo> {
        self.extensions.values().collect()
    }

    pub fn get(&self, id: &str) -> Option<&ExtensionInfo> {
        self.extensions.get(id)
    }

    pub fn install(&mut self, source_path: &str) -> Result<()> {
        let source = std::path::Path::new(source_path);

        if !source.exists() {
            return Err(QckerError::invalid_argument(format!(
                "Extension path not found: {}",
                source_path
            )));
        }

        let manifest_path = source.join("manifest.json");
        if !manifest_path.exists() {
            return Err(QckerError::invalid_argument(
                "Extension must contain manifest.json".to_string(),
            ));
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| QckerError::internal(format!("Failed to read manifest: {}", e)))?;
        let manifest: ExtensionManifest = serde_json::from_str(&content)
            .map_err(|e| QckerError::internal(format!("Failed to parse manifest: {}", e)))?;

        let ext_id = manifest.extension.id.clone();

        if self.extensions.contains_key(&ext_id) {
            return Err(QckerError::invalid_argument(format!(
                "Extension already installed: {}",
                ext_id
            )));
        }

        let dest_dir = self.data_dir.join("extensions").join(&ext_id);
        fs::create_dir_all(&dest_dir)
            .map_err(|e| QckerError::internal(format!("Failed to create extension dir: {}", e)))?;

        fs::copy(&manifest_path, dest_dir.join("manifest.json"))
            .map_err(|e| QckerError::internal(format!("Failed to copy manifest: {}", e)))?;

        for ext in &[".so", ".dylib", ".dll"] {
            let lib_path = source.join(format!("libext{}", ext));
            if lib_path.exists() {
                fs::copy(&lib_path, dest_dir.join(format!("libext{}", ext)))
                    .map_err(|e| QckerError::internal(format!("Failed to copy library: {}", e)))?;
            }
        }

        let info = ExtensionInfo {
            metadata: manifest.extension,
            status: ExtensionStatus::Loaded,
            path: dest_dir.to_string_lossy().to_string(),
        };

        self.extensions.insert(info.metadata.id.clone(), info);

        tracing::info!("Extension {} installed", ext_id);

        Ok(())
    }

    pub fn uninstall(&mut self, id: &str) -> Result<()> {
        if !self.extensions.contains_key(id) {
            return Err(QckerError::invalid_argument(format!(
                "Extension not found: {}",
                id
            )));
        }

        let ext_dir = self.data_dir.join("extensions").join(id);
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir)
                .map_err(|e| QckerError::internal(format!("Failed to remove extension: {}", e)))?;
        }

        self.extensions.remove(id);

        tracing::info!("Extension {} uninstalled", id);

        Ok(())
    }

    pub fn enable(&mut self, id: &str) -> Result<()> {
        if let Some(ext) = self.extensions.get_mut(id) {
            ext.status = ExtensionStatus::Active;
            tracing::info!("Extension {} enabled", id);
            Ok(())
        } else {
            Err(QckerError::invalid_argument(format!(
                "Extension not found: {}",
                id
            )))
        }
    }

    pub fn disable(&mut self, id: &str) -> Result<()> {
        if let Some(ext) = self.extensions.get_mut(id) {
            ext.status = ExtensionStatus::Disabled;
            tracing::info!("Extension {} disabled", id);
            Ok(())
        } else {
            Err(QckerError::invalid_argument(format!(
                "Extension not found: {}",
                id
            )))
        }
    }
}

