use std::path::PathBuf;
use std::process::{Command, Stdio};

use qcker_common::error::{QckerError, Result};
use tracing::{debug, info, warn};

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
    config: VmmConfig,
    log_path: Option<PathBuf>,
}

impl VmmManager {
    pub fn start(config: VmmConfig) -> Result<Self> {
        info!(
            "Starting QEMU MicroVM: vcpu={}, mem={}MB, cid={}",
            config.vcpu_count, config.memory_mb, config.vsock_cid
        );

        let process = Self::start_qemu(&config)?;

        Ok(Self {
            process: Some(process),
            config,
            log_path: None,
        })
    }

    pub fn start_with_log(config: VmmConfig, log_path: PathBuf) -> Result<Self> {
        info!(
            "Starting QEMU MicroVM with log: vcpu={}, mem={}MB, cid={}, log={}",
            config.vcpu_count, config.memory_mb, config.vsock_cid,
            log_path.display()
        );

        let process = Self::start_qemu_with_log(&config, &log_path)?;

        Ok(Self {
            process: Some(process),
            config,
            log_path: Some(log_path),
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
            "pci=off".to_string(),
        ];
        cmdline.extend(config.kernel_cmdline_extra.clone());
        cmd.arg("-append").arg(cmdline.join(" "));

        cmd.arg("-m").arg(config.memory_mb.to_string());
        cmd.arg("-smp").arg(config.vcpu_count.to_string());

        cmd.arg("-nodefaults");
        cmd.arg("-no-reboot");
        cmd.arg("-display").arg("none");

        cmd.arg("-device").arg(format!(
            "vhost-vsock-pci,guest-cid={}",
            config.vsock_cid
        ));

        cmd.arg("-serial").arg("stdio");

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

        debug!("QEMU command: {:?}", cmd);

        cmd.spawn()
            .map_err(|e| QckerError::internal(format!("Failed to start QEMU: {}", e)))
    }

    fn start_qemu_with_log(config: &VmmConfig, log_path: &PathBuf) -> Result<std::process::Child> {
        let _log_file = std::fs::File::create(log_path)
            .map_err(|e| QckerError::internal(format!("Failed to create log file: {}", e)))?;

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
            "pci=off".to_string(),
        ];
        cmdline.extend(config.kernel_cmdline_extra.clone());
        cmd.arg("-append").arg(cmdline.join(" "));

        cmd.arg("-m").arg(config.memory_mb.to_string());
        cmd.arg("-smp").arg(config.vcpu_count.to_string());

        cmd.arg("-nodefaults");
        cmd.arg("-no-reboot");
        cmd.arg("-display").arg("none");

        cmd.arg("-device").arg(format!(
            "vhost-vsock-pci,guest-cid={}",
            config.vsock_cid
        ));

        cmd.arg("-serial").arg(format!("file:{}", log_path.display()));

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

        debug!("QEMU command (with log): {:?}", cmd);

        cmd.spawn()
            .map_err(|e| QckerError::internal(format!("Failed to start QEMU: {}", e)))
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping QEMU MicroVM (PID: {:?})", child.id());
            let _ = child.kill();
            match child.wait() {
                Ok(status) => {
                    info!("QEMU exited with status: {}", status);
                }
                Err(e) => {
                    warn!("Error waiting for QEMU to exit: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    debug!("QEMU process exited with: {}", status);
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    warn!("Error checking QEMU process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|c| c.id())
    }

    pub fn vsock_cid(&self) -> u32 {
        self.config.vsock_cid
    }

    pub fn read_log(&self) -> Option<String> {
        if let Some(ref log_path) = self.log_path {
            if log_path.exists() {
                return std::fs::read_to_string(log_path).ok();
            }
        }
        None
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

pub fn check_kvm_available() -> bool {
    std::path::Path::new("/dev/kvm").exists()
}

pub fn get_qemu_version() -> Option<String> {
    let output = Command::new("qemu-system-x86_64")
        .arg("--version")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_qemu() {
        let available = check_qemu_available();
        println!("QEMU available: {}", available);
    }

    #[test]
    fn test_check_kvm() {
        let available = check_kvm_available();
        println!("KVM available: {}", available);
    }

    #[test]
    fn test_get_qemu_version() {
        if let Some(version) = get_qemu_version() {
            println!("QEMU version: {}", version);
            assert!(version.contains("QEMU"));
        }
    }

    #[test]
    fn test_vmm_config() {
        let config = VmmConfig {
            kernel_path: PathBuf::from("/tmp/vmlinuz"),
            rootfs_path: PathBuf::from("/tmp/initramfs.cpio.gz"),
            vcpu_count: 2,
            memory_mb: 512,
            use_acceleration: true,
            kernel_cmdline_extra: vec![],
            vsock_cid: 42,
            fs_shares: vec![],
        };
        assert_eq!(config.vsock_cid, 42);
        assert_eq!(config.vcpu_count, 2);
    }
}
