use sha2::{Digest, Sha256};

pub fn generate_container_id() -> String {
    let mut random_bytes = [0u8; 32];
    getrandom::getrandom(&mut random_bytes)
        .expect("Failed to generate random bytes for container ID");
    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
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
    let mut random_bytes = [0u8; 16];
    getrandom::getrandom(&mut random_bytes)
        .expect("Failed to generate random bytes for network ID");
    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
    let hash = hasher.finalize();
    hex::encode(&hash[..16])
}

pub fn generate_volume_id() -> String {
    let mut random_bytes = [0u8; 16];
    getrandom::getrandom(&mut random_bytes)
        .expect("Failed to generate random bytes for volume ID");
    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
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
