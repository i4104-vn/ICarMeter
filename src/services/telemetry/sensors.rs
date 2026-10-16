// Hardware Bluetooth TPMS & telemetry sensor reader for Raspberry Pi OS (/dev/rfcomm0, etc.)

use crate::models::VehicleState;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

pub struct SensorReceiver {
    device_path: String,
    baud_rate: u32,
    file: Option<File>,
    buffer: Vec<u8>,
    logged_error: bool,
}

impl SensorReceiver {
    pub fn new(device_path: &str, baud_rate: u32) -> Self {
        let mut receiver = Self {
            device_path: device_path.to_string(),
            baud_rate,
            file: None,
            buffer: Vec::with_capacity(256),
            logged_error: false,
        };
        receiver.try_open();
        receiver
    }

    fn try_open(&mut self) {
        if self.file.is_some() {
            return;
        }

        match OpenOptions::new().read(true).write(true).open(&self.device_path) {
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
                println!("[Sensors] Connected to Bluetooth sensor {} at {} baud", self.device_path, self.baud_rate);
                self.file = Some(file);
                self.logged_error = false;
            }
            Err(e) => {
                if !self.logged_error {
                    eprintln!("[Sensors] Waiting for Bluetooth sensor device {}: {}", self.device_path, e);
                    self.logged_error = true;
                }
            }
        }
    }

    /// Reads available incoming sensor telemetry and updates vehicle state
    pub fn poll(&mut self, state: &mut VehicleState) {
        if self.file.is_none() {
            self.try_open();
            if self.file.is_none() {
                return;
            }
        }

        let fd = match self.file.as_ref() {
            Some(f) => f.as_raw_fd(),
            None => return,
        };

        let mut read_buf = [0u8; 128];
        let bytes_read = unsafe {
            libc::read(fd, read_buf.as_mut_ptr() as *mut libc::c_void, read_buf.len())
        };

        if bytes_read <= 0 {
            return;
        }

        self.buffer.extend_from_slice(&read_buf[..bytes_read as usize]);

        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
            if let Ok(line) = std::str::from_utf8(&line_bytes) {
                parse_sensor_line(line.trim(), state);
            }
        }
    }
}

/// Parses telemetry string from Bluetooth TPMS / OBD / microcontroller receiver
fn parse_sensor_line(line: &str, state: &mut VehicleState) {
    if line.is_empty() {
        return;
    }

    // Emotion trigger command (e.g. "EMO:happy", "EMO:love", "EMO:angry")
    if let Some(emo_str) = line.strip_prefix("EMO:") {
        state.set_expression_name(emo_str.trim());
        return;
    }

    // Comma-separated TPMS packet: "TPMS,2.4,2.5,2.4,2.5"
    if let Some(tpms_data) = line.strip_prefix("TPMS,") {
        let parts: Vec<&str> = tpms_data.split(',').collect();
        if parts.len() >= 4 {
            if let (Ok(fl), Ok(fr), Ok(rl), Ok(rr)) = (
                parts[0].trim().parse(),
                parts[1].trim().parse(),
                parts[2].trim().parse(),
                parts[3].trim().parse(),
            ) {
                state.tires.fl = fl;
                state.tires.fr = fr;
                state.tires.rl = rl;
                state.tires.rr = rr;
                return;
            }
        }
    }

    // Key-value pairs (e.g. "FL:2.4 FR:2.5 RL:2.4 RR:2.5" or comma-separated)
    for part in line.split(|c| c == ' ' || c == ',' || c == ';') {
        let part = part.trim();
        if let Some((tire, val)) = part.split_once(':') {
            if let Ok(pressure) = val.trim().parse::<f32>() {
                match tire.trim().to_uppercase().as_str() {
                    "FL" => state.tires.fl = pressure,
                    "FR" => state.tires.fr = pressure,
                    "RL" => state.tires.rl = pressure,
                    "RR" => state.tires.rr = pressure,
                    _ => {}
                }
            }
        }
    }
}
