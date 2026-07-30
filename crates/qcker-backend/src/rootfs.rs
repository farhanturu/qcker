use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

pub struct RootfsManager {
    data_dir: PathBuf,
}

impl RootfsManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn rootfs_path(&self) -> PathBuf {
        self.data_dir.join("kernel").join("initramfs.cpio.gz")
    }

    pub fn is_cached(&self) -> bool {
        self.rootfs_path().exists()
    }

    pub fn get_or_build(&self) -> Result<PathBuf> {
        let path = self.rootfs_path();
        if path.exists() {
            return Ok(path);
        }

        Err(QckerError::Internal(
            "Rootfs not found. Build with: cargo build --release --target x86_64-unknown-linux-musl".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rootfs_path() {
        let manager = RootfsManager::new(PathBuf::from("/tmp/qcker"));
        let path = manager.rootfs_path();
        assert!(path.to_string_lossy().contains("initramfs"));
    }
}
