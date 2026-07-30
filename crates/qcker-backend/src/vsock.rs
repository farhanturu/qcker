use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::io::RawFd;

use super::protocol::{deserialize_message, serialize_message, MAX_MESSAGE_SIZE};

pub struct SyncVsockChannel {
    fd: RawFd,
}

impl SyncVsockChannel {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }

    pub fn send<T: Serialize>(&self, msg: &T) -> Result<(), String> {
        let data = serialize_message(msg)?;
        unsafe {
            let written = libc::write(
                self.fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
            );
            if written < 0 {
                return Err(format!("vsock write failed: {}", std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }

    pub fn recv<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(format!("Message too large: {} bytes", len));
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

        self.recv().map_err(|e| RecvTimeoutError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
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
                return Err(format!("vsock read failed: {}", std::io::Error::last_os_error()));
            }
            if n == 0 {
                return Err("vsock connection closed".to_string());
            }
            total += n as usize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::HostToVm;

    #[test]
    fn test_serialize_for_vsock() {
        let msg = HostToVm::Ping;
        let data = serialize_message(&msg).unwrap();
        assert!(data.len() > 4);
    }
}
