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
    create_essential_files(&rootfs_dir)?;

    if !config.skip_mounts {
        let _ = bind_mount_host_dirs(&rootfs_dir);
    }

    Ok(rootfs_dir)
}

fn bind_mount_host_dirs(rootfs: &Path) -> Result<()> {
    let dirs_to_mount = vec!["lib", "lib64", "usr/lib", "usr/lib64", "bin", "sbin", "usr/bin", "usr/sbin"];

    for dir in dirs_to_mount {
        let host_path = PathBuf::from("/").join(dir);
        let container_path = rootfs.join(dir);

        if host_path.exists() {
            fs::create_dir_all(&container_path)
                .map_err(|e| QckerError::Mount(format!("Failed to create {}: {}", dir, e)))?;

            match mount(
                Some(&host_path),
                &container_path,
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC | MsFlags::MS_RDONLY,
                None::<&str>,
            ) {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Failed to mount {}: {}", dir, e);
                }
            }
        }
    }

    Ok(())
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

fn create_essential_files(rootfs: &Path) -> Result<()> {
    let resolv_conf = rootfs.join("etc/resolv.conf");
    fs::write(&resolv_conf, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n")
        .map_err(|e| QckerError::Mount(format!("Failed to create resolv.conf: {}", e)))?;

    let hostname = rootfs.join("etc/hostname");
    fs::write(&hostname, "container\n")
        .map_err(|e| QckerError::Mount(format!("Failed to create hostname: {}", e)))?;

    let hosts = rootfs.join("etc/hosts");
    fs::write(&hosts, "127.0.0.1 localhost\n::1 localhost\n127.0.0.1 container\n")
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
    fn test_rootfs_config() {
        let config = RootfsConfig {
            container_dir: PathBuf::from("/tmp/test"),
            layers: vec![],
            rootless: false,
            skip_mounts: false,
        };
        assert_eq!(config.container_dir, PathBuf::from("/tmp/test"));
        assert!(config.layers.is_empty());
    }
}
