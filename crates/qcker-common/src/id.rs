use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{QckerError, Result};

pub fn random_bytes(buf: &mut [u8]) -> Result<()> {
    let mut filled = 0usize;
    while filled < buf.len() {
        let ret = unsafe {
            libc::getrandom(
                buf[filled..].as_mut_ptr() as *mut libc::c_void,
                buf.len() - filled,
                0,
            )
        };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(QckerError::internal(format!(
                "getrandom failed: {}",
                err
            )));
        }
        filled += ret as usize;
    }
    Ok(())
}

fn mix_fallback_entropy(hasher: &mut Sha256) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    hasher.update(timestamp.to_string().as_bytes());
    hasher.update(format!("{}", unsafe { libc::getpid() }).as_bytes());
    let local: u8 = 0;
    hasher.update(format!("{:p}", &local as *const u8).as_bytes());
}

pub fn generate_container_id() -> String {
    let mut hasher = Sha256::new();
    let mut salt = [0u8; 32];
    match random_bytes(&mut salt) {
        Ok(()) => hasher.update(&salt),
        Err(e) => {
            tracing::error!("getrandom unavailable, using fallback entropy: {}", e);
            mix_fallback_entropy(&mut hasher);
        }
    }
    hasher.update(b"qcker-container");
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn generate_unique_container_id(existing: &HashSet<String>) -> String {
    for _ in 0..32 {
        let id = generate_container_id();
        if !existing.contains(&id) {
            return id;
        }
        tracing::warn!("container ID collision, regenerating");
    }
    generate_container_id()
}

pub fn generate_image_id(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn generate_network_id() -> String {
    let mut hasher = Sha256::new();
    let mut salt = [0u8; 32];
    match random_bytes(&mut salt) {
        Ok(()) => hasher.update(&salt),
        Err(e) => {
            tracing::error!("getrandom unavailable, using fallback entropy: {}", e);
            mix_fallback_entropy(&mut hasher);
        }
    }
    hasher.update(b"qcker-network");
    let hash = hasher.finalize();
    hex::encode(&hash[..16])
}

pub fn generate_volume_id() -> String {
    let mut hasher = Sha256::new();
    let mut salt = [0u8; 32];
    match random_bytes(&mut salt) {
        Ok(()) => hasher.update(&salt),
        Err(e) => {
            tracing::error!("getrandom unavailable, using fallback entropy: {}", e);
            mix_fallback_entropy(&mut hasher);
        }
    }
    hasher.update(b"qcker-volume");
    let hash = hasher.finalize();
    hex::encode(&hash[..16])
}

pub fn short_id(full_id: &str) -> &str {
    if full_id.len() > 12 {
        &full_id[..12]
    } else {
        full_id
    }
}

