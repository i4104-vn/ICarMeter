//! RFCOMM stream wrapper for file descriptors received through BlueZ Profile1.

use std::io::{self, Read, Write};
use std::os::fd::{IntoRawFd, OwnedFd, RawFd};
use std::time::Duration;

pub struct RfcommStream {
    fd: RawFd,
    peer_addr: String,
}

impl RfcommStream {
    pub fn from_owned_fd(fd: OwnedFd, peer_addr: String) -> Self {
        Self {
            fd: fd.into_raw_fd(),
            peer_addr,
        }
    }

    pub fn peer_addr(&self) -> &str {
        &self.peer_addr
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let new_fd = unsafe { libc::dup(self.fd) };
        if new_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            fd: new_fd,
            peer_addr: self.peer_addr.clone(),
        })
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        let tv = match timeout {
            Some(d) => libc::timeval {
                tv_sec: d.as_secs() as libc::time_t,
                tv_usec: d.subsec_micros() as libc::suseconds_t,
            },
            None => libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        };
        let res = unsafe {
            libc::setsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &tv as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as libc::socklen_t,
            )
        };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Read for RfcommStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as usize)
        }
    }
}

impl Write for RfcommStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for RfcommStream {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
