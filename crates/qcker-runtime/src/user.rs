use std::fs;

use qcker_common::error::{QckerError, Result};

pub struct UserMapping {
    pub container_uid: u32,
    pub host_uid: u32,
    pub container_gid: u32,
    pub host_gid: u32,
}

pub fn setup_user_mapping(mapping: &UserMapping) -> Result<()> {
    let uid_map = format!("{} {} 1", mapping.container_uid, mapping.host_uid);
    fs::write("/proc/self/uid_map", &uid_map)
        .map_err(|e| QckerError::namespace(format!("Failed to write uid_map: {}", e)))?;

    fs::write("/proc/self/setgroups", "deny")
        .map_err(|e| QckerError::namespace(format!("Failed to write setgroups: {}", e)))?;

    let gid_map = format!("{} {} 1", mapping.container_gid, mapping.host_gid);
    fs::write("/proc/self/gid_map", &gid_map)
        .map_err(|e| QckerError::namespace(format!("Failed to write gid_map: {}", e)))?;

    Ok(())
}

pub fn set_uid_gid(uid: u32, gid: u32) -> Result<()> {
    nix::unistd::setgid(nix::unistd::Gid::from_raw(gid))
        .map_err(|e| QckerError::namespace(format!("Failed to set GID: {}", e)))?;

    nix::unistd::setuid(nix::unistd::Uid::from_raw(uid))
        .map_err(|e| QckerError::namespace(format!("Failed to set UID: {}", e)))?;

    Ok(())
}

pub fn parse_user(user: &str) -> Result<(u32, u32)> {
    if user.contains(':') {
        let parts: Vec<&str> = user.split(':').collect();
        if parts.len() != 2 {
            return Err(QckerError::invalid_argument(format!(
                "Invalid user format: {}",
                user
            )));
        }
        let uid = parts[0]
            .parse::<u32>()
            .map_err(|_| QckerError::invalid_argument(format!("Invalid UID: {}", parts[0])))?;
        let gid = parts[1]
            .parse::<u32>()
            .map_err(|_| QckerError::invalid_argument(format!("Invalid GID: {}", parts[1])))?;
        Ok((uid, gid))
    } else {
        let uid = user
            .parse::<u32>()
            .map_err(|_| QckerError::invalid_argument(format!("Invalid user: {}", user)))?;
        Ok((uid, uid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user() {
        assert_eq!(parse_user("1000:1000").unwrap(), (1000, 1000));
        assert_eq!(parse_user("0:0").unwrap(), (0, 0));
        assert_eq!(parse_user("1000").unwrap(), (1000, 1000));
        assert!(parse_user("invalid").is_err());
    }
}
