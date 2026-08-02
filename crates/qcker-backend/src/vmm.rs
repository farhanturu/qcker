use std::path::PathBuf;
use std::process::{Command, Stdio};

use qcker_common::error::{QckerError, Result};

pub struct VmmConfig {
    pub kernel_path: PathBuf,
    pub rootfs_path: PathBuf,
    pub vcpu_count: u32,
    pub memory_mb: u32,
    pub use_acceleration: bool,
    pub kernel_cmdline_extra: Vec<String>,
    pub vsock_cid: u32,
    pub fs_shares: Vec<FsShareConfig>,
}

pub struct FsShareConfig {
    pub host_path: PathBuf,
    pub mount_tag: String,
}

pub struct VmmManager {
    process: Option<std::process::Child>,
    _config: VmmConfig,
}

impl VmmManager {
    pub fn start(config: VmmConfig) -> Result<Self> {
        let process = Self::start_qemu(&config)?;

        Ok(Self {
            process: Some(process),
            _config: config,
        })
    }

    fn start_qemu(config: &VmmConfig) -> Result<std::process::Child> {
        let mut cmd = Command::new("qemu-system-x86_64");

        cmd.arg("-machine").arg("microvm");

        if config.use_acceleration {
            if cfg!(target_os = "linux") {
                cmd.arg("-accel").arg("kvm");
            } else if cfg!(target_os = "macos") {
                cmd.arg("-accel").arg("hvf");
            }
        }

        cmd.arg("-kernel").arg(&config.kernel_path);
        cmd.arg("-initrd").arg(&config.rootfs_path);

        let mut cmdline = vec![
            "console=hvc0".to_string(),
            "quiet".to_string(),
            "panic=1".to_string(),
        ];
        cmdline.extend(config.kernel_cmdline_extra.clone());
        cmd.arg("-append").arg(cmdline.join(" "));

        cmd.arg("-m").arg(config.memory_mb.to_string());
        cmd.arg("-smp").arg(config.vcpu_count.to_string());

        cmd.arg("-nodefaults");
        cmd.arg("-no-reboot");

        cmd.arg("-device").arg(format!(
            "vhost-vsock-pci,guest-cid={}",
            config.vsock_cid
        ));

        for share in &config.fs_shares {
            cmd.arg("-fsdev").arg(format!(
                "local,id={},path={},security_model=mapped-xattr",
                share.mount_tag,
                share.host_path.display()
            ));
            cmd.arg("-device").arg(format!(
                "virtio-9p-pci,fsdev={},mount_tag={}",
                share.mount_tag, share.mount_tag
            ));
        }

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        cmd.spawn()
            .map_err(|e| QckerError::internal(format!("Failed to start QEMU: {}", e)))
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Drop for VmmManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub fn check_qemu_available() -> bool {
    Command::new("qemu-system-x86_64")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_qemu() {
        let available = check_qemu_available();
        println!("QEMU available: {}", available);
    }
}
