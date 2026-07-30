use qcker_common::error::{QckerError, Result};
use serde::{Deserialize, Serialize};

const BPF_LD: u16 = 0x00;
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_JMP: u16 = 0x05;
const BPF_JEQ: u16 = 0x10;
const BPF_K: u16 = 0x00;
const BPF_RET: u16 = 0x06;

const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;
const SECCOMP_RET_ERRNO: u32 = 0x00050000;
const SECCOMP_RET_KILL: u32 = 0x00000000;

const SECCOMP_SET_MODE_FILTER: u64 = 1;

const SYS_SECCOMP: i64 = 317;

const SYSCALL_PTRACE: u32 = 101;
const SYSCALL_MOUNT: u32 = 165;
const SYSCALL_UMOUNT2: u32 = 166;
const SYSCALL_KEXEC_LOAD: u32 = 246;
const SYSCALL_OPEN_BY_HANDLE_AT: u32 = 265;
const SYSCALL_INIT_MODULE: u32 = 175;
const SYSCALL_FINIT_MODULE: u32 = 313;
const SYSCALL_DELETE_MODULE: u32 = 176;
const SYSCALL_CREATE_MODULE: u32 = 174;
const SYSCALL_GET_KERNEL_SYMS: u32 = 177;
const SYSCALL_PERF_EVENT_OPEN: u32 = 298;
const SYSCALL_PROCESS_VM_READV: u32 = 310;
const SYSCALL_PROCESS_VM_WRITEV: u32 = 311;
const SYSCALL_NFSSERVCTL: u32 = 180;
const SYSCALL_FANOTIFY_INIT: u32 = 300;
const SYSCALL_KEYCTL: u32 = 250;
const SYSCALL_ADD_KEY: u32 = 248;
const SYSCALL_REQUEST_KEY: u32 = 249;
const SYSCALL_CLOCK_SETTIME: u32 = 227;
const SYSCALL_CLOCK_ADJTIME: u32 = 226;
const SYSCALL_SETHOSTNAME: u32 = 170;
const SYSCALL_SETDOMAINNAME: u32 = 171;
const SYSCALL_REBOOT: u32 = 169;
const SYSCALL_SWAPON: u32 = 167;
const SYSCALL_SWAPOFF: u32 = 168;
const SYSCALL_SYSCTL: u32 = 156;
const SYSCALL_SYSFS: u32 = 139;
const SYSCALL_USELIB: u32 = 134;
const SYSCALL_IOPL: u32 = 172;
const SYSCALL_IOPERM: u32 = 173;
const SYSCALL_VHANGUP: u32 = 153;
const SYSCALL_PIVOT_ROOT: u32 = 217;

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

fn default_blocked_syscalls() -> Vec<u32> {
    vec![
        SYSCALL_PTRACE,
        SYSCALL_MOUNT,
        SYSCALL_UMOUNT2,
        SYSCALL_KEXEC_LOAD,
        SYSCALL_OPEN_BY_HANDLE_AT,
        SYSCALL_INIT_MODULE,
        SYSCALL_FINIT_MODULE,
        SYSCALL_DELETE_MODULE,
        SYSCALL_CREATE_MODULE,
        SYSCALL_GET_KERNEL_SYMS,
        SYSCALL_PERF_EVENT_OPEN,
        SYSCALL_PROCESS_VM_READV,
        SYSCALL_PROCESS_VM_WRITEV,
        SYSCALL_NFSSERVCTL,
        SYSCALL_FANOTIFY_INIT,
        SYSCALL_KEYCTL,
        SYSCALL_ADD_KEY,
        SYSCALL_REQUEST_KEY,
        SYSCALL_CLOCK_SETTIME,
        SYSCALL_CLOCK_ADJTIME,
        SYSCALL_SETHOSTNAME,
        SYSCALL_SETDOMAINNAME,
        SYSCALL_REBOOT,
        SYSCALL_SWAPON,
        SYSCALL_SWAPOFF,
        SYSCALL_SYSCTL,
        SYSCALL_SYSFS,
        SYSCALL_USELIB,
        SYSCALL_IOPL,
        SYSCALL_IOPERM,
        SYSCALL_VHANGUP,
        SYSCALL_PIVOT_ROOT,
    ]
}

