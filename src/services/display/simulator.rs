// Desktop window simulator backend using Minifb

use super::traits::DisplayBackend;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS};
use minifb::{Key, Scale, Window, WindowOptions};

pub struct SimulatorBackend {
    window: Window,
}

impl SimulatorBackend {
    pub fn new() -> Result<Self, String> {
        let opts = WindowOptions {
            scale: Scale::X1,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        };

        let title = "Car Digital Dashboard (7\" 184mm x 70mm - 1280x480) [Automatic Mode]";
        let mut window = Window::new(title, SCREEN_WIDTH, SCREEN_HEIGHT, opts)
            .map_err(|e| format!("Failed to create simulator window: {}", e))?;

        window.set_target_fps(TARGET_FPS as usize);

        Ok(Self { window })
    }
}

impl DisplayBackend for SimulatorBackend {
    fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        self.window
            .update_with_buffer(buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| format!("Window update error: {}", e))
    }

    fn backend_name(&self) -> &str {
        "Simulator (Minifb Window 1280x480)"
    }
}
