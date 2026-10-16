// Linux raw Bluetooth HCI socket wrapper for Raspberry Pi OS

use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::Command;

pub const AF_BLUETOOTH: libc::c_int = 31;
pub const BTPROTO_HCI: libc::c_int = 1;
pub const SOL_HCI: libc::c_int = 0;
pub const HCI_FILTER: libc::c_int = 2;

pub const HCI_EVENT_PKT: u8 = 0x04;
pub const EVT_LE_META_EVENT: u8 = 0x3E;
pub const EVT_LE_ADVERTISING_REPORT: u8 = 0x02;

#[repr(C)]
struct SockAddrHci {
    hci_family: libc::sa_family_t,
    hci_dev: u16,
    hci_channel: u16,
}

#[repr(C)]
struct HciFilter {
    type_mask: u32,
    event_mask: [u32; 2],
    opcode: u16,
}

pub struct HciSocket {
    fd: RawFd,
}

impl HciSocket {
    /// Opens and binds a raw HCI socket to the specified Bluetooth adapter (e.g. "hci0")
    pub fn open(adapter: &str) -> io::Result<Self> {
        // Ensure adapter is UP
        let _ = Command::new("hciconfig").args([adapter, "up"]).output();

        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_RAW, BTPROTO_HCI) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let sa = SockAddrHci {
            hci_family: AF_BLUETOOTH as libc::sa_family_t,
            hci_dev: 0, // hci0
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
        // LE Set Scan Parameters (passive scan, 10ms interval / 10ms window)
        let scan_params_cmd = [
            0x01, 0x0B, 0x20, 7, // OCF 0x000B, OGF 0x08, Len 7
            0x00, // Passive scan
            0x10, 0x00, // Scan interval: 10ms
            0x10, 0x00, // Scan window: 10ms
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
            0x01, 0x0C, 0x20, 2, // OCF 0x000C, OGF 0x08, Len 2
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
