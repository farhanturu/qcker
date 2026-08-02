use capctl::caps::{Cap, CapSet, CapState};
use capctl::caps::ambient;
use capctl::caps::bounding;
use core::str::FromStr;

use qcker_common::error::{QckerError, Result};

fn cap_from_name(name: &str) -> Option<Cap> {
    Cap::from_str(name).ok()
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
    let mut state = CapState::get_current()
        .map_err(|e| QckerError::capability(format!("Failed to get current caps: {}", e)))?;

    state.effective = names_to_set(&caps.effective);
    state.permitted = names_to_set(&caps.permitted);
    state.inheritable = names_to_set(&caps.inheritable);

    let permitted_set = state.permitted.clone();
    for cap in Cap::iter() {
        if !bounding::read(cap).unwrap_or(false) && !permitted_set.has(cap) {
            if let Err(e) = bounding::drop(cap) {
                tracing::warn!("Failed to drop capability from bounding set in apply: {}", e);
            }
        }
    }

    state.set_current()
        .map_err(|e| QckerError::capability(format!("Failed to set caps: {}", e)))?;

    if ambient::is_supported() {
        if let Err(e) = ambient::clear() {
            tracing::warn!("Failed to clear ambient capabilities in apply: {}", e);
        }
        for cap_name in &caps.ambient {
            if let Some(cap) = cap_from_name(cap_name) {
                if let Err(e) = ambient::raise(cap) {
                    tracing::warn!("Failed to raise ambient capability {} in apply: {}", cap_name, e);
                }
            }
        }
    }

    tracing::info!("Capabilities applied");
    Ok(())
}

pub fn drop_all_capabilities() -> Result<()> {
    let state = CapState::empty();
    state.set_current()
        .map_err(|e| QckerError::capability(format!("Failed to drop caps: {}", e)))?;

    for cap in Cap::iter() {
        if let Err(e) = bounding::drop(cap) {
            tracing::warn!("Failed to drop capability from bounding set: {}", e);
        }
    }
    if ambient::is_supported() {
        if let Err(e) = ambient::clear() {
            tracing::warn!("Failed to clear ambient capabilities: {}", e);
        }
    }

    let verify = CapState::get_current()
        .map_err(|e| QckerError::capability(format!("Failed to verify caps: {}", e)))?;

    if !verify.effective.is_empty() || !verify.permitted.is_empty() {
        return Err(QckerError::capability("Failed to verify capability drop".to_string()));
    }

    tracing::info!("All capabilities dropped");
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct OciCapabilities {
    pub bounding: Vec<String>,
    pub effective: Vec<String>,
    pub inheritable: Vec<String>,
    pub permitted: Vec<String>,
    pub ambient: Vec<String>,
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

