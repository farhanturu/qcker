use std::fs;
use std::path::Path;

use crate::error::Result;

/// Recursively copy a directory
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Recursively remove a directory
pub fn remove_dir_all_safe(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Create directory and all parents
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Check if running as root
pub fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Get current user ID
pub fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

/// Get current group ID
pub fn getgid() -> u32 {
    unsafe { libc::getgid() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_all() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create source structure
        fs::create_dir_all(src.path().join("subdir")).unwrap();
        fs::write(src.path().join("file1.txt"), "hello").unwrap();
        fs::write(src.path().join("subdir/file2.txt"), "world").unwrap();

        // Copy
        copy_dir_all(src.path(), &dst.path().join("copy")).unwrap();

        // Verify
        assert!(dst.path().join("copy/file1.txt").exists());
        assert!(dst.path().join("copy/subdir/file2.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.path().join("copy/file1.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_ensure_dir() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a/b/c");
        ensure_dir(&path).unwrap();
        assert!(path.exists());
    }
}
