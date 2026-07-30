use nix::pty::{openpty, OpenptyResult};
use nix::sys::termios::{self, SetArg};
use std::os::fd::{AsRawFd, BorrowedFd, RawFd};

use qcker_common::error::{QckerError, Result};

#[derive(Debug, Clone, Copy)]
pub enum TerminalMode {
    Interactive,
    NonInteractive,
}

pub struct Terminal {
    pub master_fd: RawFd,
    pub slave_fd: RawFd,
    pub mode: TerminalMode,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let OpenptyResult { master, slave } =
            openpty(None, None).map_err(|e| QckerError::Process(format!("Failed to open PTY: {}", e)))?;

        Ok(Self {
            master_fd: master.as_raw_fd(),
            slave_fd: slave.as_raw_fd(),
            mode: TerminalMode::Interactive,
        })
    }

    pub fn set_raw_mode(&self) -> Result<()> {
        let master_fd = unsafe { BorrowedFd::borrow_raw(self.master_fd) };
        let mut termios = termios::tcgetattr(master_fd)
            .map_err(|e| QckerError::Process(format!("Failed to get terminal attrs: {}", e)))?;

        termios::cfmakeraw(&mut termios);

        termios::tcsetattr(master_fd, SetArg::TCSANOW, &termios)
            .map_err(|e| QckerError::Process(format!("Failed to set terminal attrs: {}", e)))?;

        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let ret = libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &winsize);
            if ret != 0 {
                return Err(QckerError::Process(format!(
                    "Failed to resize terminal: {}",
                    std::io::Error::last_os_error()
                )));
            }
        }

        Ok(())
    }

    pub fn slave_fd(&self) -> RawFd {
        self.slave_fd
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.master_fd);
            libc::close(self.slave_fd);
        }
    }
}

pub fn proxy_terminal(master_fd: RawFd) -> Result<()> {
    use std::io::{self, Read, Write};

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buf = [0u8; 1024];

    let stdin_fd = unsafe { BorrowedFd::borrow_raw(0) };
    let orig_termios = termios::tcgetattr(stdin_fd)
        .map_err(|e| QckerError::Process(format!("Failed to get stdin attrs: {}", e)))?;

    let mut raw_termios = orig_termios.clone();
    termios::cfmakeraw(&mut raw_termios);
    termios::tcsetattr(stdin_fd, SetArg::TCSANOW, &raw_termios)
        .map_err(|e| QckerError::Process(format!("Failed to set stdin raw mode: {}", e)))?;

    loop {
        let mut poll_fds = [
            libc::pollfd {
                fd: 0, // stdin
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: master_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        let ret = unsafe { libc::poll(poll_fds.as_mut_ptr(), 2, -1) };
        if ret < 0 {
            break;
        }

        if poll_fds[0].revents & libc::POLLIN != 0 {
            match stdin.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    unsafe {
                        libc::write(master_fd, buf.as_ptr() as *const _, n);
                    }
                }
                Err(_) => break,
            }
        }

        if poll_fds[1].revents & libc::POLLIN != 0 {
            unsafe {
                let n = libc::read(master_fd, buf.as_mut_ptr() as *mut _, buf.len());
                if n <= 0 {
                    break;
                }
                if stdout.write_all(&buf[..n as usize]).is_err() {
                    break;
                }
                let _ = stdout.flush();
            }
        }
    }

    termios::tcsetattr(stdin_fd, SetArg::TCSANOW, &orig_termios)
        .map_err(|e| QckerError::Process(format!("Failed to restore terminal: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_mode() {
        let mode = TerminalMode::Interactive;
        assert!(matches!(mode, TerminalMode::Interactive));
    }
}
