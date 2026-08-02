use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

pub struct FsShareConfig {
    pub host_path: PathBuf,
    pub mount_tag: String,
    pub read_only: bool,
}

impl FsShareConfig {
    pub fn new(host_path: PathBuf, mount_tag: String, read_only: bool) -> Self {
        Self {
            host_path,
            mount_tag,
            read_only,
        }
    }

    pub fn to_qemu_args(&self) -> Vec<String> {
        vec![
            "-fsdev".to_string(),
            format!(
                "local,id={},path={},security_model=mapped-xattr{}",
                self.mount_tag,
                self.host_path.display(),
                if self.read_only { ",readonly" } else { "" }
            ),
            "-device".to_string(),
            format!(
                "virtio-9p-pci,fsdev={},mount_tag={}",
                self.mount_tag, self.mount_tag
            ),
        ]
    }
}

pub fn create_bind_mount_spec(
    host_path: &str,
    _container_path: &str,
    read_only: bool,
) -> Result<FsShareConfig> {
    let host = PathBuf::from(host_path);
    if !host.exists() {
        return Err(QckerError::invalid_argument(format!(
            "Host path does not exist: {}",
            host_path
        )));
    }

    let tag = format!("mnt-{}", md5_short(host_path));
    Ok(FsShareConfig::new(host, tag, read_only))
}

fn md5_short(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    hash[..8].to_string()
}

