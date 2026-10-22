//! Low-level Linux Bluetooth RFCOMM socket primitives.

use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

pub const AF_BLUETOOTH: libc::c_int = 31;
pub const BTPROTO_RFCOMM: libc::c_int = 3;
pub const RFCOMM_CHANNEL_DEFAULT: u8 = 1;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct BdAddr {
    pub b: [u8; 6],
}

impl BdAddr {
    pub const ANY: Self = Self { b: [0; 6] };

    pub fn to_mac_string(&self) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.b[5], self.b[4], self.b[3], self.b[2], self.b[1], self.b[0]
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SockAddrRc {
    pub rc_family: libc::sa_family_t,
    pub rc_bdaddr: BdAddr,
    pub rc_channel: u8,
}

pub struct RfcommStream {
    fd: RawFd,
    peer_addr: String,
}

impl RfcommStream {
    pub fn new(fd: RawFd, peer_addr: String) -> Self {
        Self { fd, peer_addr }
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
            None => libc::timeval { tv_sec: 0, tv_usec: 0 },
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

impl AsRawFd for RfcommStream {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for RfcommStream {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

pub struct RfcommListener {
    fd: RawFd,
    channel: u8,
}

impl RfcommListener {
    pub fn bind(channel: u8) -> io::Result<Self> {
        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_STREAM, BTPROTO_RFCOMM) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let opt: libc::c_int = 1;
        unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &opt as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );
        }

        let addr = SockAddrRc {
            rc_family: AF_BLUETOOTH as libc::sa_family_t,
            rc_bdaddr: BdAddr::ANY,
            rc_channel: channel,
        };

        let bind_res = unsafe {
            libc::bind(
                fd,
                &addr as *const _ as *const libc::sockaddr,
                std::mem::size_of::<SockAddrRc>() as libc::socklen_t,
            )
        };

        if bind_res < 0 {
            let err = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        let listen_res = unsafe { libc::listen(fd, 4) };
        if listen_res < 0 {
            let err = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        Ok(Self { fd, channel })
    }

    pub fn accept(&self) -> io::Result<RfcommStream> {
        let mut client_addr = SockAddrRc {
            rc_family: 0,
            rc_bdaddr: BdAddr::ANY,
            rc_channel: 0,
        };
        let mut len = std::mem::size_of::<SockAddrRc>() as libc::socklen_t;

        let client_fd = unsafe {
            libc::accept(
                self.fd,
                &mut client_addr as *mut _ as *mut libc::sockaddr,
                &mut len,
            )
        };

        if client_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let peer_addr = client_addr.rc_bdaddr.to_mac_string();
        Ok(RfcommStream::new(client_fd, peer_addr))
    }

    pub fn channel(&self) -> u8 {
        self.channel
    }
}

impl Drop for RfcommListener {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
