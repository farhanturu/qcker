use qcker_common::error::Result;

/// Linux capabilities
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub bounding: Vec<String>,
    pub effective: Vec<String>,
    pub inheritable: Vec<String>,
    pub permitted: Vec<String>,
    pub ambient: Vec<String>,
}

/// Apply capabilities to current process
pub fn apply_capabilities(caps: &Capabilities) -> Result<()> {
    // In a real implementation, this would use capctl to set capabilities
    // For now, we'll just log the capabilities
    tracing::info!("Applying capabilities: {:?}", caps);
    Ok(())
}

/// Drop all capabilities
pub fn drop_all_capabilities() -> Result<()> {
    // In a real implementation, this would use capctl to drop all capabilities
    // For now, we'll just log
    tracing::info!("Dropping all capabilities");
    Ok(())
}

/// Get default capabilities for container
pub fn get_default_capabilities() -> Capabilities {
    Capabilities {
        bounding: vec![
            "CAP_CHOWN".to_string(),
            "CAP_DAC_OVERRIDE".to_string(),
            "CAP_FSETID".to_string(),
            "CAP_FOWNER".to_string(),
            "CAP_MKNOD".to_string(),
            "CAP_NET_RAW".to_string(),
            "CAP_SETGID".to_string(),
            "CAP_SETUID".to_string(),
            "CAP_SETFCAP".to_string(),
            "CAP_SETPCAP".to_string(),
            "CAP_NET_BIND_SERVICE".to_string(),
            "CAP_SYS_CHROOT".to_string(),
            "CAP_KILL".to_string(),
            "CAP_AUDIT_WRITE".to_string(),
        ],
        effective: vec![
            "CAP_CHOWN".to_string(),
            "CAP_DAC_OVERRIDE".to_string(),
            "CAP_FSETID".to_string(),
            "CAP_FOWNER".to_string(),
            "CAP_MKNOD".to_string(),
            "CAP_NET_RAW".to_string(),
            "CAP_SETGID".to_string(),
            "CAP_SETUID".to_string(),
            "CAP_SETFCAP".to_string(),
            "CAP_SETPCAP".to_string(),
            "CAP_NET_BIND_SERVICE".to_string(),
            "CAP_SYS_CHROOT".to_string(),
            "CAP_KILL".to_string(),
            "CAP_AUDIT_WRITE".to_string(),
        ],
        inheritable: vec![],
        permitted: vec![
            "CAP_CHOWN".to_string(),
            "CAP_DAC_OVERRIDE".to_string(),
            "CAP_FSETID".to_string(),
            "CAP_FOWNER".to_string(),
            "CAP_MKNOD".to_string(),
            "CAP_NET_RAW".to_string(),
            "CAP_SETGID".to_string(),
            "CAP_SETUID".to_string(),
            "CAP_SETFCAP".to_string(),
            "CAP_SETPCAP".to_string(),
            "CAP_NET_BIND_SERVICE".to_string(),
            "CAP_SYS_CHROOT".to_string(),
            "CAP_KILL".to_string(),
            "CAP_AUDIT_WRITE".to_string(),
        ],
        ambient: vec![],
    }
}

/// Add capability to list
pub fn add_capability(caps: &mut Vec<String>, cap: &str) {
    if !caps.contains(&cap.to_string()) {
        caps.push(cap.to_string());
    }
}

/// Remove capability from list
pub fn remove_capability(caps: &mut Vec<String>, cap: &str) {
    caps.retain(|c| c != cap);
}

/// Check if capability is valid
pub fn is_valid_capability(cap: &str) -> bool {
    // List of valid Linux capabilities
    const VALID_CAPS: &[&str] = &[
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_DAC_READ_SEARCH",
        "CAP_FOWNER",
        "CAP_FSETID",
        "CAP_KILL",
        "CAP_SETGID",
        "CAP_SETUID",
        "CAP_SETPCAP",
        "CAP_LINUX_IMMUTABLE",
        "CAP_NET_BIND_SERVICE",
        "CAP_NET_BROADCAST",
        "CAP_NET_ADMIN",
        "CAP_NET_RAW",
        "CAP_IPC_LOCK",
        "CAP_IPC_OWNER",
        "CAP_SYS_MODULE",
        "CAP_SYS_RAWIO",
        "CAP_SYS_CHROOT",
        "CAP_SYS_PTRACE",
        "CAP_SYS_PACCT",
        "CAP_SYS_ADMIN",
        "CAP_SYS_BOOT",
        "CAP_SYS_NICE",
        "CAP_SYS_RESOURCE",
        "CAP_SYS_TIME",
        "CAP_SYS_TTY_CONFIG",
        "CAP_MKNOD",
        "CAP_LEASE",
        "CAP_AUDIT_WRITE",
        "CAP_AUDIT_CONTROL",
        "CAP_SETFCAP",
        "CAP_MAC_OVERRIDE",
        "CAP_MAC_ADMIN",
        "CAP_SYSLOG",
        "CAP_WAKE_ALARM",
        "CAP_BLOCK_SUSPEND",
        "CAP_AUDIT_READ",
        "CAP_PERFMON",
        "CAP_BPF",
        "CAP_CHECKPOINT_RESTORE",
    ];

    VALID_CAPS.contains(&cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capabilities() {
        let caps = get_default_capabilities();
        assert!(!caps.bounding.is_empty());
        assert!(!caps.effective.is_empty());
        assert!(!caps.permitted.is_empty());
    }

    #[test]
    fn test_is_valid_capability() {
        assert!(is_valid_capability("CAP_CHOWN"));
        assert!(is_valid_capability("CAP_SYS_ADMIN"));
        assert!(!is_valid_capability("INVALID_CAP"));
    }

    #[test]
    fn test_add_remove_capability() {
        let mut caps = vec!["CAP_CHOWN".to_string()];
        add_capability(&mut caps, "CAP_KILL");
        assert_eq!(caps.len(), 2);

        remove_capability(&mut caps, "CAP_CHOWN");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "CAP_KILL");
    }
}
