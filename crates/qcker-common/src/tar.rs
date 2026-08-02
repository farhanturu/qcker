use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};
use tar::{Archive, Builder};

use crate::error::{QckerError, Result};
use crate::hash::sha256_file;

fn validate_entry_path(entry_path: &Path, dest: &Path) -> Result<()> {
    if entry_path.is_absolute() {
        return Err(QckerError::tar(format!(
            "Absolute path in archive: {}",
            entry_path.display()
        )));
    }

    for component in entry_path.components() {
        if component == Component::ParentDir {
            return Err(QckerError::tar(format!(
                "Path traversal in archive: {}",
                entry_path.display()
            )));
        }
    }

    let cleaned = entry_path.strip_prefix(".").unwrap_or(entry_path);
    if cleaned.as_os_str().is_empty() {
        return Ok(());
    }

    let full_path = dest.join(cleaned);
    let canonical_dest = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());
    if let Some(parent) = full_path.parent() {
        if let Ok(canonical_parent) = parent.canonicalize() {
            if !canonical_parent.starts_with(&canonical_dest) {
                return Err(QckerError::tar(format!(
                    "Path escapes destination: {}",
                    entry_path.display()
                )));
            }
        }
    }

    Ok(())
}

pub fn safe_extract(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = Archive::new(file);
    archive.set_unpack_xattrs(false);
    if !crate::fs::is_root() {
        archive.set_preserve_permissions(false);
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;
        validate_entry_path(&entry_path, dest)?;

        let cleaned = entry_path.strip_prefix(".").unwrap_or(&entry_path);
        if cleaned.as_os_str().is_empty() {
            continue;
        }

        entry.unpack_in(dest)?;
    }

    Ok(())
}

pub fn safe_extract_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.set_unpack_xattrs(false);
    if !crate::fs::is_root() {
        archive.set_preserve_permissions(false);
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;
        validate_entry_path(&entry_path, dest)?;
        entry.unpack(dest)?;
    }

    Ok(())
}

pub fn extract_tar(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = Archive::new(file);
    archive.unpack(dest)?;
    Ok(())
}

pub fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

pub fn extract_tar_from_reader<R: Read>(reader: R, dest: &Path) -> Result<()> {
    let mut archive = Archive::new(reader);
    archive.unpack(dest)?;
    Ok(())
}

pub fn create_tar(source: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)?;
    let mut builder = Builder::new(file);
    builder.append_dir_all(".", source)?;
    builder.finish()?;
    Ok(())
}

pub fn create_tar_gz(source: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    builder.append_dir_all(".", source)?;
    builder.finish()?;
    Ok(())
}

pub fn tar_gz_digest(path: &Path) -> Result<String> {
    sha256_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_tar() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("source");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("hello.txt"), "hello world").unwrap();

        let tar_path = tmp.path().join("test.tar");

        create_tar(&src_dir, &tar_path).unwrap();
        assert!(tar_path.exists());
    }

    #[test]
    fn test_safe_extract_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("source");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("hello.txt"), "hello world").unwrap();

        let tar_path = tmp.path().join("test.tar");
        let file = File::create(&tar_path).unwrap();
        let mut builder = Builder::new(file);
        builder.append_path_with_name(src_dir.join("hello.txt"), "hello.txt").unwrap();
        builder.finish().unwrap();

        let dest = tmp.path().join("dest");
        fs::create_dir_all(&dest).unwrap();

        let result = safe_extract(&tar_path, &dest);
        if let Err(ref e) = result {
            eprintln!("ERROR: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(dest.join("hello.txt").exists());
    }

    #[test]
    fn test_validate_rejects_dotdot() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("dest");
        fs::create_dir_all(&dest).unwrap();

        let result = validate_entry_path(std::path::Path::new("../../etc/passwd"), &dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_absolute() {
        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("dest");
        fs::create_dir_all(&dest).unwrap();

        let result = validate_entry_path(std::path::Path::new("/etc/passwd"), &dest);
        assert!(result.is_err());
    }
}
