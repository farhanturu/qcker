use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub kernel_path: Option<PathBuf>,
    pub rootfs_path: Option<PathBuf>,
    pub vcpu_count: u32,
    pub memory_mb: u32,
    pub use_acceleration: bool,
    pub idle_timeout_secs: u64,
    pub kernel_cmdline_extra: Vec<String>,
    pub data_dir: PathBuf,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            kernel_path: None,
            rootfs_path: None,
            vcpu_count: 2,
            memory_mb: 512,
            use_acceleration: true,
            idle_timeout_secs: 60,
            kernel_cmdline_extra: Vec::new(),
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("qcker"),
        }
    }
}

impl BackendConfig {
    pub fn kernel_dir(&self) -> PathBuf {
        self.data_dir.join("kernel")
    }

    pub fn images_dir(&self) -> PathBuf {
        self.data_dir.join("images")
    }

    pub fn containers_dir(&self) -> PathBuf {
        self.data_dir.join("containers")
    }

    pub fn volumes_dir(&self) -> PathBuf {
        self.data_dir.join("volumes")
    }

    pub fn networks_dir(&self) -> PathBuf {
        self.data_dir.join("networks")
    }

    pub fn extensions_dir(&self) -> PathBuf {
        self.data_dir.join("extensions")
    }

    pub fn run_dir(&self) -> PathBuf {
        self.data_dir.join("run")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }
}
