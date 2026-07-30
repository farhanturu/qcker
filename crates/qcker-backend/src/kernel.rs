use std::path::{Path, PathBuf};

use qcker_common::error::{QckerError, Result};
use sha2::{Digest, Sha256};

pub struct KernelManager {
    data_dir: PathBuf,
}

impl KernelManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn kernel_path(&self) -> PathBuf {
        let arch = std::env::consts::ARCH;
        self.data_dir.join("kernel").join(format!("vmlinuz-{}", arch))
    }

    pub fn is_cached(&self) -> bool {
        self.kernel_path().exists()
    }

    pub fn get_or_download(&self) -> Result<PathBuf> {
        let path = self.kernel_path();
        if path.exists() {
            return Ok(path);
        }

        Err(QckerError::Internal(
            "Kernel not found. Download from https://github.com/farhanturu/qcker/releases".to_string()
        ))
    }

    pub fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool> {
        let data = std::fs::read(path)
            .map_err(|e| QckerError::Internal(format!("Failed to read kernel: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let actual = format!("{:x}", hasher.finalize());

        Ok(actual == expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_path() {
        let manager = KernelManager::new(PathBuf::from("/tmp/qcker"));
        let path = manager.kernel_path();
        assert!(path.to_string_lossy().contains("vmlinuz"));
    }

    #[test]
    fn test_not_cached() {
        let manager = KernelManager::new(PathBuf::from("/nonexistent"));
        assert!(!manager.is_cached());
    }
}
