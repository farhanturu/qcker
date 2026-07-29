use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

use super::auth::RegistryAuth;
use crate::image::store::{Image, ImageConfig};

/// OCI Distribution manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    pub schema_version: u32,
    pub media_type: Option<String>,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
}

/// OCI Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    pub urls: Option<Vec<String>>,
}

/// Registry client for OCI Distribution Spec
pub struct RegistryClient {
    pub registry: String,
    pub auth: Option<RegistryAuth>,
    pub client: reqwest::Client,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(registry: &str) -> Self {
        Self {
            registry: registry.to_string(),
            auth: None,
            client: reqwest::Client::new(),
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, auth: RegistryAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Parse image reference (e.g., "alpine:latest" -> ("library/alpine", "latest"))
    pub fn parse_reference(image: &str) -> (String, String) {
        let parts: Vec<&str> = image.splitn(2, ':').collect();
        let name = parts[0];
        let tag = if parts.len() > 1 { parts[1] } else { "latest" };

        // Add library/ prefix if no namespace
        let full_name = if !name.contains('/') {
            format!("library/{}", name)
        } else {
            name.to_string()
        };

        (full_name, tag.to_string())
    }

    /// Pull an image manifest
    pub async fn pull_manifest(&self, name: &str, tag: &str) -> Result<OciManifest> {
        let url = format!(
            "https://{}/v2/{}/manifests/{}",
            self.registry, name, tag
        );

        let mut request = self.client.get(&url)
            .header("Accept", "application/vnd.oci.image.manifest.v1+json");

        if let Some(ref auth) = self.auth {
            request = auth.apply(request);
        }

        let response = request.send().await
            .map_err(|e| QckerError::Internal(format!("Failed to pull manifest: {}", e)))?;

        if !response.status().is_success() {
            return Err(QckerError::Internal(format!(
                "Failed to pull manifest: {}",
                response.status()
            )));
        }

        let manifest: OciManifest = response.json().await
            .map_err(|e| QckerError::Internal(format!("Failed to parse manifest: {}", e)))?;

        Ok(manifest)
    }

    /// Pull a blob (layer or config)
    pub async fn pull_blob(&self, name: &str, digest: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://{}/v2/{}/blobs/{}",
            self.registry, name, digest
        );

        let mut request = self.client.get(&url);

        if let Some(ref auth) = self.auth {
            request = auth.apply(request);
        }

        let response = request.send().await
            .map_err(|e| QckerError::Internal(format!("Failed to pull blob: {}", e)))?;

        if !response.status().is_success() {
            return Err(QckerError::Internal(format!(
                "Failed to pull blob: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await
            .map_err(|e| QckerError::Internal(format!("Failed to read blob: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// Pull a complete image
    pub async fn pull_image(&self, image: &str, data_dir: PathBuf) -> Result<Image> {
        let (name, tag) = Self::parse_reference(image);

        tracing::info!("Pulling {}:{} from {}", name, tag, self.registry);

        // Pull manifest
        let manifest = self.pull_manifest(&name, &tag).await?;

        // Pull config
        let config_bytes = self.pull_blob(&name, &manifest.config.digest).await?;
        let config: ImageConfig = serde_json::from_slice(&config_bytes)
            .map_err(|e| QckerError::Internal(format!("Failed to parse config: {}", e)))?;

        // Pull layers
        let mut layer_digests = Vec::new();
        let layers_dir = data_dir.join("layers");
        std::fs::create_dir_all(&layers_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create layers dir: {}", e)))?;

        for layer_desc in &manifest.layers {
            tracing::info!("Pulling layer {}", layer_desc.digest);

            let layer_bytes = self.pull_blob(&name, &layer_desc.digest).await?;

            // Save layer
            let hash = layer_desc.digest.strip_prefix("sha256:").unwrap_or(&layer_desc.digest);
            let layer_dir = layers_dir.join(hash);
            std::fs::create_dir_all(&layer_dir)
                .map_err(|e| QckerError::Internal(format!("Failed to create layer dir: {}", e)))?;

            let layer_file = layer_dir.join("layer.tar.gz");
            std::fs::write(&layer_file, &layer_bytes)
                .map_err(|e| QckerError::Internal(format!("Failed to write layer: {}", e)))?;

            // Extract layer
            let extract_dir = layer_dir.join("layer");
            std::fs::create_dir_all(&extract_dir)
                .map_err(|e| QckerError::Internal(format!("Failed to create extract dir: {}", e)))?;
            qcker_common::tar::extract_tar_gz(&layer_file, &extract_dir)?;

            layer_digests.push(layer_desc.digest.clone());
        }

        // Create image
        let image_id = manifest.config.digest.strip_prefix("sha256:").unwrap_or(&manifest.config.digest);
        let image = Image {
            id: image_id[..12].to_string(),
            tags: vec![format!("{}:{}", name, tag)],
            created_at: chrono::Utc::now().to_rfc3339(),
            size: manifest.layers.iter().map(|l| l.size).sum(),
            layers: layer_digests,
            config: config,
        };

        // Store image
        let store = crate::image::store::ImageStore::new(data_dir);
        store.init()?;
        store.store_image(&image)?;

        tracing::info!("Image pulled successfully: {}", image.id);

        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reference() {
        let (name, tag) = RegistryClient::parse_reference("alpine:latest");
        assert_eq!(name, "library/alpine");
        assert_eq!(tag, "latest");

        let (name, tag) = RegistryClient::parse_reference("nginx:1.25");
        assert_eq!(name, "library/nginx");
        assert_eq!(tag, "1.25");

        let (name, tag) = RegistryClient::parse_reference("myuser/myapp:v1");
        assert_eq!(name, "myuser/myapp");
        assert_eq!(tag, "v1");
    }
}
