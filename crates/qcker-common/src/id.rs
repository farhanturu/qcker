use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_container_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_string().as_bytes());
    hasher.update(b"qcker-container");
    hasher.update(b"random-salt-here");
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn generate_image_id(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn generate_network_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_string().as_bytes());
    hasher.update(b"qcker-network");
    let hash = hasher.finalize();
    hex::encode(&hash[..16])
}

pub fn generate_volume_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_string().as_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_container_id() {
        let id = generate_container_id();
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_image_id() {
        let id = generate_image_id(b"test content");
        assert_eq!(id.len(), 64);
        let id2 = generate_image_id(b"different content");
        assert_ne!(id, id2);
    }

    #[test]
    fn test_unique_ids() {
        let id1 = generate_container_id();
        let id2 = generate_container_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_short_id() {
        let id = generate_container_id();
        let short = short_id(&id);
        assert_eq!(short.len(), 12);
    }
}
