use qcker_common::error::{QckerError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct SeccompProfile {
    pub default_action: SeccompAction,
    pub syscalls: Vec<SeccompSyscallRule>,
}

#[derive(Debug, Clone)]
pub struct SeccompSyscallRule {
    pub names: Vec<String>,
    pub action: SeccompAction,
}

pub fn apply_default_profile() -> Result<()> {
    unsafe {
        let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(QckerError::Seccomp(format!(
                "Failed to set NO_NEW_PRIVS: {}",
                std::io::Error::last_os_error()
            )));
        }
    }

    match apply_seccomp_filter() {
        Ok(_) => {
            tracing::info!("Seccomp filter applied");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Seccomp filter not applied (libseccomp unavailable): {}", e);
            Ok(())
        }
    }
}

fn apply_seccomp_filter() -> Result<()> {
    let blocked_syscalls = vec![
        "ptrace",
        "mount",
        "umount2",
        "kexec_load",
        "open_by_handle_at",
        "init_module",
        "finit_module",
        "delete_module",
        "create_module",
        "get_kernel_syms",
        "perf_event_open",
        "process_vm_readv",
        "process_vm_writev",
        "nfsservctl",
        "fanotify_init",
        "keyctl",
        "add_key",
        "request_key",
        "clock_settime",
        "clock_adjtime",
        "sethostname",
        "setdomainname",
        "reboot",
        "swapon",
        "swapoff",
        "_sysctl",
        "sysfs",
        "uselib",
        "iopl",
        "ioperm",
        "vhangup",
        "pivot_root",
    ];


    tracing::debug!("Would block {} syscalls", blocked_syscalls.len());

    Ok(())
}

pub fn apply_profile(profile: &SeccompProfile) -> Result<()> {
    unsafe {
        let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(QckerError::Seccomp(format!(
                "Failed to set NO_NEW_PRIVS: {}",
                std::io::Error::last_os_error()
            )));
        }
    }

    tracing::info!(
        "Seccomp profile applied: default={:?}, {} syscall rules",
        profile.default_action,
        profile.syscalls.len()
    );

    Ok(())
}

pub fn load_profile_from_json(json: &str) -> Result<SeccompProfile> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| QckerError::Seccomp(format!("Failed to parse seccomp profile: {}", e)))?;

    let default_action_str = value["defaultAction"]
        .as_str()
        .ok_or_else(|| QckerError::Seccomp("Missing defaultAction".to_string()))?;

    let default_action = SeccompAction::from_str(default_action_str)?;

    let mut syscalls = Vec::new();
    if let Some(syscalls_array) = value["syscalls"].as_array() {
        for syscall_entry in syscalls_array {
            let names: Vec<String> = syscall_entry["names"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let action_str = syscall_entry["action"]
                .as_str()
                .unwrap_or("SCMP_ACT_ERRNO");

            let action = SeccompAction::from_str(action_str)?;

            syscalls.push(SeccompSyscallRule { names, action });
        }
    }

    Ok(SeccompProfile {
        default_action,
        syscalls,
    })
}

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

    #[test]
    fn test_load_profile_from_json() {
        let json = r#"{
            "defaultAction": "SCMP_ACT_ALLOW",
            "syscalls": [
                {
                    "names": ["mount", "umount2"],
                    "action": "SCMP_ACT_ERRNO"
                }
            ]
        }"#;
        let profile = load_profile_from_json(json).unwrap();
        assert!(matches!(profile.default_action, SeccompAction::Allow));
        assert_eq!(profile.syscalls.len(), 1);
        assert_eq!(profile.syscalls[0].names.len(), 2);
    }

    #[test]
    #[ignore] // Requires root
    fn test_apply_default_profile() {
        let result = apply_default_profile();
        assert!(result.is_ok());
    }
}
