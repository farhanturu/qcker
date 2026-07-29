use std::fs;
use std::path::{Path, PathBuf};

use sha2::Digest;

use qcker_common::error::{QckerError, Result};

/// Build context
pub struct BuildContext {
    pub root: PathBuf,
    pub dockerignore: Vec<String>,
}

impl BuildContext {
    /// Create a new build context
    pub fn new(root: PathBuf) -> Result<Self> {
        let dockerignore = Self::load_dockerignore(&root)?;
        Ok(Self { root, dockerignore })
    }

    /// Load .dockerignore file
    fn load_dockerignore(root: &Path) -> Result<Vec<String>> {
        let ignore_path = root.join(".dockerignore");
        if !ignore_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&ignore_path)
            .map_err(|e| QckerError::Internal(format!("Failed to read .dockerignore: {}", e)))?;

        let patterns: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        Ok(patterns)
    }

    /// Check if a path should be ignored
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.dockerignore {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple pattern matching (supports * and **)
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob matching
        if pattern == "*" {
            return true;
        }

        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            return path.ends_with(suffix) || path.contains(suffix);
        }

        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return path.starts_with(prefix);
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.splitn(2, '*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }

        path == pattern || path.ends_with(&format!("/{}", pattern))
    }

    /// Get context hash
    pub fn compute_hash(&self) -> Result<String> {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;

        // Hash all files in context (excluding ignored ones)
        self.hash_directory(&self.root, &mut hasher)?;

        Ok(hex::encode(hasher.finalize()))
    }

    /// Recursively hash directory contents
    fn hash_directory(&self, dir: &Path, hasher: &mut sha2::Sha256) -> Result<()> {
        for entry in fs::read_dir(dir)
            .map_err(|e| QckerError::Internal(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if self.should_ignore(&path) {
                continue;
            }

            let relative = path.strip_prefix(&self.root).unwrap_or(&path);
            hasher.update(relative.to_string_lossy().as_bytes());

            if path.is_dir() {
                self.hash_directory(&path, hasher)?;
            } else {
                let content = fs::read(&path)
                    .map_err(|e| QckerError::Internal(format!("Failed to read file: {}", e)))?;
                hasher.update(&content);
            }
        }

        Ok(())
    }

    /// Get list of files in context
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.list_directory(&self.root, &mut files)?;
        Ok(files)
    }

    /// Recursively list files
    fn list_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .map_err(|e| QckerError::Internal(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| QckerError::Internal(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if self.should_ignore(&path) {
                continue;
            }

            if path.is_dir() {
                self.list_directory(&path, files)?;
            } else {
                files.push(path);
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
    fn test_build_context() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create some files
        fs::write(root.join("Dockerfile"), "FROM alpine").unwrap();
        fs::write(root.join(".dockerignore"), "*.log\ntarget/").unwrap();
        fs::write(root.join("test.log"), "log content").unwrap();
        fs::write(root.join("src"), "source code").unwrap();

        let context = BuildContext::new(root.to_path_buf()).unwrap();

        // Check ignore patterns
        assert!(context.should_ignore(&root.join("test.log")));
        assert!(!context.should_ignore(&root.join("src")));

        // List files (Dockerfile, .dockerignore, and src)
        let files = context.list_files().unwrap();
        assert_eq!(files.len(), 3);
    }
}
