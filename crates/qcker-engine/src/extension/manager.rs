use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};
use qcker_ext_api::types::{ExtensionInfo, ExtensionMetadata, ExtensionStatus};

/// Extension configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionMetadata,
    pub config: Option<HashMap<String, ConfigField>>,
}

/// Config field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub field_type: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

/// Extension manager
pub struct ExtensionManager {
    pub data_dir: PathBuf,
    pub extensions: HashMap<String, ExtensionInfo>,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            extensions: HashMap::new(),
        }
    }

    /// Initialize the extension manager
    pub fn init(&mut self) -> Result<()> {
        let extensions_dir = self.data_dir.join("extensions");
        fs::create_dir_all(&extensions_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create extensions dir: {}", e)))?;

        // Scan for extensions
        self.scan_extensions()?;

        Ok(())
    }

    /// Scan for installed extensions
    fn scan_extensions(&mut self) -> Result<()> {
        let extensions_dir = self.data_dir.join("extensions");

        if !extensions_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&extensions_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to read extensions dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            let manifest_path = entry.path().join("manifest.json");

            if manifest_path.exists() {
                let content = fs::read_to_string(&manifest_path)
                    .map_err(|e| QckerError::Internal(format!("Failed to read manifest: {}", e)))?;
                let manifest: ExtensionManifest = serde_json::from_str(&content)
                    .map_err(|e| QckerError::Internal(format!("Failed to parse manifest: {}", e)))?;

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

    /// List all extensions
    pub fn list(&self) -> Vec<&ExtensionInfo> {
        self.extensions.values().collect()
    }

    /// Get extension by ID
    pub fn get(&self, id: &str) -> Option<&ExtensionInfo> {
        self.extensions.get(id)
    }

    /// Install an extension from a path
    pub fn install(&mut self, source_path: &str) -> Result<()> {
        let source = std::path::Path::new(source_path);

        if !source.exists() {
            return Err(QckerError::InvalidArgument(format!(
                "Extension path not found: {}",
                source_path
            )));
        }

        // Read manifest
        let manifest_path = source.join("manifest.json");
        if !manifest_path.exists() {
            return Err(QckerError::InvalidArgument(
                "Extension must contain manifest.json".to_string(),
            ));
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read manifest: {}", e)))?;
        let manifest: ExtensionManifest = serde_json::from_str(&content)
            .map_err(|e| QckerError::Internal(format!("Failed to parse manifest: {}", e)))?;

        let ext_id = manifest.extension.id.clone();

        // Check if already installed
        if self.extensions.contains_key(&ext_id) {
            return Err(QckerError::InvalidArgument(format!(
                "Extension already installed: {}",
                ext_id
            )));
        }

        // Copy extension to data dir
        let dest_dir = self.data_dir.join("extensions").join(&ext_id);
        fs::create_dir_all(&dest_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create extension dir: {}", e)))?;

        // Copy manifest
        fs::copy(&manifest_path, dest_dir.join("manifest.json"))
            .map_err(|e| QckerError::Internal(format!("Failed to copy manifest: {}", e)))?;

        // Copy library files if they exist
        for ext in &[".so", ".dylib", ".dll"] {
            let lib_path = source.join(format!("libext{}", ext));
            if lib_path.exists() {
                fs::copy(&lib_path, dest_dir.join(format!("libext{}", ext)))
                    .map_err(|e| QckerError::Internal(format!("Failed to copy library: {}", e)))?;
            }
        }

        // Add to extensions
        let info = ExtensionInfo {
            metadata: manifest.extension,
            status: ExtensionStatus::Loaded,
            path: dest_dir.to_string_lossy().to_string(),
        };

        self.extensions.insert(info.metadata.id.clone(), info);

        tracing::info!("Extension {} installed", ext_id);

        Ok(())
    }

    /// Uninstall an extension
    pub fn uninstall(&mut self, id: &str) -> Result<()> {
        if !self.extensions.contains_key(id) {
            return Err(QckerError::InvalidArgument(format!(
                "Extension not found: {}",
                id
            )));
        }

        let ext_dir = self.data_dir.join("extensions").join(id);
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir)
                .map_err(|e| QckerError::Internal(format!("Failed to remove extension: {}", e)))?;
        }

        self.extensions.remove(id);

        tracing::info!("Extension {} uninstalled", id);

        Ok(())
    }

    /// Enable an extension
    pub fn enable(&mut self, id: &str) -> Result<()> {
        if let Some(ext) = self.extensions.get_mut(id) {
            ext.status = ExtensionStatus::Active;
            tracing::info!("Extension {} enabled", id);
            Ok(())
        } else {
            Err(QckerError::InvalidArgument(format!(
                "Extension not found: {}",
                id
            )))
        }
    }

    /// Disable an extension
    pub fn disable(&mut self, id: &str) -> Result<()> {
        if let Some(ext) = self.extensions.get_mut(id) {
            ext.status = ExtensionStatus::Disabled;
            tracing::info!("Extension {} disabled", id);
            Ok(())
        } else {
            Err(QckerError::InvalidArgument(format!(
                "Extension not found: {}",
                id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extension_manager() {
        let tmp = TempDir::new().unwrap();
        let mut manager = ExtensionManager::new(tmp.path().to_path_buf());
        manager.init().unwrap();

        // Create a test extension
        let ext_dir = tmp.path().join("extensions").join("com.test.ext");
        fs::create_dir_all(&ext_dir).unwrap();

        let manifest = ExtensionManifest {
            extension: ExtensionMetadata {
                id: "com.test.ext".to_string(),
                name: "Test Extension".to_string(),
                version: "1.0.0".to_string(),
                api_version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "A test extension".to_string(),
                capabilities: vec![],
            },
            config: None,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(ext_dir.join("manifest.json"), manifest_json).unwrap();

        // Scan extensions
        manager.scan_extensions().unwrap();

        // List extensions
        let extensions = manager.list();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].metadata.id, "com.test.ext");
    }
}
