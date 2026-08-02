use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

pub struct ExtensionLoader {
    pub extensions_dir: PathBuf,
}

impl ExtensionLoader {
    pub fn new(extensions_dir: PathBuf) -> Self {
        Self { extensions_dir }
    }

    pub fn load(&self, extension_id: &str) -> Result<LoadedExtension> {
        let ext_dir = self.extensions_dir.join(extension_id);

        if !ext_dir.exists() {
            return Err(QckerError::invalid_argument(format!(
                "Extension not found: {}",
                extension_id
            )));
        }

        let lib_path = self.find_library(&ext_dir)?;

        tracing::info!("Loading extension {} from {:?}", extension_id, lib_path);

        Ok(LoadedExtension {
            id: extension_id.to_string(),
            path: lib_path,
            loaded: true,
        })
    }

    fn find_library(&self, ext_dir: &PathBuf) -> Result<PathBuf> {
        for ext in &[".so", ".dylib", ".dll"] {
            let lib_path = ext_dir.join(format!("libext{}", ext));
            if lib_path.exists() {
                return Ok(lib_path);
            }
        }

        Err(QckerError::invalid_argument(
            "No library file found in extension directory".to_string(),
        ))
    }
}

pub struct LoadedExtension {
    pub id: String,
    pub path: PathBuf,
    pub loaded: bool,
}

