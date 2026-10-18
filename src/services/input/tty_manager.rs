// Linux TTY Console Manager: configures KD_GRAPHICS console mode, hides cursor,
// enables raw non-blocking input, and handles graceful signal restoration for direct rendering on Debian / Raspberry Pi.

use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};

pub static GLOBAL_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn is_app_running() -> bool {
    GLOBAL_RUNNING.load(Ordering::SeqCst)
}

const KDSETMODE: libc::c_ulong = 0x4B3A;
const KD_TEXT: libc::c_ulong = 0x00;
const KD_GRAPHICS: libc::c_ulong = 0x01;

extern "C" fn signal_handler(_sig: libc::c_int) {
    GLOBAL_RUNNING.store(false, Ordering::SeqCst);
}

pub struct TtyManager {
    tty_fd: Option<RawFd>,
    orig_termios: Option<libc::termios>,
    graphics_mode_set: bool,
}

impl TtyManager {
    pub fn new() -> Self {
        GLOBAL_RUNNING.store(true, Ordering::SeqCst);

        // Register SIGINT / SIGTERM handlers so terminal state is cleanly restored
        // Ignore SIGTTIN and SIGTTOU so background service never gets suspended by tty
        unsafe {
            libc::signal(libc::SIGINT, signal_handler as *const () as usize);
            libc::signal(libc::SIGTERM, signal_handler as *const () as usize);
            libc::signal(libc::SIGTTIN, libc::SIG_IGN);
            libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        }

        let mut manager = Self {
            tty_fd: None,
            orig_termios: None,
            graphics_mode_set: false,
        };

        // Attempt to find a suitable TTY file descriptor (controlling tty, active console, or stdin)
        let candidates = ["/dev/tty1", "/dev/tty0", "/dev/tty", "/dev/console"];
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

            // 1. Hide terminal cursor directly on the hardware TTY
            let hide_cursor_cmd = b"\x1b[?25l";
            unsafe {
                libc::write(
                    fd,
                    hide_cursor_cmd.as_ptr() as *const libc::c_void,
                    hide_cursor_cmd.len(),
                );
            }

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

    /// Polls one non-blocking keyboard byte.
    ///
    /// Only q/Q changes the process state. Other keys are returned to the
    /// controller so feature-specific shortcuts can be handled there.
    pub fn poll_key(&mut self) -> Option<u8> {
        if !is_app_running() {
            return None;
        }

        if let Some(fd) = self.tty_fd {
            let mut buf = [0u8; 16];
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n > 0 {
                let mut key = None;
                for &b in &buf[..n as usize] {
                    if b == b'q' || b == b'Q' {
                        GLOBAL_RUNNING.store(false, Ordering::SeqCst);
                    } else if key.is_none() && b != 0x1B {
                        // Ignore escape bytes from terminal sequences, but keep
                        // ordinary application shortcut keys.
                        key = Some(b);
                    }
                }
                return key;
            }
        }

        None
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
        }
    }
}
