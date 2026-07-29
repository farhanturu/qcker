use std::fs;
use std::path::{Path, PathBuf};

use qcker_common::error::{QckerError, Result};
use qcker_common::hash::sha256_file;

/// Layer metadata
#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: String,
    pub size: u64,
    pub path: PathBuf,
}

/// Layer manager for content-addressable storage
pub struct LayerManager {
    pub layers_dir: PathBuf,
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new(layers_dir: PathBuf) -> Self {
        Self { layers_dir }
    }

    /// Initialize the layer manager
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.layers_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create layers dir: {}", e)))?;
        Ok(())
    }

    /// Store a layer from a tar.gz file
    pub fn store_layer(&self, tar_path: &Path) -> Result<Layer> {
        // Compute digest
        let digest = sha256_file(tar_path)?;
        let digest_str = format!("sha256:{}", digest);

        // Get file size
        let metadata = fs::metadata(tar_path)
            .map_err(|e| QckerError::Internal(format!("Failed to get file metadata: {}", e)))?;
        let size = metadata.len();

        // Create layer directory
        let layer_dir = self.layers_dir.join(&digest);
        fs::create_dir_all(&layer_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create layer dir: {}", e)))?;

        // Copy tar file
        let layer_file = layer_dir.join("layer.tar.gz");
        fs::copy(tar_path, &layer_file)
            .map_err(|e| QckerError::Internal(format!("Failed to copy layer: {}", e)))?;

        // Extract layer
        let extract_dir = layer_dir.join("layer");
        fs::create_dir_all(&extract_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create extract dir: {}", e)))?;
        qcker_common::tar::extract_tar_gz(tar_path, &extract_dir)?;

        Ok(Layer {
            digest: digest_str,
            size,
            path: layer_dir,
        })
    }

    /// Get layer by digest
    pub fn get_layer(&self, digest: &str) -> Result<Layer> {
        // Remove "sha256:" prefix if present
        let hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        let layer_dir = self.layers_dir.join(hash);

        if !layer_dir.exists() {
            return Err(QckerError::Internal(format!("Layer not found: {}", digest)));
        }

        let layer_file = layer_dir.join("layer.tar.gz");
        let metadata = fs::metadata(&layer_file)
            .map_err(|e| QckerError::Internal(format!("Failed to get layer metadata: {}", e)))?;

        Ok(Layer {
            digest: digest.to_string(),
            size: metadata.len(),
            path: layer_dir,
        })
    }

    /// Check if layer exists
    pub fn layer_exists(&self, digest: &str) -> bool {
        let hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        self.layers_dir.join(hash).exists()
    }

    /// Get layer path for mounting
    pub fn get_layer_path(&self, digest: &str) -> Result<PathBuf> {
        let hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        let layer_dir = self.layers_dir.join(hash).join("layer");

        if !layer_dir.exists() {
            return Err(QckerError::Internal(format!("Layer not found: {}", digest)));
        }

        Ok(layer_dir)
    }

    /// List all layers
    pub fn list_layers(&self) -> Result<Vec<Layer>> {
        let mut layers = Vec::new();

        if !self.layers_dir.exists() {
            return Ok(layers);
        }

        for entry in fs::read_dir(&self.layers_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to read layers dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            if entry.path().is_dir() {
                let layer_file = entry.path().join("layer.tar.gz");
                if layer_file.exists() {
                    let metadata = fs::metadata(&layer_file)
                        .map_err(|e| QckerError::Internal(format!("Failed to get metadata: {}", e)))?;
                    layers.push(Layer {
                        digest: format!("sha256:{}", entry.file_name().to_string_lossy()),
                        size: metadata.len(),
                        path: entry.path(),
                    });
                }
            }
        }

        Ok(layers)
    }

    /// Remove a layer
    pub fn remove_layer(&self, digest: &str) -> Result<()> {
        let hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        let layer_dir = self.layers_dir.join(hash);

        if !layer_dir.exists() {
            return Err(QckerError::Internal(format!("Layer not found: {}", digest)));
        }

        fs::remove_dir_all(&layer_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to remove layer: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_layer_manager() {
        let tmp = TempDir::new().unwrap();
        let manager = LayerManager::new(tmp.path().to_path_buf());
        manager.init().unwrap();

        // Create a test tar.gz
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("test.txt"), "hello").unwrap();

        let tar_path = tmp.path().join("test.tar.gz");
        qcker_common::tar::create_tar_gz(&src_dir, &tar_path).unwrap();

        // Store layer
        let layer = manager.store_layer(&tar_path).unwrap();
        assert!(layer.digest.starts_with("sha256:"));

        // Get layer
        let loaded = manager.get_layer(&layer.digest).unwrap();
        assert_eq!(loaded.digest, layer.digest);

        // List layers
        let layers = manager.list_layers().unwrap();
        assert_eq!(layers.len(), 1);

        // Remove layer
        manager.remove_layer(&layer.digest).unwrap();
        assert!(!manager.layer_exists(&layer.digest));
    }
}
