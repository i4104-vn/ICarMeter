#![allow(dead_code, unused_imports)]

use std::sync::atomic::{AtomicBool, Ordering};

pub static GLOBAL_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn is_app_running() -> bool {
    GLOBAL_RUNNING.load(Ordering::SeqCst)
}

pub fn request_app_exit() {
    GLOBAL_RUNNING.store(false, Ordering::SeqCst);
}

#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

#[cfg(unix)]
const KDSETMODE: libc::c_ulong = 0x4B3A;
#[cfg(unix)]
const KD_TEXT: libc::c_ulong = 0x00;
#[cfg(unix)]
const KD_GRAPHICS: libc::c_ulong = 0x01;

#[cfg(unix)]
extern "C" fn signal_handler(_sig: libc::c_int) {
    GLOBAL_RUNNING.store(false, Ordering::SeqCst);
}

pub struct TtyManager {
    #[cfg(unix)]
    tty_fd: Option<RawFd>,
    #[cfg(unix)]
    orig_termios: Option<libc::termios>,
    #[cfg(unix)]
    graphics_mode_set: bool,
}

#[cfg(unix)]
impl TtyManager {
    pub fn new() -> Self {
        GLOBAL_RUNNING.store(true, Ordering::SeqCst);

        // Register SIGINT / SIGTERM handlers so terminal state is cleanly restored
        unsafe {
            libc::signal(libc::SIGINT, signal_handler as usize);
            libc::signal(libc::SIGTERM, signal_handler as usize);
        }

        let mut manager = Self {
            tty_fd: None,
            orig_termios: None,
            graphics_mode_set: false,
        };

        // Attempt to find a suitable TTY file descriptor (controlling tty, active console, or stdin)
        let candidates = ["/dev/tty", "/dev/tty0", "/dev/tty1", "/dev/console"];
        let mut chosen_fd: Option<RawFd> = None;

        for dev in &candidates {
            if let Ok(file) = OpenOptions::new().read(true).write(true).open(dev) {
                let fd = file.as_raw_fd();
                std::mem::forget(file); // Keep file descriptor open for process lifetime
                chosen_fd = Some(fd);
                break;
            }
        }

        if chosen_fd.is_none() && unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
            chosen_fd = Some(libc::STDIN_FILENO);
        }

        if let Some(fd) = chosen_fd {
            manager.tty_fd = Some(fd);

            // 1. Hide terminal cursor & clear console screen
            print!("\x1b[?25l\x1b[2J\x1b[H");
            let _ = std::io::stdout().flush();

            // 2. Set graphics mode to silence kernel fbcon text output
            unsafe {
                if libc::ioctl(fd, KDSETMODE, KD_GRAPHICS) == 0 {
                    manager.graphics_mode_set = true;
                }
            }

            // 3. Save original terminal settings and switch to raw non-blocking input
            unsafe {
                let mut termios: libc::termios = std::mem::zeroed();
                if libc::tcgetattr(fd, &mut termios) == 0 {
                    manager.orig_termios = Some(termios);

                    let mut raw = termios;
                    // Disable echo, canonical mode, extended input processing, and signal chars
                    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN);
                    // Non-blocking read (return immediately if no input available)
                    raw.c_cc[libc::VMIN] = 0;
                    raw.c_cc[libc::VTIME] = 0;

                    libc::tcsetattr(fd, libc::TCSANOW, &raw);
                }
            }
        }

        manager
    }

    /// Polls non-blocking keyboard input. Returns true if user requested exit ('q', 'Q', Esc, Ctrl+C).
    pub fn poll_exit(&self) -> bool {
        if !is_app_running() {
            return true;
        }

        if let Some(fd) = self.tty_fd {
            let mut buf = [0u8; 16];
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n > 0 {
                for &b in &buf[..n as usize] {
                    // 'q', 'Q', ESC (0x1B), or Ctrl+C (0x03)
                    if b == b'q' || b == b'Q' || b == 0x1B || b == 0x03 {
                        request_app_exit();
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(unix)]
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
            // Restore terminal cursor and clear screen
            print!("\x1b[?25h\x1b[2J\x1b[H");
            let _ = std::io::stdout().flush();
        }
    }
}

#[cfg(not(unix))]
impl TtyManager {
    pub fn new() -> Self {
        GLOBAL_RUNNING.store(true, Ordering::SeqCst);
        Self {}
    }

    pub fn poll_exit(&self) -> bool {
        !is_app_running()
    }
}
