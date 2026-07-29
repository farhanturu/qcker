use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a new unique container ID
pub fn generate_container_id() -> String {
    generate_id()
}

/// Generate a new unique image ID
pub fn generate_image_id() -> String {
    generate_id()
}

/// Generate a new unique network ID
pub fn generate_network_id() -> String {
    generate_id()
}

/// Generate a new unique volume ID
pub fn generate_volume_id() -> String {
    generate_id()
}

/// Generate a unique ID using timestamp + counter
fn generate_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_string().as_bytes());
    hasher.update(b"qcker");
    let hash = hasher.finalize();
    hex::encode(&hash[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_container_id() {
        let id = generate_container_id();
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_unique_ids() {
        let id1 = generate_container_id();
        let id2 = generate_container_id();
        assert_ne!(id1, id2);
    }
}
