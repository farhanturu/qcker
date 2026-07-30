use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use qcker_common::error::{QckerError, Result};
use qcker_common::hash::sha256_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub layer_digest: String,
    pub created_at: String,
}

pub struct BuildCache {
    pub cache_dir: PathBuf,
}

impl BuildCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create cache dir: {}", e)))?;
        Ok(())
    }

    pub fn compute_key(
        &self,
        instruction: &str,
        context_hash: &str,
        parent_key: &str,
    ) -> String {
        let input = format!("{}:{}:{}", parent_key, instruction, context_hash);
        sha256_str(&input)
    }

    pub fn has_entry(&self, key: &str) -> bool {
        self.cache_dir.join(key).exists()
    }

    pub fn get_entry(&self, key: &str) -> Result<Option<CacheEntry>> {
        let entry_dir = self.cache_dir.join(key);
        if !entry_dir.exists() {
            return Ok(None);
        }

        let meta_path = entry_dir.join("meta.json");
        if !meta_path.exists() {
            return Ok(None);
        }

        let meta_json = fs::read_to_string(&meta_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read cache meta: {}", e)))?;

        let entry: CacheEntry = serde_json::from_str(&meta_json)
            .map_err(|e| QckerError::Internal(format!("Failed to parse cache meta: {}", e)))?;

        Ok(Some(entry))
    }

    pub fn store_entry(&self, key: &str, layer_digest: &str) -> Result<()> {
        let entry_dir = self.cache_dir.join(key);
        fs::create_dir_all(&entry_dir)
            .map_err(|e| QckerError::Internal(format!("Failed to create cache entry dir: {}", e)))?;

        let entry = CacheEntry {
            key: key.to_string(),
            layer_digest: layer_digest.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let meta_json = serde_json::to_string_pretty(&entry)
            .map_err(|e| QckerError::Internal(format!("Failed to serialize cache entry: {}", e)))?;

        let meta_path = entry_dir.join("meta.json");
        fs::write(&meta_path, meta_json)
            .map_err(|e| QckerError::Internal(format!("Failed to write cache entry: {}", e)))?;

        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| QckerError::Internal(format!("Failed to clear cache: {}", e)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_cache() {
        let tmp = TempDir::new().unwrap();
        let cache = BuildCache::new(tmp.path().to_path_buf());
        cache.init().unwrap();

        let key = cache.compute_key("RUN echo hello", "abc123", "parent_key");
        assert!(!key.is_empty());

        cache.store_entry(&key, "sha256:layer123").unwrap();

        let entry = cache.get_entry(&key).unwrap().unwrap();
        assert_eq!(entry.layer_digest, "sha256:layer123");
    }
}
