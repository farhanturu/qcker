use serde::{Deserialize, Serialize};
use std::os::unix::io::RawFd;
use tracing::{debug, warn};

use super::protocol::{deserialize_message, serialize_message, MAX_MESSAGE_SIZE};

pub struct SyncVsockChannel {
    fd: RawFd,
}

impl SyncVsockChannel {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }

    pub fn connect(cid: u32, port: u32) -> Result<Self, String> {
        let fd = unsafe {
            let sock = libc::socket(
                libc::AF_VSOCK,
                libc::SOCK_STREAM | libc::SOCK_CLOEXEC,
                0,
            );
            if sock < 0 {
                return Err(format!(
                    "Failed to create vsock socket: {}",
                    std::io::Error::last_os_error()
                ));
            }

            let mut addr: libc::sockaddr_vm = std::mem::zeroed();
            addr.svm_family = libc::AF_VSOCK as u16;
            addr.svm_cid = cid;
            addr.svm_port = port;

            let addr_len = std::mem::size_of::<libc::sockaddr_vm>() as u32;
            let ret = libc::connect(
                sock,
                &addr as *const libc::sockaddr_vm as *const libc::sockaddr,
                addr_len,
            );
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                libc::close(sock);
                return Err(format!(
                    "Failed to connect vsock to CID:{} port:{}: {}",
                    cid, port, err
                ));
            }

            sock
        };

        debug!("Connected to vsock CID:{} port:{}", cid, port);
        Ok(Self { fd })
    }

    pub fn send<T: Serialize>(&self, msg: &T) -> Result<(), String> {
        let data = serialize_message(msg)?;
        let total_len = data.len();

        let mut written = 0;
        while written < total_len {
            let n = unsafe {
                libc::write(
                    self.fd,
                    data[written..].as_ptr() as *const libc::c_void,
                    total_len - written,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(format!("vsock write failed: {}", err));
            }
            written += n as usize;
        }

        debug!("Sent {} bytes via vsock", total_len);
        Ok(())
    }

    pub fn recv<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(format!(
                "Message too large: {} bytes (max: {} bytes)",
                len, MAX_MESSAGE_SIZE
            ));
        }

        let mut data = vec![0u8; len];
        self.read_exact(&mut data)?;

        deserialize_message(&data)
    }

    pub fn recv_timeout<T: for<'de> Deserialize<'de>>(
        &self,
        timeout: std::time::Duration,
    ) -> Result<T, RecvTimeoutError> {
        let mut pollfds = [libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        }];

        let timeout_ms = timeout.as_millis() as i32;
        let ret = unsafe { libc::poll(pollfds.as_mut_ptr(), 1, timeout_ms) };

        if ret < 0 {
            return Err(RecvTimeoutError::Io(std::io::Error::last_os_error()));
        }
        if ret == 0 {
            return Err(RecvTimeoutError::Timeout);
        }

        if pollfds[0].revents & (libc::POLLERR | libc::POLLHUP) != 0 {
            return Err(RecvTimeoutError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "vsock connection closed or error",
            )));
        }

        self.recv().map_err(|e| RecvTimeoutError::Io(std::io::Error::other(e)))
    }

    fn read_exact(&self, buf: &mut [u8]) -> Result<(), String> {
        let mut total = 0;
        while total < buf.len() {
            let n = unsafe {
                libc::read(
                    self.fd,
                    buf[total..].as_mut_ptr() as *mut libc::c_void,
                    buf.len() - total,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(format!("vsock read failed: {}", err));
            }
            if n == 0 {
                return Err("vsock connection closed by peer".to_string());
            }
            total += n as usize;
        }
        Ok(())
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn set_buffer_sizes(&self, send_size: usize, recv_size: usize) -> Result<(), String> {
        unsafe {
            let send_val = send_size as libc::c_int;
            let ret = libc::setsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_SNDBUF,
                &send_val as *const libc::c_int as *const libc::c_void,
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );
            if ret < 0 {
                warn!("Failed to set vsock send buffer size: {}", std::io::Error::last_os_error());
            }

            let recv_val = recv_size as libc::c_int;
            let ret = libc::setsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_RCVBUF,
                &recv_val as *const libc::c_int as *const libc::c_void,
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );
            if ret < 0 {
                warn!("Failed to set vsock recv buffer size: {}", std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

impl Drop for SyncVsockChannel {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

#[derive(Debug)]
pub enum RecvTimeoutError {
    Timeout,
    Io(std::io::Error),
}

impl std::fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvTimeoutError::Timeout => write!(f, "vsock receive timed out"),
            RecvTimeoutError::Io(e) => write!(f, "vsock IO error: {}", e),
        }
    }
}

impl std::error::Error for RecvTimeoutError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::HostToVm;

    #[test]
    fn test_serialize_for_vsock() {
        let msg = HostToVm::Ping;
        let data = serialize_message(&msg).unwrap();
        assert!(data.len() > 4);
        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(len as usize, data.len() - 4);
    }

    #[test]
    fn test_channel_new() {
        let channel = SyncVsockChannel::new(-1);
        assert_eq!(channel.fd(), -1);
    }

    #[test]
    fn test_recv_timeout_error_display() {
        let err = RecvTimeoutError::Timeout;
        assert_eq!(format!("{}", err), "vsock receive timed out");
    }
}
