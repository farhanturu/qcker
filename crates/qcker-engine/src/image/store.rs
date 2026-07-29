use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use qcker_common::error::{QckerError, Result};

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub size: u64,
    pub layers: Vec<String>,
    pub config: ImageConfig,
}

/// Image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub architecture: String,
    pub os: String,
    pub config: Option<ContainerConfig>,
    pub rootfs: RootFs,
}

/// Container configuration from image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub cmd: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
}

/// Root filesystem type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    pub r#type: String,
    pub diff_ids: Vec<String>,
}

/// Local image store
pub struct ImageStore {
    pub data_dir: PathBuf,
}

impl ImageStore {
    /// Create a new image store
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Get images directory
    fn images_dir(&self) -> PathBuf {
        self.data_dir.join("images")
    }

    /// Get layers directory
    fn layers_dir(&self) -> PathBuf {
        self.data_dir.join("layers")
    }

    /// Initialize the store
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.images_dir())
            .map_err(|e| QckerError::Internal(format!("Failed to create images dir: {}", e)))?;
        fs::create_dir_all(self.layers_dir())
            .map_err(|e| QckerError::Internal(format!("Failed to create layers dir: {}", e)))?;
        Ok(())
    }

    /// Store an image
    pub fn store_image(&self, image: &Image) -> Result<()> {
        let image_dir = self.images_dir().join(&image.id);
        fs::create_dir_all(&image_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create image dir: {}", e)))?;

        // Save image metadata
        let meta_path = image_dir.join("manifest.json");
        let meta_json = serde_json::to_string_pretty(image)
            .map_err(|e| QckerError::Internal(format!("Failed to serialize image: {}", e)))?;
        fs::write(&meta_path, meta_json)
            .map_err(|e| QckerError::Internal(format!("Failed to write manifest: {}", e)))?;

        // Create tag references
        let refs_dir = image_dir.join("refs");
        fs::create_dir_all(&refs_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create refs dir: {}", e)))?;

        for tag in &image.tags {
            let tag_file = refs_dir.join(tag.replace('/', "_"));
            fs::write(&tag_file, &image.id)
                .map_err(|e| QckerError::Internal(format!("Failed to write tag ref: {}", e)))?;
        }

        Ok(())
    }

    /// Get image by ID or tag
    pub fn get_image(&self, id_or_tag: &str) -> Result<Image> {
        // Try as ID first
        let image_dir = self.images_dir().join(id_or_tag);
        if image_dir.exists() {
            return self.load_image(&image_dir);
        }

        // Try as tag - search all images
        for entry in fs::read_dir(self.images_dir())
            .map_err(|e| QckerError::Internal(format!("Failed to read images dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            let refs_dir = entry.path().join("refs");
            if refs_dir.exists() {
                for tag_entry in fs::read_dir(&refs_dir)
                    .map_err(|e| QckerError::Internal(format!("Failed to read refs: {}", e)))?
                {
                    let tag_entry = tag_entry
                        .map_err(|e| QckerError::Internal(format!("Failed to read tag: {}", e)))?;
                    if tag_entry.file_name().to_string_lossy() == id_or_tag.replace('/', "_") {
                        return self.load_image(&entry.path());
                    }
                }
            }
        }

        Err(QckerError::ImageNotFound(id_or_tag.to_string()))
    }

    /// Load image from directory
    fn load_image(&self, image_dir: &Path) -> Result<Image> {
        let meta_path = image_dir.join("manifest.json");
        let meta_json = fs::read_to_string(&meta_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read manifest: {}", e)))?;
        let image: Image = serde_json::from_str(&meta_json)
            .map_err(|e| QckerError::Internal(format!("Failed to parse manifest: {}", e)))?;
        Ok(image)
    }

    /// List all images
    pub fn list_images(&self) -> Result<Vec<Image>> {
        let mut images = Vec::new();

        if !self.images_dir().exists() {
            return Ok(images);
        }

        for entry in fs::read_dir(self.images_dir())
            .map_err(|e| QckerError::Internal(format!("Failed to read images dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            if entry.path().is_dir() {
                if let Ok(image) = self.load_image(&entry.path()) {
                    images.push(image);
                }
            }
        }

        Ok(images)
    }

    /// Remove an image
    pub fn remove_image(&self, id: &str) -> Result<()> {
        let image_dir = self.images_dir().join(id);
        if !image_dir.exists() {
            return Err(QckerError::ImageNotFound(id.to_string()));
        }

        fs::remove_dir_all(&image_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to remove image: {}", e)))?;

        Ok(())
    }

    /// Check if image exists
    pub fn image_exists(&self, id: &str) -> bool {
        self.images_dir().join(id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_image_store() {
        let tmp = TempDir::new().unwrap();
        let store = ImageStore::new(tmp.path().to_path_buf());
        store.init().unwrap();

        let image = Image {
            id: "test123".to_string(),
            tags: vec!["latest".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            size: 1024,
            layers: vec!["layer1".to_string()],
            config: ImageConfig {
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                config: None,
                rootfs: RootFs {
                    r#type: "layers".to_string(),
                    diff_ids: vec!["sha256:abc123".to_string()],
                },
            },
        };

        // Store image
        store.store_image(&image).unwrap();

        // Get by ID
        let loaded = store.get_image("test123").unwrap();
        assert_eq!(loaded.id, "test123");

        // Get by tag
        let loaded = store.get_image("latest").unwrap();
        assert_eq!(loaded.id, "test123");

        // List images
        let images = store.list_images().unwrap();
        assert_eq!(images.len(), 1);

        // Remove image
        store.remove_image("test123").unwrap();
        assert!(!store.image_exists("test123"));
    }
}
