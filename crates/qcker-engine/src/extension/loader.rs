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
            return Err(QckerError::InvalidArgument(format!(
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

        Err(QckerError::InvalidArgument(
            "No library file found in extension directory".to_string(),
        ))
    }
}

pub struct LoadedExtension {
    pub id: String,
    pub path: PathBuf,
    pub loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extension_loader() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = tmp.path().join("extensions").join("com.test.ext");
        fs::create_dir_all(&ext_dir).unwrap();

        fs::write(ext_dir.join("libext.so"), "dummy").unwrap();

        let loader = ExtensionLoader::new(tmp.path().join("extensions"));
        let loaded = loader.load("com.test.ext").unwrap();

        assert_eq!(loaded.id, "com.test.ext");
        assert!(loaded.loaded);
    }
}
