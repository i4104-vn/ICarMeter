// Hardware GPS NMEA 0183 serial receiver module

pub mod nmea;

use nmea::parse_nmea_speed;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::time::{Duration, Instant};

const GPS_FIX_TIMEOUT: Duration = Duration::from_secs(3);

pub struct GpsReceiver {
    device_path: String,
    baud_rate: u32,
    file: Option<File>,
    buffer: Vec<u8>,
    logged_error: bool,
    last_valid_fix: Option<Instant>,
}

impl GpsReceiver {
    pub fn new(device_path: &str, baud_rate: u32) -> Self {
        let mut receiver = Self {
            device_path: device_path.to_string(),
            baud_rate,
            file: None,
            buffer: Vec::with_capacity(512),
            logged_error: false,
            last_valid_fix: None,
        };
        receiver.try_open();
        receiver
    }

    pub fn is_connected(&self) -> bool {
        self.file.is_some()
            && self
                .last_valid_fix
                .is_some_and(|timestamp| timestamp.elapsed() < GPS_FIX_TIMEOUT)
    }

    pub fn matches_config(&self, device: &str, baud: u32) -> bool {
        self.device_path == device && self.baud_rate == baud
    }

    fn try_open(&mut self) {
        if self.file.is_some() {
            return;
        }

        match OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.device_path)
        {
            Ok(file) => {
                let fd = file.as_raw_fd();
                unsafe {
                    let mut termios: libc::termios = std::mem::zeroed();
                    if libc::tcgetattr(fd, &mut termios) == 0 {
                        libc::cfmakeraw(&mut termios);
                        let speed = match self.baud_rate {
                            4800 => libc::B4800,
                            9600 => libc::B9600,
                            19200 => libc::B19200,
                            38400 => libc::B38400,
                            57600 => libc::B57600,
                            115200 => libc::B115200,
                            _ => libc::B9600,
                        };
                        libc::cfsetispeed(&mut termios, speed);
                        libc::cfsetospeed(&mut termios, speed);
                        termios.c_cflag |= libc::CREAD | libc::CLOCAL;
                        termios.c_cc[libc::VMIN] = 0;
                        termios.c_cc[libc::VTIME] = 0; // non-blocking
                        libc::tcsetattr(fd, libc::TCSANOW, &termios);
                    }
                }
                eprintln!(
                    "[GPS] Connected to {} at {} baud",
                    self.device_path, self.baud_rate
                );
                self.file = Some(file);
                self.logged_error = false;
            }
            Err(e) => {
                if !self.logged_error {
                    eprintln!("[GPS] Waiting for GPS device {}: {}", self.device_path, e);
                    self.logged_error = true;
                }
            }
        }
    }

    /// Reads available NMEA bytes and returns updated speed (km/h) if a valid sentence was received
    pub fn poll_speed(&mut self) -> Option<f32> {
        if self.file.is_none() {
            self.try_open();
            self.file.as_ref()?;
        }

        let fd = self.file.as_ref()?.as_raw_fd();
        let mut read_buf = [0u8; 256];
        let bytes_read = unsafe {
            libc::read(
                fd,
                read_buf.as_mut_ptr() as *mut libc::c_void,
                read_buf.len(),
            )
        };

        if bytes_read < 0 {
            // The GPS was unplugged or the serial device disappeared. Let the
            // next poll attempt reopen it instead of keeping a stale handle.
            self.file = None;
            self.last_valid_fix = None;
            self.buffer.clear();
            return None;
        }

        if bytes_read == 0 {
            return None;
        }

        self.buffer
            .extend_from_slice(&read_buf[..bytes_read as usize]);

        let mut latest_speed = None;

        // Process complete NMEA lines
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
            if let Ok(line) = std::str::from_utf8(&line_bytes) {
                let trimmed = line.trim();
                if let Some(spd) = parse_nmea_speed(trimmed) {
                    self.last_valid_fix = Some(Instant::now());
                    latest_speed = Some(spd);
                }
            }
        }

        latest_speed
    }
}
