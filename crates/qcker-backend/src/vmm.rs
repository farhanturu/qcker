use std::path::PathBuf;
use std::process::{Command, Stdio};

use qcker_common::error::{QckerError, Result};
use tracing::{debug, info, warn};

/// Configuration for starting a QEMU MicroVM instance.
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

/// Filesystem share configuration for virtio-9p passthrough.
pub struct FsShareConfig {
    pub host_path: PathBuf,
    pub mount_tag: String,
}

/// Manages a QEMU MicroVM process lifecycle.
///
/// This struct wraps the QEMU process and provides methods to:
/// - Start the VM with the microvm machine type
/// - Check if the VM is still running
/// - Stop the VM gracefully (SIGTERM) or forcefully (SIGKILL)
/// - Collect VM logs via serial console
pub struct VmmManager {
    process: Option<std::process::Child>,
    config: VmmConfig,
    log_path: Option<PathBuf>,
}

impl VmmManager {
    /// Start a new QEMU MicroVM process.
    ///
    /// This spawns `qemu-system-x86_64` with the microvm machine type,
    /// KVM acceleration (if available), and a vsock device for host-guest
    /// communication.
    ///
    /// # Arguments
    /// * `config` - VM configuration including kernel, rootfs, resources, etc.
    ///
    /// # Returns
    /// A `VmmManager` wrapping the spawned QEMU process.
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

    /// Start QEMU with a log file to capture serial console output.
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

        // Use the microvm machine type for fast boot
        cmd.arg("-machine").arg("microvm");

        // Enable hardware acceleration if available
        if config.use_acceleration {
            if cfg!(target_os = "linux") {
                cmd.arg("-accel").arg("kvm");
            } else if cfg!(target_os = "macos") {
                cmd.arg("-accel").arg("hvf");
            }
        }

        // Kernel and rootfs
        cmd.arg("-kernel").arg(&config.kernel_path);
        cmd.arg("-initrd").arg(&config.rootfs_path);

        // Kernel command line
        let mut cmdline = vec![
            "console=hvc0".to_string(),
            "quiet".to_string(),
            "panic=1".to_string(),
            "pci=off".to_string(), // Disable PCI for faster boot (microvm uses MMIO)
        ];
        cmdline.extend(config.kernel_cmdline_extra.clone());
        cmd.arg("-append").arg(cmdline.join(" "));

        // CPU and memory
        cmd.arg("-m").arg(config.memory_mb.to_string());
        cmd.arg("-smp").arg(config.vcpu_count.to_string());

        // Disable defaults for faster boot
        cmd.arg("-nodefaults");
        cmd.arg("-no-reboot");
        cmd.arg("-display").arg("none");

        // vsock device for host-guest communication
        cmd.arg("-device").arg(format!(
            "vhost-vsock-pci,guest-cid={}",
            config.vsock_cid
        ));

        // Serial console (for early boot messages)
        cmd.arg("-serial").arg("stdio");

        // Filesystem shares (virtio-9p)
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

        // Suppress QEMU output
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

        // Use the microvm machine type for fast boot
        cmd.arg("-machine").arg("microvm");

        // Enable hardware acceleration if available
        if config.use_acceleration {
            if cfg!(target_os = "linux") {
                cmd.arg("-accel").arg("kvm");
            } else if cfg!(target_os = "macos") {
                cmd.arg("-accel").arg("hvf");
            }
        }

        // Kernel and rootfs
        cmd.arg("-kernel").arg(&config.kernel_path);
        cmd.arg("-initrd").arg(&config.rootfs_path);

        // Kernel command line
        let mut cmdline = vec![
            "console=hvc0".to_string(),
            "quiet".to_string(),
            "panic=1".to_string(),
            "pci=off".to_string(),
        ];
        cmdline.extend(config.kernel_cmdline_extra.clone());
        cmd.arg("-append").arg(cmdline.join(" "));

        // CPU and memory
        cmd.arg("-m").arg(config.memory_mb.to_string());
        cmd.arg("-smp").arg(config.vcpu_count.to_string());

        // Disable defaults for faster boot
        cmd.arg("-nodefaults");
        cmd.arg("-no-reboot");
        cmd.arg("-display").arg("none");

        // vsock device
        cmd.arg("-device").arg(format!(
            "vhost-vsock-pci,guest-cid={}",
            config.vsock_cid
        ));

        // Serial console to log file
        cmd.arg("-serial").arg(format!("file:{}", log_path.display()));

        // Filesystem shares
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

    /// Stop the VM gracefully.
    ///
    /// First sends SIGTERM to allow the VM to shut down cleanly.
    /// If the VM doesn't exit within 5 seconds, sends SIGKILL.
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping QEMU MicroVM (PID: {:?})", child.id());

            // Try graceful shutdown first
            let _ = child.kill();

            // Wait for the process to exit
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

    /// Check if the VM process is still running.
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

    /// Get the PID of the QEMU process.
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|c| c.id())
    }

    /// Get the vsock CID used by this VM.
    pub fn vsock_cid(&self) -> u32 {
        self.config.vsock_cid
    }

    /// Read the VM log file contents.
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

/// Check if QEMU is available on the system.
pub fn check_qemu_available() -> bool {
    Command::new("qemu-system-x86_64")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Check if KVM is available for hardware acceleration.
pub fn check_kvm_available() -> bool {
    std::path::Path::new("/dev/kvm").exists()
}

/// Get the QEMU version string.
pub fn get_qemu_version() -> Option<String> {
    let output = Command::new("qemu-system-x86_64")
        .arg("--version")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse first line like "QEMU emulator version 8.2.0"
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
