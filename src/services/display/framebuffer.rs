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

#[cfg(unix)]
const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
#[cfg(unix)]
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
#[cfg(unix)]
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

pub struct FramebufferBackend {
    #[cfg(unix)]
    file: File,
    #[cfg(unix)]
    mmap: MmapMut,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub line_length: usize,
    is_bgr: bool,
    offset_x: usize,
    offset_y: usize,
    tty: TtyManager,
    has_double_buffer: bool,
    current_page: usize,
    var_info: FbVarScreeninfo,
}

#[cfg(unix)]
impl FramebufferBackend {
    pub fn open(path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| {
                format!(
                    "Cannot open framebuffer '{}': {}. (Ensure permissions with 'sudo usermod -a -G video,render $USER' or run with sudo)",
                    path, e
                )
            })?;

        let fd = file.as_raw_fd();
        let mut var_info: FbVarScreeninfo = Default::default();
        let mut fix_info: FbFixScreeninfo = Default::default();

        unsafe {
            if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) < 0 {
                return Err("Failed to execute ioctl FBIOGET_VSCREENINFO on framebuffer".to_string());
            }
            if libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix_info) < 0 {
                return Err("Failed to execute ioctl FBIOGET_FSCREENINFO on framebuffer".to_string());
            }
        }

        let width = var_info.xres as usize;
        let height = var_info.yres as usize;
        let bpp = var_info.bits_per_pixel as usize;
        let mut line_length = fix_info.line_length as usize;

        if width == 0 || height == 0 {
            return Err(format!(
                "Invalid framebuffer resolution detected: {}x{}",
                width, height
            ));
        }

        if line_length == 0 {
            line_length = width * (bpp.max(8) / 8);
        }

        // Determine actual memory buffer length to map
        let smem_len = if fix_info.smem_len > 0 {
            fix_info.smem_len as usize
        } else {
            let y_virtual = if var_info.yres_virtual > 0 {
                var_info.yres_virtual as usize
            } else {
                height
            };
            line_length * y_virtual
        };

        // Explicitly set mapping length to smem_len (Linux char devices return length 0 with file.metadata())
        let mut mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(smem_len)
                .map_mut(&file)
                .map_err(|e| {
                    format!(
                        "Failed to mmap framebuffer device (length {} bytes): {}",
                        smem_len, e
                    )
                })?
        };

        // Check if double-buffering is supported in video memory
        let has_double_buffer = var_info.yres_virtual >= (var_info.yres * 2)
            && smem_len >= (2 * height * line_length);

        // Check if color layout has red at byte 0 (BGR) vs red at byte 2 (standard RGB/XRGB)
        let is_bgr = var_info.red.offset == 0 && var_info.blue.offset >= 16;

        // Calculate letterboxing / centering offsets for resolutions larger than 1280x480 (e.g. 1920x1080)
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

        // Clear entire framebuffer memory once to pure black on startup
        mmap.fill(0);

        // Initialize TTY manager (hides cursor, KD_GRAPHICS mode, raw keyboard)
        let tty = TtyManager::new();

        eprintln!(
            "[Framebuffer] Opened {}: {}x{} @ {} bpp (stride: {} bytes, smem_len: {} bytes, double_buf: {}, BGR: {}). Centering: ({}, {})",
            path, width, height, bpp, line_length, smem_len, has_double_buffer, is_bgr, offset_x, offset_y
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
            tty,
            has_double_buffer,
            current_page: 0,
            var_info,
        })
    }
}

#[cfg(unix)]
impl DisplayBackend for FramebufferBackend {
    fn is_open(&self) -> bool {
        !self.tty.poll_exit()
    }

    fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        let copy_width = SCREEN_WIDTH.min(self.width.saturating_sub(self.offset_x));
        let copy_height = SCREEN_HEIGHT.min(self.height.saturating_sub(self.offset_y));

        let page_y_offset = if self.has_double_buffer && self.current_page == 1 {
            self.height
        } else {
            0
        };

