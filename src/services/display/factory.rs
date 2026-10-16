// Factory to instantiate display backends based on type or auto-detection

use super::drm_kms::DrmKmsBackend;
use super::framebuffer::FramebufferBackend;
use super::simulator::SimulatorBackend;
use super::traits::DisplayBackend;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendType {
    Auto,
    Simulator,
    Framebuffer(String),
    DrmKms(String),
}

pub fn create_backend(requested: BackendType) -> Result<Box<dyn DisplayBackend>, String> {
    match requested {
        BackendType::Simulator => {
            let sim = SimulatorBackend::new()?;
            Ok(Box::new(sim))
        }
        BackendType::Framebuffer(path) => {
            let fb = FramebufferBackend::open(&path)?;
            Ok(Box::new(fb))
        }
        BackendType::DrmKms(path) => {
            let drm = DrmKmsBackend::open(&path)?;
            Ok(Box::new(drm))
        }
        BackendType::Auto => {
            let has_gui = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();
            if has_gui {
                println!("[Backend] Display server detected. Initializing Desktop Simulator...");
                let sim = SimulatorBackend::new()?;
                return Ok(Box::new(sim));
            }

            println!("[Backend] Pure console TTY detected on Linux. Probing DRM/KMS direct output...");
            for card in &["/dev/dri/card0", "/dev/dri/card1"] {
                if std::path::Path::new(card).exists() {
                    match DrmKmsBackend::open(card) {
                        Ok(drm) => return Ok(Box::new(drm)),
                        Err(e) => eprintln!("[Backend] DRM on {} failed: {}", card, e),
                    }
                }
            }

            println!("[Backend] Probing Linux Framebuffer (/dev/fb0, /dev/fb1)...");
            for fb_path in &["/dev/fb0", "/dev/fb1"] {
                if std::path::Path::new(fb_path).exists() {
                    match FramebufferBackend::open(fb_path) {
                        Ok(fb) => return Ok(Box::new(fb)),
                        Err(e) => eprintln!("[Backend] Framebuffer on {} failed: {}", fb_path, e),
                    }
                }
            }

            Err("Could not initialize any display backend. Ensure permissions for /dev/fb0 or /dev/dri/card0 (e.g. add user to video and render groups, or run with sudo).".to_string())
        }
    }
}
