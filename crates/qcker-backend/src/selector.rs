use qcker_common::error::{QckerError, Result};

use crate::microvm::MicroVmBackend;
use crate::native::NativeBackend;
use crate::RuntimeBackend;

pub fn select_backend(override_backend: Option<&str>) -> Result<Box<dyn RuntimeBackend>> {
    if let Some(name) = override_backend {
        return match name {
            "native" => Ok(Box::new(NativeBackend::new())),
            "microvm" => Ok(Box::new(MicroVmBackend::new())),
            _ => Err(QckerError::internal(format!("Unknown backend: {}", name))),
        };
    }

    if cfg!(target_os = "linux") {
        let native = NativeBackend::new();
        if native.is_available() {
            tracing::info!("Selected native backend (Linux direct)");
            return Ok(Box::new(native));
        }
    }

    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        let microvm = MicroVmBackend::new();
        if microvm.is_available() {
            tracing::info!("Selected MicroVM backend");
            return Ok(Box::new(microvm));
        }
    }

    let microvm = MicroVmBackend::new();
    if microvm.is_available() {
        return Ok(Box::new(microvm));
    }

    Err(QckerError::internal("No runtime backend available".to_string()))
}
