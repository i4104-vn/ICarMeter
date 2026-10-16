// Linux Framebuffer backend (/dev/fb0) for direct console rendering on Arch Linux without WM/DE

use super::traits::DisplayBackend;
use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::services::input::TtyManager;
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbFixScreeninfo {
    id: [u8; 16],
    smem_start: usize,
    smem_len: u32,
    fb_type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: usize,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

pub struct FramebufferBackend {
    file: File,
    mmap: MmapMut,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub line_length: usize,
    is_bgr: bool,
    offset_x: usize,
    offset_y: usize,
    _tty: TtyManager,
    has_double_buffer: bool,
    current_page: usize,
    var_info: FbVarScreeninfo,
}

impl FramebufferBackend {
    pub fn open(path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| {
                format!(
                    "Cannot open framebuffer '{}': {}. (Hint: check permissions with 'sudo usermod -a -G video $USER' or run with sudo)",
                    path, e
                )
            })?;

        let fd = file.as_raw_fd();
        let mut var_info: FbVarScreeninfo = Default::default();
        let mut fix_info: FbFixScreeninfo = Default::default();

        unsafe {
            if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) < 0 {
                return Err("Failed to ioctl FBIOGET_VSCREENINFO".to_string());
            }
            if libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix_info) < 0 {
                return Err("Failed to ioctl FBIOGET_FSCREENINFO".to_string());
            }
        }

        let width = var_info.xres as usize;
        let height = var_info.yres as usize;
        let bpp = var_info.bits_per_pixel as usize;
        let line_length = fix_info.line_length as usize;

        // Check if double-buffering is supported in video memory
        let has_double_buffer = var_info.yres_virtual >= (var_info.yres * 2);

        // Check BGR vs RGB pixel layout
        let is_bgr = var_info.blue.offset > var_info.red.offset;

        // Calculate letterboxing / centering offsets if screen is larger than 1280x480
        let offset_x = if width > SCREEN_WIDTH {
            (width - SCREEN_WIDTH) / 2
        } else {
            0
        };
        let offset_y = if height > SCREEN_HEIGHT {
            (height - SCREEN_HEIGHT) / 2
        } else {
            0
        };

        let mut mmap = unsafe {
            MmapMut::map_mut(&file).map_err(|e| format!("Failed to mmap framebuffer: {}", e))?
        };

        // Clear entire framebuffer memory once to pure dark on launch
        let fill_len = mmap.len();
        mmap[..fill_len].fill(0);

        // Initialize TTY console manager (hides cursor, KD_GRAPHICS mode, raw keyboard)
        let tty = TtyManager::new();

        eprintln!(
            "[FB] Opened {}: {}x{} @ {} bpp (stride: {}B, double_buf: {}, BGR: {}). Centering offset: ({}, {})",
            path, width, height, bpp, line_length, has_double_buffer, is_bgr, offset_x, offset_y
        );

        Ok(Self {
            file,
            mmap,
            width,
            height,
            bpp,
            line_length,
            is_bgr,
            offset_x,
            offset_y,
            _tty: tty,
            has_double_buffer,
            current_page: 0,
            var_info,
        })
    }
}

impl DisplayBackend for FramebufferBackend {
    fn is_open(&self) -> bool {
        true
    }

    fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        let copy_width = (self.width - self.offset_x).min(SCREEN_WIDTH);
        let copy_height = (self.height - self.offset_y).min(SCREEN_HEIGHT);

        // Page flipping offset
        let page_y_offset = if self.has_double_buffer && self.current_page == 1 {
            self.height
        } else {
            0
        };

        if self.bpp == 32 {
            if !self.is_bgr {
                // Standard 32-bit XRGB copy
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let src_slice = &buffer[src_start..src_start + copy_width];

                    let dst_y = page_y_offset + self.offset_y + y;
                    let dst_offset = dst_y * self.line_length + self.offset_x * 4;
                    let dst_slice = &mut self.mmap[dst_offset..dst_offset + copy_width * 4];

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            src_slice.as_ptr() as *const u8,
                            dst_slice.as_mut_ptr(),
                            copy_width * 4,
                        );
                    }
                }
            } else {
                // BGRX byte swap copy
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let dst_y = page_y_offset + self.offset_y + y;
                    let dst_offset = dst_y * self.line_length + self.offset_x * 4;

                    for x in 0..copy_width {
                        let pixel = buffer[src_start + x];
                        let r = ((pixel >> 16) & 0xFF) as u8;
                        let g = ((pixel >> 8) & 0xFF) as u8;
                        let b = (pixel & 0xFF) as u8;

                        let px_idx = dst_offset + x * 4;
                        self.mmap[px_idx] = b;
                        self.mmap[px_idx + 1] = g;
                        self.mmap[px_idx + 2] = r;
                        self.mmap[px_idx + 3] = 0xFF;
                    }
                }
            }
        } else if self.bpp == 16 {
            // 16-bit RGB565 fallback
            for y in 0..copy_height {
                let src_start = y * SCREEN_WIDTH;
                let dst_y = page_y_offset + self.offset_y + y;
                let dst_offset = dst_y * self.line_length + self.offset_x * 2;

                for x in 0..copy_width {
                    let pixel = buffer[src_start + x];
                    let r = ((pixel >> 16) & 0xFF) as u16;
                    let g = ((pixel >> 8) & 0xFF) as u16;
                    let b = (pixel & 0xFF) as u16;

                    let rgb565 = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3);
                    let px_idx = dst_offset + x * 2;
                    self.mmap[px_idx] = (rgb565 & 0xFF) as u8;
                    self.mmap[px_idx + 1] = ((rgb565 >> 8) & 0xFF) as u8;
                }
            }
        }

        // Execute pan display if double buffering is available
        if self.has_double_buffer {
            self.var_info.yoffset = page_y_offset as u32;
            let fd = self.file.as_raw_fd();
            unsafe {
                libc::ioctl(fd, FBIOPAN_DISPLAY, &self.var_info);
            }
            self.current_page = 1 - self.current_page;
        }

        Ok(())
    }

    fn backend_name(&self) -> &str {
        "Linux Framebuffer (/dev/fb0) [TTY Console]"
    }
}

impl Drop for FramebufferBackend {
    fn drop(&mut self) {
        // Reset pan display to page 0
        if self.has_double_buffer {
            self.var_info.yoffset = 0;
            let fd = self.file.as_raw_fd();
            unsafe {
                libc::ioctl(fd, FBIOPAN_DISPLAY, &self.var_info);
            }
        }
    }
}
