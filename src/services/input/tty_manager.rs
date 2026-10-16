// Linux TTY Console Manager: configures KD_GRAPHICS console mode and hides cursor for direct framebuffer rendering

use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};

const KDSETMODE: libc::c_ulong = 0x4B3A;
const KD_TEXT: libc::c_ulong = 0x00;
const KD_GRAPHICS: libc::c_ulong = 0x01;

pub struct TtyManager {
    tty_fd: Option<RawFd>,
    orig_termios: Option<libc::termios>,
    graphics_mode_set: bool,
}

impl TtyManager {
    pub fn new() -> Self {
        let mut manager = Self {
            tty_fd: None,
            orig_termios: None,
            graphics_mode_set: false,
        };

        let fd = if let Ok(file) = OpenOptions::new().read(true).write(true).open("/dev/tty") {
            let fd = file.as_raw_fd();
            std::mem::forget(file);
            Some(fd)
        } else if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
            Some(libc::STDIN_FILENO)
        } else {
            None
        };

        if let Some(fd) = fd {
            manager.tty_fd = Some(fd);

            // 1. Hide terminal cursor
            print!("\x1b[?25l\x1b[2J\x1b[H");

            // 2. Set graphics mode to silence kernel fbcon text output
            unsafe {
                if libc::ioctl(fd, KDSETMODE, KD_GRAPHICS) == 0 {
                    manager.graphics_mode_set = true;
                }
            }

            // 3. Save terminal settings
            unsafe {
                let mut termios: libc::termios = std::mem::zeroed();
                if libc::tcgetattr(fd, &mut termios) == 0 {
                    manager.orig_termios = Some(termios);
                }
            }
        }

        manager
    }
}

impl Drop for TtyManager {
    fn drop(&mut self) {
        if let Some(fd) = self.tty_fd {
            unsafe {
                if self.graphics_mode_set {
                    libc::ioctl(fd, KDSETMODE, KD_TEXT);
                }
                if let Some(ref orig) = self.orig_termios {
                    libc::tcsetattr(fd, libc::TCSANOW, orig);
                }
            }
            print!("\x1b[?25h\x1b[2J\x1b[H");
        }
    }
}
