// Linux raw Bluetooth HCI socket wrapper for Raspberry Pi OS

use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::models::bluetooth::address::AF_BLUETOOTH;
use crate::models::bluetooth::socket::{
    BTPROTO_HCI, HCI_EVENT_PKT, HCI_FILTER, HciFilter, SOL_HCI, SockAddrHci,
};

pub struct HciSocket {
    fd: RawFd,
}

impl HciSocket {
    /// Opens and binds a raw HCI socket to the specified Bluetooth adapter (e.g. "hci0")
    pub fn open(adapter: &str) -> io::Result<Self> {
        // Configure adapter power and discoverability through BlueZ D-Bus.
        crate::services::bluetooth::adapter::ensure_adapter_ready(adapter);

        let hci_dev = adapter
            .strip_prefix("hci")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid Bluetooth adapter name",
                )
            })?;

        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_RAW, BTPROTO_HCI) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let sa = SockAddrHci {
            hci_family: AF_BLUETOOTH as libc::sa_family_t,
            hci_dev,
            hci_channel: 0,
        };

        let bind_res = unsafe {
            libc::bind(
                fd,
                &sa as *const _ as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as libc::socklen_t,
            )
        };

        if bind_res < 0 {
            let err = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        let socket = Self { fd };
        socket.configure_le_filter()?;
        socket.enable_le_scan()?;

        Ok(socket)
    }

    /// Sets up HCI event filter to only receive LE Meta Events (Advertising Reports)
    fn configure_le_filter(&self) -> io::Result<()> {
        let filter = HciFilter {
            type_mask: 1 << HCI_EVENT_PKT,
            event_mask: [0xFFFF_FFFF, 0xFFFF_FFFF],
            opcode: 0,
        };

        let res = unsafe {
            libc::setsockopt(
                self.fd,
                SOL_HCI,
                HCI_FILTER,
                &filter as *const _ as *const libc::c_void,
                std::mem::size_of::<HciFilter>() as libc::socklen_t,
            )
        };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// Sends HCI command packets to start active/passive LE scanning
    fn enable_le_scan(&self) -> io::Result<()> {
        // LE Set Scan Parameters (passive scan, 100ms interval / 50ms window to allow simultaneous advertising)
        let scan_params_cmd = [
            0x01, 0x0B, 0x20, 7,    // OCF 0x000B, OGF 0x08, Len 7
            0x00, // Passive scan
            0xA0, 0x00, // Scan interval: 100ms (160 * 0.625ms)
            0x50, 0x00, // Scan window: 50ms (80 * 0.625ms)
            0x00, // Public address
            0x00, // All packets
        ];
        unsafe {
            libc::write(
                self.fd,
                scan_params_cmd.as_ptr() as *const libc::c_void,
                scan_params_cmd.len(),
            );
        }

        // LE Set Scan Enable (enable = 1, filter_duplicates = 0)
        let scan_enable_cmd = [
            0x01, 0x0C, 0x20, 2,    // OCF 0x000C, OGF 0x08, Len 2
            0x01, // Enable scanning
            0x00, // Filter duplicates: 0 (report continuous updates)
        ];
        unsafe {
            libc::write(
                self.fd,
                scan_enable_cmd.as_ptr() as *const libc::c_void,
                scan_enable_cmd.len(),
            );
        }

        Ok(())
    }

    /// Reads incoming raw HCI bytes
    pub fn read(&self, buf: &mut [u8]) -> isize {
        unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) }
    }
}

impl AsRawFd for HciSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for HciSocket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}
