use qcker_common::error::{QckerError, Result};

/// Default seccomp action
#[derive(Debug, Clone)]
pub enum SeccompAction {
    Allow,
    Errno,
    Kill,
    Trace,
    Log,
}

impl SeccompAction {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "SCMP_ACT_ALLOW" => Ok(SeccompAction::Allow),
            "SCMP_ACT_ERRNO" => Ok(SeccompAction::Errno),
            "SCMP_ACT_KILL" => Ok(SeccompAction::Kill),
            "SCMP_ACT_TRACE" => Ok(SeccompAction::Trace),
            "SCMP_ACT_LOG" => Ok(SeccompAction::Log),
            _ => Err(QckerError::Seccomp(format!("Unknown action: {}", s))),
        }
    }
}

/// Seccomp profile
pub struct SeccompProfile {
    pub default_action: SeccompAction,
    pub syscalls: Vec<SeccompSyscallRule>,
}

/// Seccomp syscall rule
pub struct SeccompSyscallRule {
    pub names: Vec<String>,
    pub action: SeccompAction,
}

/// Apply default seccomp profile
pub fn apply_default_profile() -> Result<()> {
    // In a real implementation, this would use libseccomp
    // For now, we'll use a simple prctl-based approach

    // Set NO_NEW_PRIVS
    unsafe {
        let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(QckerError::Seccomp(format!(
                "Failed to set NO_NEW_PRIVS: {}",
                std::io::Error::last_os_error()
            )));
        }
    }

    Ok(())
}

/// Apply seccomp profile
pub fn apply_profile(_profile: &SeccompProfile) -> Result<()> {
    // In a real implementation, this would use libseccomp to load the profile
    // For now, just set NO_NEW_PRIVS
    apply_default_profile()?;

    tracing::info!("Seccomp profile applied (NO_NEW_PRIVS set)");
    Ok(())
}

/// Load seccomp profile from JSON
pub fn load_profile_from_json(json: &str) -> Result<SeccompProfile> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| QckerError::Seccomp(format!("Failed to parse seccomp profile: {}", e)))?;

    let default_action = value["defaultAction"]
        .as_str()
        .ok_or_else(|| QckerError::Seccomp("Missing defaultAction".to_string()))?;

    Ok(SeccompProfile {
        default_action: SeccompAction::from_str(default_action)?,
        syscalls: vec![], // TODO: Parse syscalls
    })
}

/// Create a restrictive seccomp profile
pub fn create_restrictive_profile() -> SeccompProfile {
    SeccompProfile {
        default_action: SeccompAction::Allow,
        syscalls: vec![
            SeccompSyscallRule {
                names: vec![
                    "add_key".to_string(),
                    "bpf".to_string(),
                    "clock_settime".to_string(),
                    "create_module".to_string(),
                    "delete_module".to_string(),
                    "finit_module".to_string(),
                    "init_module".to_string(),
                    "ioperm".to_string(),
                    "iopl".to_string(),
                    "kcmp".to_string(),
                    "kexec_file_load".to_string(),
                    "kexec_load".to_string(),
                    "keyctl".to_string(),
                    "lookup_dcookie".to_string(),
                    "mount".to_string(),
                    "move_pages".to_string(),
                    "nfsservctl".to_string(),
                    "perf_event_open".to_string(),
                    "personality".to_string(),
                    "pivot_root".to_string(),
                    "process_vm_readv".to_string(),
                    "process_vm_writev".to_string(),
                    "ptrace".to_string(),
                    "reboot".to_string(),
                    "request_key".to_string(),
                    "set_mempolicy".to_string(),
                    "swapoff".to_string(),
                    "swapon".to_string(),
                    "sysfs".to_string(),
                    "_sysctl".to_string(),
                    "umount2".to_string(),
                    "unshare".to_string(),
                    "uselib".to_string(),
                    "userfaultfd".to_string(),
                    "ustat".to_string(),
                ],
                action: SeccompAction::Errno,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seccomp_action_from_str() {
        assert!(matches!(
            SeccompAction::from_str("SCMP_ACT_ALLOW").unwrap(),
            SeccompAction::Allow
        ));
        assert!(matches!(
            SeccompAction::from_str("SCMP_ACT_ERRNO").unwrap(),
            SeccompAction::Errno
        ));
        assert!(SeccompAction::from_str("INVALID").is_err());
    }

    #[test]
    fn test_restrictive_profile() {
        let profile = create_restrictive_profile();
        assert!(matches!(profile.default_action, SeccompAction::Allow));
        assert!(!profile.syscalls.is_empty());
    }
}
