// Command-line interface argument parser and options

use crate::configs::display::{
    PHYSICAL_HEIGHT_MM, PHYSICAL_WIDTH_MM, SCREEN_HEIGHT, SCREEN_WIDTH, TARGET_FPS,
};
use crate::render::components::EmoExpression;
use crate::services::display::BackendType;
use std::env;

pub struct CliConfig {
    pub backend_type: BackendType,
    pub snapshot_path: Option<String>,
    pub target_fps: u64,
    pub initial_speed: Option<f32>,
    pub forced_emo: Option<EmoExpression>,
    pub should_exit: bool,
}

pub fn print_help() {
    println!(
        r#"
================================================================================
    Car Digital Dashboard - 7" Display ({:.0}mm x {:.0}mm, {}x{})
================================================================================
Usage: car_dashboard [OPTIONS]

Options:
  --backend <TYPE>    Display backend: auto | sim | fb | drm (default: auto)
  --device <PATH>     Device path for fb (/dev/fb0) or drm (/dev/dri/card0)
  --snapshot <PATH>   Render a single frame, save as PNG image and exit
  --fps <NUMBER>      Target frames per second (15-240, default: 60)
  --speed <KM/H>      Initial speed override (0 to 160)
  --emo <EXPR>        Set the companion's expression:
                      normal | happy | love | sparkle | surprised | starry |
                      blush | sleeping | dizzy (legacy moods are also accepted)
  --help, -h          Display this help message

Mode:
  Fully automatic telemetry & driving simulation (no user input required).
  When vehicle is stationary (speed == 0) for 3s, the animated expression appears.
================================================================================
"#,
        PHYSICAL_WIDTH_MM, PHYSICAL_HEIGHT_MM, SCREEN_WIDTH, SCREEN_HEIGHT
    );
}

pub fn parse_args() -> CliConfig {
    let args: Vec<String> = env::args().collect();

    let mut backend_type = BackendType::Auto;
    let mut custom_device: Option<String> = None;
    let mut snapshot_path: Option<String> = None;
    let mut target_fps = TARGET_FPS;
    let mut initial_speed: Option<f32> = None;
    let mut forced_emo: Option<EmoExpression> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--backend" => {
                if i + 1 < args.len() {
                    i += 1;
                    match args[i].to_lowercase().as_str() {
                        "sim" | "simulator" => backend_type = BackendType::Simulator,
                        "fb" | "framebuffer" => {
                            backend_type = BackendType::Framebuffer("/dev/fb0".to_string())
                        }
                        "drm" | "kms" => {
                            backend_type = BackendType::DrmKms("/dev/dri/card0".to_string())
                        }
                        "auto" => backend_type = BackendType::Auto,
                        other => {
                            eprintln!("Unknown backend '{}', falling back to auto.", other);
                        }
                    }
                }
            }
            "--device" => {
                if i + 1 < args.len() {
                    i += 1;
                    custom_device = Some(args[i].clone());
                }
            }
            "--snapshot" => {
                if i + 1 < args.len() {
                    i += 1;
                    snapshot_path = Some(args[i].clone());
                }
            }
            "--fps" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(fps) = args[i].parse::<u64>() {
                        target_fps = fps.clamp(15, 240);
                    }
                }
            }
            "--speed" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(spd) = args[i].parse::<f32>() {
                        initial_speed = Some(spd);
                    }
                }
            }
            "--emo" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Some(expr) = EmoExpression::from_str(&args[i]) {
                        forced_emo = Some(expr);
                    } else {
                        eprintln!("Unknown emo expression '{}'", args[i]);
                    }
                }
            }
            "--help" | "-h" => {
                print_help();
                return CliConfig {
                    backend_type,
                    snapshot_path: None,
                    target_fps,
                    initial_speed: None,
                    forced_emo: None,
                    should_exit: true,
                };
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(dev) = custom_device {
        match backend_type {
            BackendType::Framebuffer(_) => backend_type = BackendType::Framebuffer(dev),
            BackendType::DrmKms(_) => backend_type = BackendType::DrmKms(dev),
            _ => {}
        }
    }

    CliConfig {
        backend_type,
        snapshot_path,
        target_fps,
        initial_speed,
        forced_emo,
        should_exit: false,
    }
}
