use nix::unistd::pipe;
use std::os::fd::{AsRawFd, RawFd};

use qcker_common::error::{QckerError, Result};

/// Container I/O configuration
pub struct ContainerIo {
    pub stdin_fd: Option<RawFd>,
    pub stdout_fd: Option<RawFd>,
    pub stderr_fd: Option<RawFd>,
}

/// Create pipes for container I/O
pub fn create_pipes() -> Result<(ContainerIo, ContainerIo)> {
    let (stdin_read, stdin_write) = pipe()
        .map_err(|e| QckerError::Process(format!("Failed to create stdin pipe: {}", e)))?;

    let (stdout_read, stdout_write) = pipe()
        .map_err(|e| QckerError::Process(format!("Failed to create stdout pipe: {}", e)))?;

    let (stderr_read, stderr_write) = pipe()
        .map_err(|e| QckerError::Process(format!("Failed to create stderr pipe: {}", e)))?;

    let parent_io = ContainerIo {
        stdin_fd: Some(stdin_write.as_raw_fd()),
        stdout_fd: Some(stdout_read.as_raw_fd()),
        stderr_fd: Some(stderr_read.as_raw_fd()),
    };

    let child_io = ContainerIo {
        stdin_fd: Some(stdin_read.as_raw_fd()),
        stdout_fd: Some(stdout_write.as_raw_fd()),
        stderr_fd: Some(stderr_write.as_raw_fd()),
    };

    Ok((parent_io, child_io))
}

/// Redirect container I/O to /dev/null
pub fn redirect_to_null() -> Result<()> {
    let dev_null = nix::fcntl::open(
        "/dev/null",
        nix::fcntl::OFlag::O_RDWR,
        nix::sys::stat::Mode::empty(),
    )
    .map_err(|e| QckerError::Process(format!("Failed to open /dev/null: {}", e)))?;

    unsafe {
        libc::dup2(dev_null.as_raw_fd(), 0);
        libc::dup2(dev_null.as_raw_fd(), 1);
        libc::dup2(dev_null.as_raw_fd(), 2);
    }

    Ok(())
}

/// Close pipe file descriptors
pub fn close_pipes(io: &ContainerIo) -> Result<()> {
    if let Some(fd) = io.stdin_fd {
        unsafe { libc::close(fd); }
    }
    if let Some(fd) = io.stdout_fd {
        unsafe { libc::close(fd); }
    }
    if let Some(fd) = io.stderr_fd {
        unsafe { libc::close(fd); }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pipes() {
        let (parent_io, child_io) = create_pipes().unwrap();
        assert!(parent_io.stdin_fd.is_some());
        assert!(parent_io.stdout_fd.is_some());
        assert!(parent_io.stderr_fd.is_some());
        assert!(child_io.stdin_fd.is_some());
        assert!(child_io.stdout_fd.is_some());
        assert!(child_io.stderr_fd.is_some());

        // Clean up
        close_pipes(&parent_io).unwrap();
        close_pipes(&child_io).unwrap();
    }
}
