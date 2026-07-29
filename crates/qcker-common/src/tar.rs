use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tar::{Archive, Builder};

use crate::error::Result;
use crate::hash::sha256_file;

/// Extract a tar archive to a destination directory
pub fn extract_tar(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = Archive::new(file);
    archive.unpack(dest)?;
    Ok(())
}

/// Extract a gzipped tar archive to a destination directory
pub fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Extract tar from a reader
pub fn extract_tar_from_reader<R: Read>(reader: R, dest: &Path) -> Result<()> {
    let mut archive = Archive::new(reader);
    archive.unpack(dest)?;
    Ok(())
}

/// Create a tar archive from a directory
pub fn create_tar(source: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)?;
    let mut builder = Builder::new(file);
    builder.append_dir_all(".", source)?;
    builder.finish()?;
    Ok(())
}

/// Create a gzipped tar archive from a directory
pub fn create_tar_gz(source: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    builder.append_dir_all(".", source)?;
    builder.finish()?;
    Ok(())
}

/// Compute SHA256 of a tar.gz file (used for layer digest)
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

        // Create tar from subdirectory
        create_tar(&src_dir, &tar_path).unwrap();
        assert!(tar_path.exists());
    }
}