fn syscall_name_to_nr(name: &str) -> Option<u32> {
    match name {
        "ptrace" => Some(SYSCALL_PTRACE),
        "mount" => Some(SYSCALL_MOUNT),
        "umount2" => Some(SYSCALL_UMOUNT2),
        "kexec_load" => Some(SYSCALL_KEXEC_LOAD),
        "open_by_handle_at" => Some(SYSCALL_OPEN_BY_HANDLE_AT),
        "init_module" => Some(SYSCALL_INIT_MODULE),
        "finit_module" => Some(SYSCALL_FINIT_MODULE),
        "delete_module" => Some(SYSCALL_DELETE_MODULE),
        "create_module" => Some(SYSCALL_CREATE_MODULE),
        "get_kernel_syms" => Some(SYSCALL_GET_KERNEL_SYMS),
        "perf_event_open" => Some(SYSCALL_PERF_EVENT_OPEN),
        "process_vm_readv" => Some(SYSCALL_PROCESS_VM_READV),
        "process_vm_writev" => Some(SYSCALL_PROCESS_VM_WRITEV),
        "nfsservctl" => Some(SYSCALL_NFSSERVCTL),
        "fanotify_init" => Some(SYSCALL_FANOTIFY_INIT),
        "keyctl" => Some(SYSCALL_KEYCTL),
        "add_key" => Some(SYSCALL_ADD_KEY),
        "request_key" => Some(SYSCALL_REQUEST_KEY),
        "clock_settime" => Some(SYSCALL_CLOCK_SETTIME),
        "clock_adjtime" => Some(SYSCALL_CLOCK_ADJTIME),
        "sethostname" => Some(SYSCALL_SETHOSTNAME),
        "setdomainname" => Some(SYSCALL_SETDOMAINNAME),
        "reboot" => Some(SYSCALL_REBOOT),
        "swapon" => Some(SYSCALL_SWAPON),
        "swapoff" => Some(SYSCALL_SWAPOFF),
        "_sysctl" => Some(SYSCALL_SYSCTL),
        "sysfs" => Some(SYSCALL_SYSFS),
        "uselib" => Some(SYSCALL_USELIB),
        "iopl" => Some(SYSCALL_IOPL),
        "ioperm" => Some(SYSCALL_IOPERM),
        "vhangup" => Some(SYSCALL_VHANGUP),
        "pivot_root" => Some(SYSCALL_PIVOT_ROOT),
        "bpf" => Some(321),
        "kcmp" => Some(306),
        "kexec_file_load" => Some(322),
        "lookup_dcookie" => Some(212),
        "move_pages" => Some(317),
        "personality" => Some(135),
        "set_mempolicy" => Some(238),
        "unshare" => Some(272),
        "userfaultfd" => Some(323),
        "ustat" => Some(136),
        _ => None,
    }
}

fn install_filter(filter: &[libc::sock_filter]) -> Result<()> {
    let mut fprog = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_ptr() as *mut libc::sock_filter,
    };

    unsafe {
        let ret = libc::syscall(
            SYS_SECCOMP,
            SECCOMP_SET_MODE_FILTER,
            0u64,
            &mut fprog as *mut libc::sock_fprog,
        );
        if ret != 0 {
            return Err(QckerError::Seccomp(format!(
                "Failed to install seccomp filter: {}",
                std::io::Error::last_os_error()
            )));
        }
    }

    Ok(())
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

    apply_seccomp_filter()
}

fn apply_seccomp_filter() -> Result<()> {
    let blocked = default_blocked_syscalls();
    let n = blocked.len();
    let mut filter = Vec::with_capacity(n + 2);

    filter.push(libc::sock_filter {
        code: BPF_LD | BPF_W | BPF_ABS,
        jt: 0,
        jf: 0,
        k: 0,
    });

    for (i, nr) in blocked.iter().enumerate() {
        let jt = (n - i + 1) as u8;
        filter.push(libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt,
            jf: 0,
            k: *nr,
        });
    }

    filter.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_ALLOW,
    });

    filter.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_ERRNO | 1,
    });

    install_filter(&filter)
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

    let mut blocked_errno: Vec<u32> = Vec::new();
    let mut blocked_kill: Vec<u32> = Vec::new();

    for rule in &profile.syscalls {
        match rule.action {
            SeccompAction::Errno => {
                for name in &rule.names {
                    if let Some(nr) = syscall_name_to_nr(name) {
                        blocked_errno.push(nr);
                    }
                }
            }
            SeccompAction::Kill => {
                for name in &rule.names {
                    if let Some(nr) = syscall_name_to_nr(name) {
                        blocked_kill.push(nr);
                    }
                }
            }
            _ => {}
        }
    }

    let default_ret = match profile.default_action {
        SeccompAction::Allow => SECCOMP_RET_ALLOW,
        SeccompAction::Kill => SECCOMP_RET_KILL,
        SeccompAction::Errno => SECCOMP_RET_ERRNO | 1,
        _ => SECCOMP_RET_ALLOW,
    };

    let errno_count = blocked_errno.len();
    let kill_count = blocked_kill.len();
    let total = errno_count + kill_count;

    let errno_ret_pos = total + 1;
    let kill_ret_pos = total + 2;

    let mut filter = Vec::with_capacity(total + 4);

    filter.push(libc::sock_filter {
        code: BPF_LD | BPF_W | BPF_ABS,
        jt: 0,
        jf: 0,
        k: 0,
    });

    for (i, nr) in blocked_errno.iter().enumerate() {
        let jt = (errno_ret_pos - (1 + i + 1)) as u8;
        filter.push(libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt,
            jf: 0,
            k: *nr,
        });
    }

    for (j, nr) in blocked_kill.iter().enumerate() {
        let jt = (kill_ret_pos - (1 + errno_count + j + 1)) as u8;
        filter.push(libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt,
            jf: 0,
            k: *nr,
        });
    }

    filter.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: default_ret,
    });

    filter.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_ERRNO | 1,
    });

    filter.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_KILL,
    });

    install_filter(&filter)
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
    #[ignore]
    fn test_apply_default_profile() {
        let result = apply_default_profile();
        assert!(result.is_ok());
    }
}
