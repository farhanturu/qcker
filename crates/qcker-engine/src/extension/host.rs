use std::collections::HashMap;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};
use qcker_ext_api::types::{ContainerInfo, ExtensionInfo, IpcRequest, IpcResponse};

use super::loader::ExtensionLoader;
use super::manager::ExtensionManager;

/// Extension host - manages extension lifecycle and IPC
pub struct ExtensionHost {
    pub manager: ExtensionManager,
    pub loader: ExtensionLoader,
    pub active_extensions: HashMap<String, ActiveExtension>,
}

/// Active extension
pub struct ActiveExtension {
    pub info: ExtensionInfo,
    pub loaded: bool,
}

impl ExtensionHost {
    /// Create a new extension host
    pub fn new(data_dir: PathBuf) -> Self {
        let extensions_dir = data_dir.join("extensions");
        Self {
            manager: ExtensionManager::new(data_dir),
            loader: ExtensionLoader::new(extensions_dir),
            active_extensions: HashMap::new(),
        }
    }

    /// Initialize the extension host
    pub fn init(&mut self) -> Result<()> {
        self.manager.init()?;

        // Load all installed extensions
        for ext_info in self.manager.list() {
            let ext_id = ext_info.metadata.id.clone();
            match self.loader.load(&ext_id) {
                Ok(loaded) => {
                    self.active_extensions.insert(
                        ext_id.clone(),
                        ActiveExtension {
                            info: ext_info.clone(),
                            loaded: loaded.loaded,
                        },
                    );
                    tracing::info!("Extension {} loaded", ext_id);
                }
                Err(e) => {
                    tracing::warn!("Failed to load extension {}: {}", ext_id, e);
                }
            }
        }

        Ok(())
    }

    /// List active extensions
    pub fn list_active(&self) -> Vec<&ActiveExtension> {
        self.active_extensions.values().collect()
    }

    /// Send IPC request to extension
    pub fn send_request(&self, extension_id: &str, request: IpcRequest) -> Result<IpcResponse> {
        if !self.active_extensions.contains_key(extension_id) {
            return Err(QckerError::InvalidArgument(format!(
                "Extension not active: {}",
                extension_id
            )));
        }

        // In a real implementation, this would send the request via Unix socket
        // For now, return a placeholder response
        tracing::info!(
            "Sending IPC request to {}: {}",
            extension_id,
            request.method
        );

        Ok(IpcResponse::success(
            request.id,
            serde_json::json!({"status": "ok"}),
        ))
    }

    /// Notify extensions of container event
    pub fn notify_container_event(&self, event: &str, _container: &ContainerInfo) -> Result<()> {
        for (ext_id, ext) in &self.active_extensions {
            if ext.loaded {
                tracing::info!(
                    "Notifying extension {} of container event: {}",
                    ext_id,
                    event
                );
                // In a real implementation, this would send the event via IPC
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extension_host() {
        let tmp = TempDir::new().unwrap();
        let mut host = ExtensionHost::new(tmp.path().to_path_buf());

        // Init may fail if no extensions, that's ok
        let _ = host.init();

        // List should work even with no extensions
        let active = host.list_active();
        assert_eq!(active.len(), 0);
    }
}
