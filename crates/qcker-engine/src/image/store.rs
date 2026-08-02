use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use qcker_common::error::{QckerError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub size: u64,
    pub layers: Vec<String>,
    pub config: ImageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub architecture: String,
    pub os: String,
    pub config: Option<ContainerConfig>,
    pub rootfs: RootFs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub cmd: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    pub r#type: String,
    pub diff_ids: Vec<String>,
}

pub struct ImageStore {
    pub data_dir: PathBuf,
}

impl ImageStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn images_dir(&self) -> PathBuf {
        self.data_dir.join("images")
    }

    fn layers_dir(&self) -> PathBuf {
        self.data_dir.join("layers")
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.images_dir())
            .map_err(|e| QckerError::internal(format!("Failed to create images dir: {}", e)))?;
        fs::create_dir_all(self.layers_dir())
            .map_err(|e| QckerError::internal(format!("Failed to create layers dir: {}", e)))?;
        Ok(())
    }

    pub fn store_image(&self, image: &Image) -> Result<()> {
        let image_dir = self.images_dir().join(&image.id);
        fs::create_dir_all(&image_dir)
            .map_err(|e| QckerError::internal(format!("Failed to create image dir: {}", e)))?;

        let meta_path = image_dir.join("manifest.json");
        let meta_json = serde_json::to_string_pretty(image)
            .map_err(|e| QckerError::internal(format!("Failed to serialize image: {}", e)))?;
        fs::write(&meta_path, meta_json)
            .map_err(|e| QckerError::internal(format!("Failed to write manifest: {}", e)))?;

        let refs_dir = image_dir.join("refs");
        fs::create_dir_all(&refs_dir)
            .map_err(|e| QckerError::internal(format!("Failed to create refs dir: {}", e)))?;

        for tag in &image.tags {
            let tag_file = refs_dir.join(tag.replace('/', "_"));
            fs::write(&tag_file, &image.id)
                .map_err(|e| QckerError::internal(format!("Failed to write tag ref: {}", e)))?;
        }

        Ok(())
    }

    pub fn get_image(&self, id_or_tag: &str) -> Result<Image> {
        let image_dir = self.images_dir().join(id_or_tag);
        if image_dir.exists() {
            return self.load_image(&image_dir);
        }

        for entry in fs::read_dir(self.images_dir())
            .map_err(|e| QckerError::internal(format!("Failed to read images dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::internal(format!("Failed to read entry: {}", e)))?;
            let refs_dir = entry.path().join("refs");
            if refs_dir.exists() {
                for tag_entry in fs::read_dir(&refs_dir)
                    .map_err(|e| QckerError::internal(format!("Failed to read refs: {}", e)))?
                {
                    let tag_entry = tag_entry
                        .map_err(|e| QckerError::internal(format!("Failed to read tag: {}", e)))?;
                    if tag_entry.file_name().to_string_lossy() == id_or_tag.replace('/', "_") {
                        return self.load_image(&entry.path());
                    }
                }
            }
        }

        Err(QckerError::image_not_found(id_or_tag.to_string()))
    }

    fn load_image(&self, image_dir: &Path) -> Result<Image> {
        let meta_path = image_dir.join("manifest.json");
        let meta_json = fs::read_to_string(&meta_path)
            .map_err(|e| QckerError::internal(format!("Failed to read manifest: {}", e)))?;
        let image: Image = serde_json::from_str(&meta_json)
            .map_err(|e| QckerError::internal(format!("Failed to parse manifest: {}", e)))?;
        Ok(image)
    }

    pub fn list_images(&self) -> Result<Vec<Image>> {
        let mut images = Vec::new();

        if !self.images_dir().exists() {
            return Ok(images);
        }

        for entry in fs::read_dir(self.images_dir())
            .map_err(|e| QckerError::internal(format!("Failed to read images dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::internal(format!("Failed to read entry: {}", e)))?;
            if entry.path().is_dir() {
                if let Ok(image) = self.load_image(&entry.path()) {
                    images.push(image);
                }
            }
        }

        Ok(images)
    }

    pub fn remove_image(&self, id: &str) -> Result<()> {
        let image_dir = self.images_dir().join(id);
        if !image_dir.exists() {
            return Err(QckerError::image_not_found(id.to_string()));
        }

        fs::remove_dir_all(&image_dir)
            .map_err(|e| QckerError::internal(format!("Failed to remove image: {}", e)))?;

        Ok(())
    }

    pub fn image_exists(&self, id: &str) -> bool {
        self.images_dir().join(id).exists()
    }
}

