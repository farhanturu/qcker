use std::fs;
use std::path::{Path, PathBuf};

use nix::mount::{mount, MsFlags};
use qcker_common::error::{QckerError, Result};

use crate::mount as mount_module;

pub struct RootfsConfig {
    pub container_dir: PathBuf,
    pub layers: Vec<PathBuf>,
    pub rootless: bool,
    pub skip_mounts: bool,
    pub hostname: Option<String>,
    pub dns_servers: Vec<String>,
}

impl Default for RootfsConfig {
    fn default() -> Self {
        Self {
            container_dir: PathBuf::new(),
            layers: Vec::new(),
            rootless: false,
            skip_mounts: true,
            hostname: None,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
        }
    }
}

pub fn create_rootfs(config: &RootfsConfig) -> Result<PathBuf> {
    let container_dir = &config.container_dir;
    let rootfs_dir = container_dir.join("rootfs");
    let upper_dir = container_dir.join("upper");
    let work_dir = container_dir.join("work");

    fs::create_dir_all(&rootfs_dir)
        .map_err(|e| QckerError::Mount(format!("Failed to create rootfs dir: {}", e)))?;
    fs::create_dir_all(&upper_dir)
        .map_err(|e| QckerError::Mount(format!("Failed to create upper dir: {}", e)))?;
    fs::create_dir_all(&work_dir)
        .map_err(|e| QckerError::Mount(format!("Failed to create work dir: {}", e)))?;

    if !config.layers.is_empty() {
        let lower_refs: Vec<&Path> = config.layers.iter().map(|p| p.as_path()).collect();
        mount_module::setup_rootfs(
            &lower_refs,
            &upper_dir,
            &work_dir,
            &rootfs_dir,
            config.rootless,
        )?;
    }

    create_essential_dirs(&rootfs_dir)?;
    create_essential_files(&rootfs_dir, config)?;

    Ok(rootfs_dir)
}

fn create_essential_dirs(rootfs: &Path) -> Result<()> {
    let dirs = ["proc", "sys", "dev", "tmp", "etc", "var", "run", "root", "home"];
    for dir in &dirs {
        let path = rootfs.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| QckerError::Mount(format!("Failed to create {}: {}", dir, e)))?;
    }
    Ok(())
}

fn create_essential_files(rootfs: &Path, config: &RootfsConfig) -> Result<()> {
    // DNS
    let resolv_conf = rootfs.join("etc/resolv.conf");
    let dns_content: String = config.dns_servers.iter()
        .map(|s| format!("nameserver {}\n", s))
        .collect();
    fs::write(&resolv_conf, dns_content)
        .map_err(|e| QckerError::Mount(format!("Failed to create resolv.conf: {}", e)))?;

    // Hostname
    let hostname = config.hostname.as_deref().unwrap_or("container");
    let hostname_file = rootfs.join("etc/hostname");
    fs::write(&hostname_file, format!("{}\n", hostname))
        .map_err(|e| QckerError::Mount(format!("Failed to create hostname: {}", e)))?;

    // Hosts
    let hosts = rootfs.join("etc/hosts");
    fs::write(&hosts, format!("127.0.0.1 localhost\n::1 localhost\n127.0.0.1 {}\n", hostname))
        .map_err(|e| QckerError::Mount(format!("Failed to create hosts: {}", e)))?;

    Ok(())
}

pub fn extract_layers(layers: &[PathBuf], dest: &Path) -> Result<()> {
    for layer in layers {
        qcker_common::tar::extract_tar_gz(layer, dest)?;
    }
    Ok(())
}

pub fn enter_rootfs(rootfs: &Path, _rootless: bool) -> Result<()> {
    use nix::unistd::chroot;

    chroot(rootfs)
        .map_err(|e| QckerError::Mount(format!("Failed to chroot: {}", e)))?;
    std::env::set_current_dir("/")
        .map_err(|e| QckerError::Mount(format!("Failed to chdir to /: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rootfs_config_default() {
        let config = RootfsConfig::default();
        assert!(config.skip_mounts);
        assert_eq!(config.dns_servers, vec!["8.8.8.8", "8.8.4.4"]);
    }

    #[test]
    fn test_rootfs_config() {
        let config = RootfsConfig {
            container_dir: PathBuf::from("/tmp/test"),
            layers: vec![],
            rootless: false,
            skip_mounts: true,
            hostname: Some("myhost".to_string()),
            dns_servers: vec!["1.1.1.1".to_string()],
        };
        assert_eq!(config.hostname, Some("myhost".to_string()));
        assert_eq!(config.dns_servers, vec!["1.1.1.1"]);
    }
}
