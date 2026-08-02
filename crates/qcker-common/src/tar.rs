use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use tar::{Archive, Builder};

use crate::error::{QckerError, Result};
use crate::hash::sha256_file;

fn validate_entry_path(entry_path: &Path, dest: &Path) -> Result<()> {
    let path_str = entry_path.to_string_lossy();
    if path_str.starts_with('/') {
        return Err(QckerError::tar(format!(
            "Path traversal detected (absolute path): {:?}",
            path_str
        )));
    }
    if entry_path
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(QckerError::tar(format!(
            "Path traversal detected (parent component): {:?}",
            path_str
        )));
    }
    let joined = dest.join(entry_path);
    let mut normalized = PathBuf::new();
    for comp in joined.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(QckerError::tar(format!(
                        "Path escapes destination: {:?}",
                        path_str
                    )));
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(dest) {
        return Err(QckerError::tar(format!(
            "Path escapes destination: {:?}",
            path_str
        )));
    }
    Ok(())
}

fn validate_link_target(link_name: &Path, dest: &Path) -> Result<()> {
    let name_str = link_name.to_string_lossy();
    if name_str.starts_with('/') {
        return Err(QckerError::tar(format!(
            "Path traversal detected (absolute link target): {:?}",
            name_str
        )));
    }
    let resolved = dest.join(link_name);
    let mut normalized = PathBuf::new();
    for comp in resolved.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(QckerError::tar(format!(
                        "Link target escapes destination: {:?}",
                        name_str
                    )));
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(dest) {
        return Err(QckerError::tar(format!(
            "Link target escapes destination: {:?}",
            name_str
        )));
    }
    Ok(())
}

fn unpack_entry_safe<R: Read>(entry: &mut tar::Entry<R>, dest: &Path) -> Result<()> {
    let entry_path = entry
        .path()
        .map_err(|e| QckerError::tar(format!("Failed to read entry path: {}", e)))?
        .into_owned();
    let path_display = entry_path.to_string_lossy().to_string();
    validate_entry_path(&entry_path, dest)?;

    let entry_type = entry.header().entry_type();
    if entry_type.is_hard_link() || entry_type.is_symlink() {
        if let Some(link_name) = entry.link_name()? {
            let link_owned = link_name.into_owned();
            validate_link_target(&link_owned, dest)?;
        }
    }

    entry
        .unpack_in(dest)
        .map_err(|e| QckerError::tar(format!("Failed to unpack entry {:?}: {}", path_display, e)))?;
    Ok(())
}

fn safe_unpack<R: Read>(archive: &mut Archive<R>, dest: &Path) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry.map_err(|e| QckerError::tar(format!("Failed to read entry: {}", e)))?;
        unpack_entry_safe(&mut entry, dest)?;
    }
    Ok(())
}

pub fn extract_tar(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = Archive::new(file);
    safe_unpack(&mut archive, dest)
}

pub fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    safe_unpack(&mut archive, dest)
}

pub fn extract_tar_from_reader<R: Read>(reader: R, dest: &Path) -> Result<()> {
    let mut archive = Archive::new(reader);
    safe_unpack(&mut archive, dest)
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

