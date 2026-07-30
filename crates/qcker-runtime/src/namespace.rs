use nix::sched::{unshare, CloneFlags};
use std::fs;
use std::path::Path;

use crate::spec::{NamespaceConfig, NamespaceType};
use qcker_common::error::{QckerError, Result};

pub fn setup_namespaces(namespaces: &[NamespaceConfig], rootless: bool) -> Result<()> {
    let mut flags = CloneFlags::empty();

    for ns in namespaces {
        match ns.r#type {
            NamespaceType::Pid => flags |= CloneFlags::CLONE_NEWPID,
            NamespaceType::Network => flags |= CloneFlags::CLONE_NEWNET,
            NamespaceType::Mount => flags |= CloneFlags::CLONE_NEWNS,
            NamespaceType::Uts => flags |= CloneFlags::CLONE_NEWUTS,
            NamespaceType::Ipc => flags |= CloneFlags::CLONE_NEWIPC,
            NamespaceType::User => {
                if rootless {
                    flags |= CloneFlags::CLONE_NEWUSER;
                }
            }
            NamespaceType::Cgroup => flags |= CloneFlags::CLONE_NEWCGROUP,
        }
    }

    unshare(flags).map_err(|e| QckerError::Namespace(format!("Failed to unshare: {}", e)))?;

    Ok(())
}

pub fn setup_user_namespace_mapping(uid: u32, gid: u32) -> Result<()> {
    let uid_map = format!("0 {} 1", uid);
    fs::write("/proc/self/uid_map", &uid_map)
        .map_err(|e| QckerError::Namespace(format!("Failed to write uid_map: {}", e)))?;

    fs::write("/proc/self/setgroups", "deny")
        .map_err(|e| QckerError::Namespace(format!("Failed to write setgroups: {}", e)))?;

    let gid_map = format!("0 {} 1", gid);
    fs::write("/proc/self/gid_map", &gid_map)
        .map_err(|e| QckerError::Namespace(format!("Failed to write gid_map: {}", e)))?;

    Ok(())
}

pub fn set_container_hostname(hostname: &str) -> Result<()> {
    nix::unistd::sethostname(hostname)
        .map_err(|e| QckerError::Namespace(format!("Failed to set hostname: {}", e)))?;
    Ok(())
}

pub fn enter_namespace(pid: i32, ns_type: NamespaceType) -> Result<()> {
    let ns_path = format!("/proc/{}/ns/{}", pid, ns_type_name(&ns_type));
    let ns_file = Path::new(&ns_path);

    if !ns_file.exists() {
        return Err(QckerError::Namespace(format!(
            "Namespace file not found: {}",
            ns_path
        )));
    }

    let ns_path_c = std::ffi::CString::new(ns_path.as_str()).unwrap();
    let fd = unsafe { libc::open(ns_path_c.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(QckerError::Namespace(format!(
            "Failed to open namespace: {}",
            std::io::Error::last_os_error()
        )));
    }

    let ret = unsafe { libc::setns(fd, 0) };
    if ret != 0 {
        unsafe { libc::close(fd); }
        return Err(QckerError::Namespace(format!(
            "Failed to enter namespace: {}",
            std::io::Error::last_os_error()
        )));
    }

    unsafe { libc::close(fd); }
    Ok(())
}

fn ns_type_name(ns_type: &NamespaceType) -> &'static str {
    match ns_type {
        NamespaceType::Pid => "pid",
        NamespaceType::Network => "net",
        NamespaceType::Mount => "mnt",
        NamespaceType::Uts => "uts",
        NamespaceType::Ipc => "ipc",
        NamespaceType::User => "user",
        NamespaceType::Cgroup => "cgroup",
    }
}

pub fn user_namespaces_supported() -> bool {
    let flags = CloneFlags::CLONE_NEWUSER;
    unshare(flags).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns_type_name() {
        assert_eq!(ns_type_name(&NamespaceType::Pid), "pid");
        assert_eq!(ns_type_name(&NamespaceType::Network), "net");
        assert_eq!(ns_type_name(&NamespaceType::Mount), "mnt");
    }
}
