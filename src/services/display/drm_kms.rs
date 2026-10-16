#![allow(dead_code, unused_imports)]
use super::traits::DisplayBackend;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::services::input::TtyManager;

#[cfg(unix)]
use memmap2::MmapMut;
#[cfg(unix)]
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(unix)]
const DRM_IOCTL_BASE: u8 = b'd';

#[cfg(unix)]
macro_rules! drm_iowr {
    ($nr:expr, $ty:ty) => {
        nix_iowr(DRM_IOCTL_BASE, $nr, std::mem::size_of::<$ty>())
    };
}

#[cfg(unix)]
const fn nix_iowr(base: u8, nr: u32, size: usize) -> libc::c_ulong {
    let dir = 3u32; // _IOC_READ | _IOC_WRITE
    (((dir) << 30) | ((size as u32) << 16) | ((base as u32) << 8) | nr) as libc::c_ulong
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeCardRes {
    fb_id_ptr: u64,
    crtc_id_ptr: u64,
    connector_id_ptr: u64,
    encoder_id_ptr: u64,
    count_fbs: u32,
    count_crtcs: u32,
    count_connectors: u32,
    count_encoders: u32,
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct DrmModeModeInfo {
    pub clock: u32,
    pub hdisplay: u16,
    pub hsync_start: u16,
    pub hsync_end: u16,
    pub htotal: u16,
    pub hskew: u16,
    pub vdisplay: u16,
    pub vsync_start: u16,
    pub vsync_end: u16,
    pub vtotal: u16,
    pub vscan: u16,
    pub vrefresh: u32,
    pub flags: u32,
    pub type_: u32,
    pub name: [u8; 32],
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeGetConnector {
    encoders_ptr: u64,
    modes_ptr: u64,
    props_ptr: u64,
    prop_values_ptr: u64,
    count_modes: u32,
    count_props: u32,
    count_encoders: u32,
    width_mm: u32,
    height_mm: u32,
    subpixel: u32,
    connector_id: u32,
    connector_type: u32,
    connector_type_id: u32,
    connection: u32,
    mm_width: u32,
    mm_height: u32,
    encoder_id: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeGetEncoder {
    encoder_id: u32,
    encoder_type: u32,
    crtc_id: u32,
    possible_crtcs: u32,
    possible_clones: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeCrtc {
    set_connectors_ptr: u64,
    count_connectors: u32,
    crtc_id: u32,
    fb_id: u32,
    x: u32,
    y: u32,
    gamma_size: u32,
    mode_valid: u32,
    mode: DrmModeModeInfo,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeCreateDumb {
    height: u32,
    width: u32,
    bpp: u32,
    flags: u32,
    handle: u32,
    pitch: u32,
    size: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeMapDumb {
    handle: u32,
    pad: u32,
    offset: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DrmModeFbCmd {
    fb_id: u32,
    width: u32,
    height: u32,
    pitch: u32,
    bpp: u32,
    depth: u32,
    handle: u32,
}

#[cfg(unix)]
struct DumbBuffer {
    #[allow(dead_code)]
    handle: u32,
    fb_id: u32,
    pitch: u32,
    mmap: MmapMut,
}

pub struct DrmKmsBackend {
    #[cfg(unix)]
    file: File,
    #[cfg(unix)]
    crtc_id: u32,
    #[cfg(unix)]
    connector_id: u32,
    #[cfg(unix)]
    saved_crtc: DrmModeCrtc,
    #[cfg(unix)]
    mode: DrmModeModeInfo,
    #[cfg(unix)]
    buffers: [DumbBuffer; 2],
    #[cfg(unix)]
    front_buf: usize,
    #[cfg(unix)]
    offset_x: usize,
    #[cfg(unix)]
    offset_y: usize,
    tty: TtyManager,
}

#[cfg(unix)]
impl DrmKmsBackend {
    pub fn open(device_path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .map_err(|e| format!("Cannot open DRM device '{}': {}", device_path, e))?;

        let fd = file.as_raw_fd();

        // 1. Query resources
        let mut res: DrmModeCardRes = Default::default();
        let ioctl_get_resources = drm_iowr!(0xA0, DrmModeCardRes);
        unsafe {
            if libc::ioctl(fd, ioctl_get_resources, &mut res) < 0 {
                return Err("Failed to query DRM resources".to_string());
            }
        }

        let mut connector_ids = vec![0u32; res.count_connectors as usize];
        let mut crtc_ids = vec![0u32; res.count_crtcs as usize];
        res.connector_id_ptr = connector_ids.as_mut_ptr() as u64;
        res.crtc_id_ptr = crtc_ids.as_mut_ptr() as u64;

        unsafe {
            if libc::ioctl(fd, ioctl_get_resources, &mut res) < 0 {
                return Err("Failed to query DRM connector/CRTC list".to_string());
            }
        }

        // 2. Find connected connector
        let ioctl_get_connector = drm_iowr!(0xA7, DrmModeGetConnector);
        let mut selected_conn: Option<(u32, DrmModeModeInfo, u32)> = None;

        for &conn_id in &connector_ids {
            let mut conn: DrmModeGetConnector = Default::default();
            conn.connector_id = conn_id;

            unsafe {
                if libc::ioctl(fd, ioctl_get_connector, &mut conn) < 0 {
                    continue;
                }
            }

            if conn.connection != 1 || conn.count_modes == 0 {
                continue;
            }

            let mut modes = vec![DrmModeModeInfo::default(); conn.count_modes as usize];
            conn.modes_ptr = modes.as_mut_ptr() as u64;

            unsafe {
                if libc::ioctl(fd, ioctl_get_connector, &mut conn) < 0 {
                    continue;
                }
            }

            // Look for exact 1280x480 resolution or fallback to native connected display mode (e.g. 1920x1080)
            let chosen_mode = modes
                .iter()
                .find(|m| {
                    m.hdisplay as usize == SCREEN_WIDTH && m.vdisplay as usize == SCREEN_HEIGHT
                })
                .cloned()
                .unwrap_or_else(|| modes[0]);

            selected_conn = Some((conn_id, chosen_mode, conn.encoder_id));
            break;
        }

        let (connector_id, mode, encoder_id) =
            selected_conn.ok_or_else(|| "No connected DRM display connector found".to_string())?;

        // 3. Find CRTC
        let ioctl_get_encoder = drm_iowr!(0xA6, DrmModeGetEncoder);
        let mut enc: DrmModeGetEncoder = Default::default();
        enc.encoder_id = encoder_id;
        let mut crtc_id = 0;
        if encoder_id != 0 {
            unsafe {
                if libc::ioctl(fd, ioctl_get_encoder, &mut enc) == 0 && enc.crtc_id != 0 {
                    crtc_id = enc.crtc_id;
                }
            }
        }
        if crtc_id == 0 && !crtc_ids.is_empty() {
            crtc_id = crtc_ids[0];
        }

        // 4. Save existing CRTC state to restore on exit
        let ioctl_get_crtc = drm_iowr!(0xA1, DrmModeCrtc);
        let mut saved_crtc: DrmModeCrtc = Default::default();
        saved_crtc.crtc_id = crtc_id;
        unsafe {
            libc::ioctl(fd, ioctl_get_crtc, &mut saved_crtc);
        }

        // 5. Allocate 2 dumb buffers for double buffering
        let mut buf0 = Self::create_dumb_buffer(&file, mode.hdisplay as u32, mode.vdisplay as u32)?;
        let mut buf1 = Self::create_dumb_buffer(&file, mode.hdisplay as u32, mode.vdisplay as u32)?;

        // Clear both buffers to black
        buf0.mmap.fill(0);
        buf1.mmap.fill(0);

        let disp_w = mode.hdisplay as usize;
        let disp_h = mode.vdisplay as usize;

        // Centering offset for 1280x480 inside display resolution
        let offset_x = if disp_w > SCREEN_WIDTH {
            (disp_w - SCREEN_WIDTH) / 2
        } else {
            0
        };
        let offset_y = if disp_h > SCREEN_HEIGHT {
            (disp_h - SCREEN_HEIGHT) / 2
        } else {
            0
        };

        eprintln!(
            "[DRM/KMS] Bound to connector {} (mode: {}x{}@{}Hz), CRTC: {}. Centering: ({}, {})",
            connector_id, mode.hdisplay, mode.vdisplay, mode.vrefresh, crtc_id, offset_x, offset_y
        );

        // 6. Set CRTC mode
        let ioctl_set_crtc = drm_iowr!(0xA2, DrmModeCrtc);
        let mut set_crtc: DrmModeCrtc = Default::default();
        set_crtc.crtc_id = crtc_id;
        set_crtc.fb_id = buf0.fb_id;
        set_crtc.set_connectors_ptr = &connector_id as *const u32 as u64;
        set_crtc.count_connectors = 1;
        set_crtc.mode = mode;
        set_crtc.mode_valid = 1;

        unsafe {
            if libc::ioctl(fd, ioctl_set_crtc, &mut set_crtc) < 0 {
                return Err("Failed to execute DRM_IOCTL_MODE_SETCRTC".to_string());
            }
        }

        let tty = TtyManager::new();

        Ok(Self {
            file,
            crtc_id,
            connector_id,
            saved_crtc,
            mode,
            buffers: [buf0, buf1],
            front_buf: 0,
            offset_x,
            offset_y,
            tty,
        })
    }

    fn create_dumb_buffer(file: &File, width: u32, height: u32) -> Result<DumbBuffer, String> {
        let fd = file.as_raw_fd();

        let mut create_req = DrmModeCreateDumb {
            width,
            height,
            bpp: 32,
            ..Default::default()
        };
        let ioctl_create_dumb = drm_iowr!(0xB2, DrmModeCreateDumb);
        unsafe {
            if libc::ioctl(fd, ioctl_create_dumb, &mut create_req) < 0 {
                return Err("Failed DRM_IOCTL_MODE_CREATE_DUMB".to_string());
            }
        }

        let mut fb_cmd = DrmModeFbCmd {
            width,
            height,
            pitch: create_req.pitch,
            bpp: 32,
            depth: 24,
            handle: create_req.handle,
            fb_id: 0,
        };
        let ioctl_add_fb = drm_iowr!(0xAE, DrmModeFbCmd);
        unsafe {
            if libc::ioctl(fd, ioctl_add_fb, &mut fb_cmd) < 0 {
                return Err("Failed DRM_IOCTL_MODE_ADDFB".to_string());
            }
        }

        let mut map_req = DrmModeMapDumb {
            handle: create_req.handle,
            pad: 0,
            offset: 0,
        };
        let ioctl_map_dumb = drm_iowr!(0xB3, DrmModeMapDumb);
        unsafe {
            if libc::ioctl(fd, ioctl_map_dumb, &mut map_req) < 0 {
                return Err("Failed DRM_IOCTL_MODE_MAP_DUMB".to_string());
            }
        }

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .offset(map_req.offset)
                .len(create_req.size as usize)
                .map_mut(file)
                .map_err(|e| format!("Failed to mmap dumb buffer: {}", e))?
        };

        Ok(DumbBuffer {
            handle: create_req.handle,
            fb_id: fb_cmd.fb_id,
            pitch: create_req.pitch,
            mmap,
        })
    }
}

#[cfg(unix)]
impl DisplayBackend for DrmKmsBackend {
    fn is_open(&self) -> bool {
        !self.tty.poll_exit()
    }

    fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        let back_idx = 1 - self.front_buf;
        let buf = &mut self.buffers[back_idx];

        let disp_w = self.mode.hdisplay as usize;
        let disp_h = self.mode.vdisplay as usize;

        let copy_w = SCREEN_WIDTH.min(disp_w.saturating_sub(self.offset_x));
        let copy_h = SCREEN_HEIGHT.min(disp_h.saturating_sub(self.offset_y));

        for y in 0..copy_h {
            let src_start = y * SCREEN_WIDTH;
            let src_slice = &buffer[src_start..src_start + copy_w];
            let dst_offset = (self.offset_y + y) * buf.pitch as usize + self.offset_x * 4;
            let byte_count = copy_w * 4;

            if dst_offset + byte_count <= buf.mmap.len() {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        src_slice.as_ptr() as *const u8,
                        buf.mmap.as_mut_ptr().add(dst_offset),
                        byte_count,
                    );
                }
            }
        }

        // Set CRTC to back buffer
        let fd = self.file.as_raw_fd();
        let ioctl_set_crtc = drm_iowr!(0xA2, DrmModeCrtc);
        let mut set_crtc: DrmModeCrtc = Default::default();
        set_crtc.crtc_id = self.crtc_id;
        set_crtc.fb_id = buf.fb_id;
        set_crtc.set_connectors_ptr = &self.connector_id as *const u32 as u64;
        set_crtc.count_connectors = 1;
        set_crtc.mode = self.mode;
        set_crtc.mode_valid = 1;

        unsafe {
            libc::ioctl(fd, ioctl_set_crtc, &mut set_crtc);
        }

        self.front_buf = back_idx;
        Ok(())
    }

    fn backend_name(&self) -> &str {
        "Linux DRM/KMS (Direct Hardware Output) [Debian Raspberry Pi]"
    }
}

#[cfg(unix)]
impl Drop for DrmKmsBackend {
    fn drop(&mut self) {
        let fd = self.file.as_raw_fd();
        let ioctl_set_crtc = drm_iowr!(0xA2, DrmModeCrtc);
        unsafe {
            libc::ioctl(fd, ioctl_set_crtc, &mut self.saved_crtc);
        }
    }
}

#[cfg(not(unix))]
impl DrmKmsBackend {
    pub fn open(_path: &str) -> Result<Self, String> {
        Err("DRM/KMS backend is only supported on Linux/Unix systems".to_string())
    }
}

#[cfg(not(unix))]
impl DisplayBackend for DrmKmsBackend {
    fn is_open(&self) -> bool {
        false
    }

    fn present(&mut self, _buffer: &[u32]) -> Result<(), String> {
        Ok(())
    }

    fn backend_name(&self) -> &str {
        "Linux DRM/KMS (Unsupported on this OS)"
    }
}
