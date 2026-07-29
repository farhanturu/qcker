use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

/// Registry authentication
#[derive(Debug, Clone)]
pub struct RegistryAuth {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

impl RegistryAuth {
    /// Create basic auth
    pub fn basic(username: &str, password: &str) -> Self {
        Self {
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            token: None,
        }
    }

    /// Create token auth
    pub fn token(token: &str) -> Self {
        Self {
            username: None,
            password: None,
            token: Some(token.to_string()),
        }
    }

    /// Apply auth to request
    pub fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.token {
            request.header("Authorization", format!("Bearer {}", token))
        } else if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            request.basic_auth(username, Some(password))
        } else {
            request
        }
    }
}

/// Docker config file format
#[derive(Debug, Serialize, Deserialize)]
struct DockerConfig {
    auths: std::collections::HashMap<String, AuthEntry>,
}

/// Auth entry in docker config
#[derive(Debug, Serialize, Deserialize)]
struct AuthEntry {
    auth: Option<String>,
    identitytoken: Option<String>,
}

/// Load auth from docker config
pub fn load_docker_auth(registry: &str) -> Result<Option<RegistryAuth>> {
    let config_path = get_docker_config_path()?;

    if !config_path.exists() {
        return Ok(None);
    }

    let config_json = fs::read_to_string(&config_path)
        .map_err(|e| QckerError::Internal(format!("Failed to read docker config: {}", e)))?;

    let config: DockerConfig = serde_json::from_str(&config_json)
        .map_err(|e| QckerError::Internal(format!("Failed to parse docker config: {}", e)))?;

    // Try to find auth for registry
    let registry_url = format!("https://{}", registry);
    if let Some(entry) = config.auths.get(&registry_url).or_else(|| config.auths.get(registry)) {
        if let Some(ref token) = entry.identitytoken {
            return Ok(Some(RegistryAuth::token(token)));
        }

        if let Some(ref auth) = entry.auth {
            let decoded = base64_decode(auth)?;
            let parts: Vec<&str> = decoded.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Ok(Some(RegistryAuth::basic(parts[0], parts[1])));
            }
        }
    }

    Ok(None)
}

/// Get docker config path
fn get_docker_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| QckerError::Internal("Failed to get home directory".to_string()))?;

    // Check for DOCKER_CONFIG env var
    if let Ok(docker_config) = std::env::var("DOCKER_CONFIG") {
        return Ok(PathBuf::from(docker_config).join("config.json"));
    }

    Ok(home_dir.join(".docker").join("config.json"))
}

/// Simple base64 decode
fn base64_decode(input: &str) -> Result<String> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(input)
        .map_err(|e| QckerError::Internal(format!("Failed to decode base64: {}", e)))?;
    String::from_utf8(decoded)
        .map_err(|e| QckerError::Internal(format!("Failed to decode utf8: {}", e)))
}

/// Save auth to qcker config
pub fn save_qcker_auth(registry: &str, auth: &RegistryAuth) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| QckerError::Internal("Failed to get config directory".to_string()))?
        .join("qcker");

    fs::create_dir_all(&config_dir)
        .map_err(|e| QckerError::Internal(format!("Failed to create config dir: {}", e)))?;

    let config_path = config_dir.join("auth.json");

    // Load existing config
    let mut config: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read config: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| QckerError::Internal(format!("Failed to parse config: {}", e)))?
    } else {
        serde_json::json!({ "auths": {} })
    };

    // Update auth
    let auths = config["auths"].as_object_mut()
        .ok_or_else(|| QckerError::Internal("Invalid config format".to_string()))?;

    let entry = if let Some(ref token) = auth.token {
        serde_json::json!({ "identitytoken": token })
    } else if let (Some(ref username), Some(ref password)) = (&auth.username, &auth.password) {
        let auth_str = base64_encode(&format!("{}:{}", username, password));
        serde_json::json!({ "auth": auth_str })
    } else {
        return Ok(());
    };

    auths.insert(registry.to_string(), entry);

    // Save config
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| QckerError::Internal(format!("Failed to serialize config: {}", e)))?;
    fs::write(&config_path, config_json)
        .map_err(|e| QckerError::Internal(format!("Failed to write config: {}", e)))?;

    Ok(())
}

/// Simple base64 encode
fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_auth() {
        let auth = RegistryAuth::basic("user", "pass");
        assert_eq!(auth.username, Some("user".to_string()));
        assert_eq!(auth.password, Some("pass".to_string()));

        let auth = RegistryAuth::token("mytoken");
        assert_eq!(auth.token, Some("mytoken".to_string()));
    }
}
