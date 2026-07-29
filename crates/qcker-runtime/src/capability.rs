use capctl::caps::CapSet;

use qcker_common::error::{QckerError, Result};

fn cap_from_name(name: &str) -> Option<capctl::caps::Cap> {
    match name {
        "CAP_CHOWN" => Some(capctl::caps::Cap::CHOWN),
        "CAP_DAC_OVERRIDE" => Some(capctl::caps::Cap::DAC_OVERRIDE),
        "CAP_DAC_READ_SEARCH" => Some(capctl::caps::Cap::DAC_READ_SEARCH),
        "CAP_FOWNER" => Some(capctl::caps::Cap::FOWNER),
        "CAP_FSETID" => Some(capctl::caps::Cap::FSETID),
        "CAP_KILL" => Some(capctl::caps::Cap::KILL),
        "CAP_SETGID" => Some(capctl::caps::Cap::SETGID),
        "CAP_SETUID" => Some(capctl::caps::Cap::SETUID),
        "CAP_SETPCAP" => Some(capctl::caps::Cap::SETPCAP),
        "CAP_LINUX_IMMUTABLE" => Some(capctl::caps::Cap::LINUX_IMMUTABLE),
        "CAP_NET_BIND_SERVICE" => Some(capctl::caps::Cap::NET_BIND_SERVICE),
        "CAP_NET_BROADCAST" => Some(capctl::caps::Cap::NET_BROADCAST),
        "CAP_NET_ADMIN" => Some(capctl::caps::Cap::NET_ADMIN),
        "CAP_NET_RAW" => Some(capctl::caps::Cap::NET_RAW),
        "CAP_IPC_LOCK" => Some(capctl::caps::Cap::IPC_LOCK),
        "CAP_IPC_OWNER" => Some(capctl::caps::Cap::IPC_OWNER),
        "CAP_SYS_MODULE" => Some(capctl::caps::Cap::SYS_MODULE),
        "CAP_SYS_RAWIO" => Some(capctl::caps::Cap::SYS_RAWIO),
        "CAP_SYS_CHROOT" => Some(capctl::caps::Cap::SYS_CHROOT),
        "CAP_SYS_PTRACE" => Some(capctl::caps::Cap::SYS_PTRACE),
        "CAP_SYS_PACCT" => Some(capctl::caps::Cap::SYS_PACCT),
        "CAP_SYS_ADMIN" => Some(capctl::caps::Cap::SYS_ADMIN),
        "CAP_SYS_BOOT" => Some(capctl::caps::Cap::SYS_BOOT),
        "CAP_SYS_NICE" => Some(capctl::caps::Cap::SYS_NICE),
        "CAP_SYS_RESOURCE" => Some(capctl::caps::Cap::SYS_RESOURCE),
        "CAP_SYS_TIME" => Some(capctl::caps::Cap::SYS_TIME),
        "CAP_SYS_TTY_CONFIG" => Some(capctl::caps::Cap::SYS_TTY_CONFIG),
        "CAP_MKNOD" => Some(capctl::caps::Cap::MKNOD),
        "CAP_LEASE" => Some(capctl::caps::Cap::LEASE),
        "CAP_AUDIT_WRITE" => Some(capctl::caps::Cap::AUDIT_WRITE),
        "CAP_AUDIT_CONTROL" => Some(capctl::caps::Cap::AUDIT_CONTROL),
        "CAP_SETFCAP" => Some(capctl::caps::Cap::SETFCAP),
        "CAP_MAC_OVERRIDE" => Some(capctl::caps::Cap::MAC_OVERRIDE),
        "CAP_MAC_ADMIN" => Some(capctl::caps::Cap::MAC_ADMIN),
        "CAP_SYSLOG" => Some(capctl::caps::Cap::SYSLOG),
        "CAP_WAKE_ALARM" => Some(capctl::caps::Cap::WAKE_ALARM),
        "CAP_BLOCK_SUSPEND" => Some(capctl::caps::Cap::BLOCK_SUSPEND),
        "CAP_AUDIT_READ" => Some(capctl::caps::Cap::AUDIT_READ),
        "CAP_PERFMON" => Some(capctl::caps::Cap::PERFMON),
        "CAP_BPF" => Some(capctl::caps::Cap::BPF),
        "CAP_CHECKPOINT_RESTORE" => Some(capctl::caps::Cap::CHECKPOINT_RESTORE),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub struct OciCapabilities {
    pub bounding: Vec<String>,
    pub effective: Vec<String>,
    pub inheritable: Vec<String>,
    pub permitted: Vec<String>,
    pub ambient: Vec<String>,
}

fn names_to_set(names: &[String]) -> CapSet {
    let mut set = CapSet::empty();
    for name in names {
        if let Some(cap) = cap_from_name(name) {
            set.add(cap);
        }
    }
    set
}

pub fn apply_capabilities(caps: &OciCapabilities) -> Result<()> {
    let mut state = capctl::caps::CapState::get_current()
        .map_err(|e| QckerError::Capability(format!("Failed to get current caps: {}", e)))?;

    state.effective = names_to_set(&caps.effective);
    state.permitted = names_to_set(&caps.permitted);
    state.inheritable = names_to_set(&caps.inheritable);

    state.set_current()
        .map_err(|e| QckerError::Capability(format!("Failed to set caps: {}", e)))?;

    tracing::info!("Capabilities applied");
    Ok(())
}

pub fn drop_all_capabilities() -> Result<()> {
    let mut state = capctl::caps::CapState::empty();
    state.set_current()
        .map_err(|e| QckerError::Capability(format!("Failed to drop caps: {}", e)))?;

    // Verify
    let verify = capctl::caps::CapState::get_current()
        .map_err(|e| QckerError::Capability(format!("Failed to verify caps: {}", e)))?;

    if !verify.effective.is_empty() || !verify.permitted.is_empty() {
        return Err(QckerError::Capability("Failed to verify capability drop".to_string()));
    }

    tracing::info!("All capabilities dropped");
    Ok(())
}

pub fn get_default_capabilities() -> OciCapabilities {
    let default_caps = vec![
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
    ];

    OciCapabilities {
        bounding: default_caps.clone(),
        effective: default_caps.clone(),
        inheritable: vec![],
        permitted: default_caps,
        ambient: vec![],
    }
}

pub fn is_valid_capability(cap: &str) -> bool {
    cap_from_name(cap).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_from_name() {
        assert!(cap_from_name("CAP_CHOWN").is_some());
        assert!(cap_from_name("CAP_SYS_ADMIN").is_some());
        assert!(cap_from_name("INVALID_CAP").is_none());
    }

    #[test]
    fn test_default_capabilities() {
        let caps = get_default_capabilities();
        assert!(!caps.bounding.is_empty());
        assert!(!caps.effective.is_empty());
    }

    #[test]
    fn test_is_valid_capability() {
        assert!(is_valid_capability("CAP_CHOWN"));
        assert!(!is_valid_capability("INVALID"));
    }

    #[test]
    #[ignore]
    fn test_drop_all_capabilities() {
        assert!(drop_all_capabilities().is_ok());
    }
}