        if self.bpp == 32 {
            if !self.is_bgr {
                // Standard Little-Endian 32-bit XRGB (bytes: [B, G, R, 0]) matches buffer representation
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let src_slice = &buffer[src_start..src_start + copy_width];

                    let dst_y = page_y_offset + self.offset_y + y;
                    let dst_offset = dst_y * self.line_length + self.offset_x * 4;
                    let byte_count = copy_width * 4;

                    if dst_offset + byte_count <= self.mmap.len() {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                src_slice.as_ptr() as *const u8,
                                self.mmap.as_mut_ptr().add(dst_offset),
                                byte_count,
                            );
                        }
                    }
                }
            } else {
                // BGRX byte swap (Red at byte 0, Green at byte 1, Blue at byte 2)
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let dst_y = page_y_offset + self.offset_y + y;
                    let dst_offset = dst_y * self.line_length + self.offset_x * 4;

                    if dst_offset + copy_width * 4 <= self.mmap.len() {
                        let dst_ptr = unsafe { self.mmap.as_mut_ptr().add(dst_offset) as *mut u32 };
                        for x in 0..copy_width {
                            let pixel = buffer[src_start + x];
                            // Swap R (bits 16-23) and B (bits 0-7)
                            let swapped = (pixel & 0x0000FF00)
                                | ((pixel & 0x00FF0000) >> 16)
                                | ((pixel & 0x000000FF) << 16)
                                | 0xFF000000;
                            unsafe {
                                *dst_ptr.add(x) = swapped;
                            }
                        }
                    }
                }
            }
        } else if self.bpp == 24 {
            // 24-bit RGB / BGR
            for y in 0..copy_height {
                let src_start = y * SCREEN_WIDTH;
                let dst_y = page_y_offset + self.offset_y + y;
                let dst_row_offset = dst_y * self.line_length + self.offset_x * 3;

                for x in 0..copy_width {
                    let pixel = buffer[src_start + x];
                    let r = ((pixel >> 16) & 0xFF) as u8;
                    let g = ((pixel >> 8) & 0xFF) as u8;
                    let b = (pixel & 0xFF) as u8;

                    let px_idx = dst_row_offset + x * 3;
                    if px_idx + 3 <= self.mmap.len() {
                        if self.is_bgr {
                            self.mmap[px_idx] = r;
                            self.mmap[px_idx + 1] = g;
                            self.mmap[px_idx + 2] = b;
                        } else {
                            self.mmap[px_idx] = b;
                            self.mmap[px_idx + 1] = g;
                            self.mmap[px_idx + 2] = r;
                        }
                    }
                }
            }
        } else if self.bpp == 16 {
            // 16-bit RGB565 / BGR565
            for y in 0..copy_height {
                let src_start = y * SCREEN_WIDTH;
                let dst_y = page_y_offset + self.offset_y + y;
                let dst_row_offset = dst_y * self.line_length + self.offset_x * 2;

                for x in 0..copy_width {
                    let pixel = buffer[src_start + x];
                    let r = ((pixel >> 16) & 0xFF) as u16;
                    let g = ((pixel >> 8) & 0xFF) as u16;
                    let b = (pixel & 0xFF) as u16;

                    let color16 = if self.is_bgr {
                        ((b >> 3) << 11) | ((g >> 2) << 5) | (r >> 3)
                    } else {
                        ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
                    };

                    let px_idx = dst_row_offset + x * 2;
                    if px_idx + 2 <= self.mmap.len() {
                        self.mmap[px_idx] = (color16 & 0xFF) as u8;
                        self.mmap[px_idx + 1] = ((color16 >> 8) & 0xFF) as u8;
                    }
                }
            }
        }

        // Execute pan display if double buffering is enabled
        if self.has_double_buffer {
            self.var_info.yoffset = page_y_offset as u32;
            let fd = self.file.as_raw_fd();
            let ret = unsafe { libc::ioctl(fd, FBIOPAN_DISPLAY, &self.var_info) };
            if ret < 0 {
                // If FBIOPAN_DISPLAY is not supported by driver, seamlessly fallback to single buffer
                self.has_double_buffer = false;
                self.current_page = 0;
            } else {
                self.current_page = 1 - self.current_page;
            }
        }

        Ok(())
    }

    fn backend_name(&self) -> &str {
        "Linux Framebuffer (/dev/fb0) [Debian Raspberry Pi TTY]"
    }
}

#[cfg(unix)]
impl Drop for FramebufferBackend {
    fn drop(&mut self) {
        if self.has_double_buffer {
            self.var_info.yoffset = 0;
            let fd = self.file.as_raw_fd();
            unsafe {
                libc::ioctl(fd, FBIOPAN_DISPLAY, &self.var_info);
            }
        }
    }
}

#[cfg(not(unix))]
impl FramebufferBackend {
    pub fn open(_path: &str) -> Result<Self, String> {
        Err("Framebuffer backend is only supported on Linux/Unix systems".to_string())
    }
}

#[cfg(not(unix))]
impl DisplayBackend for FramebufferBackend {
    fn is_open(&self) -> bool {
        false
    }

    fn present(&mut self, _buffer: &[u32]) -> Result<(), String> {
        Ok(())
    }

    fn backend_name(&self) -> &str {
        "Linux Framebuffer (Unsupported on this OS)"
    }
}
