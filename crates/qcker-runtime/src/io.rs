use nix::unistd::pipe;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use qcker_common::error::{QckerError, Result};

pub struct ContainerIo {
    pub stdin_fd: Option<OwnedFd>,
    pub stdout_fd: Option<OwnedFd>,
    pub stderr_fd: Option<OwnedFd>,
}

unsafe fn own_fd(raw: std::os::fd::RawFd) -> OwnedFd {
    OwnedFd::from_raw_fd(raw)
}

pub fn create_pipes() -> Result<(ContainerIo, ContainerIo)> {
    let (stdin_read, stdin_write) = pipe()
        .map_err(|e| QckerError::process(format!("Failed to create stdin pipe: {}", e)))?;

    let (stdout_read, stdout_write) = pipe()
        .map_err(|e| QckerError::process(format!("Failed to create stdout pipe: {}", e)))?;

    let (stderr_read, stderr_write) = pipe()
        .map_err(|e| QckerError::process(format!("Failed to create stderr pipe: {}", e)))?;

    let parent_io = ContainerIo {
        stdin_fd: Some(unsafe { own_fd(stdin_write) }),
        stdout_fd: Some(unsafe { own_fd(stdout_read) }),
        stderr_fd: Some(unsafe { own_fd(stderr_read) }),
    };

    let child_io = ContainerIo {
        stdin_fd: Some(unsafe { own_fd(stdin_read) }),
        stdout_fd: Some(unsafe { own_fd(stdout_write) }),
        stderr_fd: Some(unsafe { own_fd(stderr_write) }),
    };

    Ok((parent_io, child_io))
}

pub fn redirect_to_null() -> Result<()> {
    let dev_null = nix::fcntl::open(
        "/dev/null",
        nix::fcntl::OFlag::O_RDWR,
        nix::sys::stat::Mode::empty(),
    )
    .map_err(|e| QckerError::process(format!("Failed to open /dev/null: {}", e)))?;

    unsafe {
        libc::dup2(dev_null.as_raw_fd(), 0);
        libc::dup2(dev_null.as_raw_fd(), 1);
        libc::dup2(dev_null.as_raw_fd(), 2);
    }

    Ok(())
}

pub fn close_pipes(io: &mut ContainerIo) {
    io.stdin_fd.take();
    io.stdout_fd.take();
    io.stderr_fd.take();
}

pub fn stdin_fd(io: &ContainerIo) -> Option<std::os::fd::RawFd> {
    io.stdin_fd.as_ref().map(|fd| fd.as_raw_fd())
}

pub fn stdout_fd(io: &ContainerIo) -> Option<std::os::fd::RawFd> {
    io.stdout_fd.as_ref().map(|fd| fd.as_raw_fd())
}

pub fn stderr_fd(io: &ContainerIo) -> Option<std::os::fd::RawFd> {
    io.stderr_fd.as_ref().map(|fd| fd.as_raw_fd())
}

