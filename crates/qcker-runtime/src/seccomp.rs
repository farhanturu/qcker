use libc::{self, *};

use qcker_common::error::{QckerError, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum SeccompAction {
    Allow,
    Errno,
    Kill,
    Trace,
    Log,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OciSeccompProfile {
    pub default_action: SeccompAction,
    pub syscalls: Vec<OciSeccompSyscallRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OciSeccompSyscallRule {
    pub names: Vec<String>,
    pub action: SeccompAction,
}

const AUDIT_ARCH_X86_64: u32 = 0xC000003E;

const fn bpf_stmt(code: __u16, k: __u32) -> sock_filter {
    sock_filter { code, jt: 0, jf: 0, k }
}

const fn bpf_jmp(code: __u16, jf: __u8, k: __u32, jt: __u8) -> sock_filter {
    sock_filter { code, jt, jf, k }
}

fn build_bpf_program(blocked_nrs: &[i32]) -> Vec<sock_filter> {
    let mut prog: Vec<sock_filter> = Vec::new();
    prog.push(bpf_stmt(0x20, 0));
    prog.push(bpf_jmp(libc::BPF_JMP as u16, 1, AUDIT_ARCH_X86_64, 0));
    for &nr in blocked_nrs {
        prog.push(libc::sock_filter { code: (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, jt: 0, jf: 0, k: (nr as u32) });
        prog.push(libc::sock_filter { code: (libc::BPF_JMP | libc::BPF_JEQ) as u16, jt: 0, jf: 0, k: 0 });
        prog.push(libc::sock_filter { code: (libc::BPF_RET | libc::BPF_K | SECCOMP_RET_ERRNO) as u16, jt: 0, jf: 0, k: libc::EPERM as u32 });
    }
    prog.push(libc::sock_filter { code: (libc::BPF_RET | libc::BPF_K | SECCOMP_RET_ALLOW) as u16, jt: 0, jf: 0, k: 0 });
    prog
}

unsafe fn install_filter(prog: &Vec<sock_filter>) -> Result<()> {
    let fprog = sock_fprog {
        len: prog.len() as libc::c_ushort,
        filter: prog.as_ptr() as *mut _,
    };
    let ret = libc::syscall(SYS_seccomp, SECCOMP_SET_MODE_FILTER, 0, &fprog);
    if ret != 0 {
        return Err(QckerError::seccomp(format!("seccomp filter install failed: {}", std::io::Error::last_os_error())));
    }
    Ok(())
}

fn syscall_nr(name: &str) -> Option<i32> {
    match name {
        "ptrace" => Some(libc::SYS_ptrace as i32),
        "mount" => Some(libc::SYS_mount as i32),
        "umount2" => Some(libc::SYS_umount2 as i32),
        "kexec_load" => Some(libc::SYS_kexec_load as i32),
        "open_by_handle_at" => Some(libc::SYS_open_by_handle_at as i32),
        "init_module" => Some(libc::SYS_init_module as i32),
        "finit_module" => Some(libc::SYS_finit_module as i32),
        "delete_module" => Some(libc::SYS_delete_module as i32),
        "create_module" => Some(libc::SYS_create_module as i32),
        "get_kernel_syms" => Some(libc::SYS_get_kernel_syms as i32),
        "perf_event_open" => Some(libc::SYS_perf_event_open as i32),
        "process_vm_readv" => Some(libc::SYS_process_vm_readv as i32),
        "process_vm_writev" => Some(libc::SYS_process_vm_writev as i32),
        "nfsservctl" => Some(libc::SYS_nfsservctl as i32),
        "fanotify_init" => Some(libc::SYS_fanotify_init as i32),
        "keyctl" => Some(libc::SYS_keyctl as i32),
        "add_key" => Some(libc::SYS_add_key as i32),
        "request_key" => Some(libc::SYS_request_key as i32),
        "clock_settime" => Some(libc::SYS_clock_settime as i32),
        "clock_adjtime" => Some(libc::SYS_clock_adjtime as i32),
        "sethostname" => Some(libc::SYS_sethostname as i32),
        "setdomainname" => Some(libc::SYS_setdomainname as i32),
        "reboot" => Some(libc::SYS_reboot as i32),
        "swapon" => Some(libc::SYS_swapon as i32),
        "swapoff" => Some(libc::SYS_swapoff as i32),
        "_sysctl" => Some(libc::SYS__sysctl as i32),
        "sysfs" => Some(libc::SYS_sysfs as i32),
        "uselib" => Some(libc::SYS_uselib as i32),
        "iopl" => Some(libc::SYS_iopl as i32),
        "ioperm" => Some(libc::SYS_ioperm as i32),
        "vhangup" => Some(libc::SYS_vhangup as i32),
        "pivot_root" => Some(libc::SYS_pivot_root as i32),
        "unshare" => Some(libc::SYS_unshare as i32),
        _ => None,
    }
}

fn resolve_blocklist_names(syscalls: &[&str]) -> Vec<i32> {
    syscalls.iter().filter_map(|n| syscall_nr(n)).collect()
}

pub fn apply_seccomp_filter() -> Result<()> {
    let blocked = vec![
        "ptrace", "mount", "umount2", "kexec_load", "open_by_handle_at", "init_module",
        "finit_module", "delete_module", "create_module", "get_kernel_syms",
        "perf_event_open", "process_vm_readv", "process_vm_writev", "nfsservctl",
        "fanotify_init", "keyctl", "add_key", "request_key", "clock_settime",
        "clock_adjtime", "sethostname", "setdomainname", "reboot", "swapon",
        "swapoff", "_sysctl", "sysfs", "uselib", "iopl", "ioperm", "vhangup",
        "pivot_root", "unshare",
    ];
    let nr_vec = resolve_blocklist_names(&blocked);
    let prog = build_bpf_program(&nr_vec);
    unsafe { install_filter(&prog) }
}

pub fn apply_profile(profile: &OciSeccompProfile) -> Result<()> {
    let mut nrs: Vec<i32> = Vec::new();
    for rule in &profile.syscalls {
        nrs.extend(rule.names.iter().filter_map(|n| syscall_nr(n)));
    }
    let prog = build_bpf_program(&nrs);
    unsafe { install_filter(&prog) }
}

pub fn load_profile_from_json(json: &str) -> Result<OciSeccompProfile> {
    use serde_json::Value;
    let value: Value = serde_json::from_str(json)?;
    let default_action_str = value["defaultAction"]
        .as_str()
        .ok_or_else(|| QckerError::seccomp("Missing defaultAction".to_string()))?;
    let default_action = match default_action_str {
        "SCMP_ACT_ALLOW" => SeccompAction::Allow,
        "SCMP_ACT_ERRNO" => SeccompAction::Errno,
        "SCMP_ACT_KILL" => SeccompAction::Kill,
        "SCMP_ACT_TRACE" => SeccompAction::Trace,
        "SCMP_ACT_LOG" => SeccompAction::Log,
        _ => return Err(QckerError::seccomp(format!("Unknown action: {}", default_action_str))),
    };
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
            let action = match action_str {
                "SCMP_ACT_ALLOW" => SeccompAction::Allow,
                "SCMP_ACT_ERRNO" => SeccompAction::Errno,
                "SCMP_ACT_KILL" => SeccompAction::Kill,
                "SCMP_ACT_TRACE" => SeccompAction::Trace,
                "SCMP_ACT_LOG" => SeccompAction::Log,
                _ => SeccompAction::Errno,
            };
            syscalls.push(OciSeccompSyscallRule { names, action });
        }
    }
    Ok(OciSeccompProfile { default_action, syscalls })
}

#[allow(dead_code)]
fn create_restrictive_profile() -> OciSeccompProfile {
    OciSeccompProfile {
        default_action: SeccompAction::Allow,
        syscalls: vec![
            OciSeccompSyscallRule {
                names: vec![
                    "add_key".to_string(), "bpf".to_string(), "clock_settime".to_string(),
                    "create_module".to_string(), "delete_module".to_string(), "finit_module".to_string(),
                    "init_module".to_string(), "ioperm".to_string(), "iopl".to_string(), "kcmp".to_string(),
                    "kexec_file_load".to_string(), "kexec_load".to_string(), "keyctl".to_string(),
                    "lookup_dcookie".to_string(), "mount".to_string(), "move_pages".to_string(),
                    "nfsservctl".to_string(), "perf_event_open".to_string(), "personality".to_string(),
                    "pivot_root".to_string(), "process_vm_readv".to_string(), "process_vm_writev".to_string(),
                    "ptrace".to_string(), "reboot".to_string(), "request_key".to_string(),
                    "set_mempolicy".to_string(), "swapoff".to_string(), "swapon".to_string(),
                    "sysfs".to_string(), "_sysctl".to_string(), "umount2".to_string(), "unshare".to_string(),
                    "uselib".to_string(), "userfaultfd".to_string(), "ustat".to_string(),
                ],
                action: SeccompAction::Errno,
            },
        ],
    }
}

impl SeccompAction {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "SCMP_ACT_ALLOW" => Ok(SeccompAction::Allow),
            "SCMP_ACT_ERRNO" => Ok(SeccompAction::Errno),
            "SCMP_ACT_KILL" => Ok(SeccompAction::Kill),
            "SCMP_ACT_TRACE" => Ok(SeccompAction::Trace),
            "SCMP_ACT_LOG" => Ok(SeccompAction::Log),
            _ => Err(QckerError::seccomp(format!("Unknown action: {}", s))),
        }
    }
}

pub fn apply_default_profile() -> Result<()> {
    unsafe {
        let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(QckerError::seccomp(format!(
                "Failed to set NO_NEW_PRIVS: {}",
                std::io::Error::last_os_error()
            )));
        }
    }
    apply_seccomp_filter()
}

