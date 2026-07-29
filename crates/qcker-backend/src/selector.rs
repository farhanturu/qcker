use crate::native::NativeBackend;
use crate::RuntimeBackend;

pub fn select_backend(override_backend: Option<&str>) -> Result<Box<dyn RuntimeBackend>, String> {
    if let Some(name) = override_backend {
        return match name {
            "native" => Ok(Box::new(NativeBackend::new())),
            _ => Err(format!("Unknown backend: {}", name)),
        };
    }

    if cfg!(target_os = "linux") {
        let native = NativeBackend::new();
        if native.is_available() {
            tracing::info!("Selected native backend (Linux direct)");
            return Ok(Box::new(native));
        }
    }

    Err("No runtime backend available. On Linux, ensure kernel supports namespaces.".into())
}
