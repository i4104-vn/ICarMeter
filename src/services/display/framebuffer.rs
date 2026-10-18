// Linux Framebuffer backend (/dev/fb0) optimized specifically for Raspberry Pi OS (Debian) TTY console

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

pub struct FramebufferBackend {
    _file: File,
    mmap: MmapMut,
    pub width: usize,
    pub height: usize,
    pub line_length: usize,
    pub bpp: usize,
    is_bgr: bool,
    offset_x: usize,
    offset_y: usize,
    tty: TtyManager,
}

impl FramebufferBackend {
    pub fn open(path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| {
                format!(
                    "Cannot open framebuffer '{}': {}. (Hint: run with sudo or check group permissions)",
                    path, e
                )
            })?;

        let fd = file.as_raw_fd();
        let mut var_info: FbVarScreeninfo = Default::default();
        let mut fix_info: FbFixScreeninfo = Default::default();

        unsafe {
            if libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) < 0 {
                return Err("Failed to query FBIOGET_VSCREENINFO on framebuffer".to_string());
            }
            if libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix_info) < 0 {
                return Err("Failed to query FBIOGET_FSCREENINFO on framebuffer".to_string());
            }
        }

        // Attempt to request 32-bit depth if current driver defaulted to 16 bpp
        if var_info.bits_per_pixel != 32 {
            let mut req = var_info;
            req.bits_per_pixel = 32;
            req.activate = 0; // FB_ACTIVATE_NOW
            if unsafe { libc::ioctl(fd, 0x4601, &mut req) } == 0 { // FBIOPUT_VSCREENINFO
                let _ = unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) };
                let _ = unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix_info) };
            }
        }

        let width = var_info.xres as usize;
        let height = var_info.yres as usize;
        let bpp = var_info.bits_per_pixel as usize;
        let bytes_per_pixel = (bpp / 8).max(2);
        let line_length = if fix_info.line_length > 0 {
            fix_info.line_length as usize
        } else {
            width * bytes_per_pixel
        };

        if width == 0 || height == 0 {
            return Err(format!("Invalid framebuffer resolution: {}x{}", width, height));
        }

        let smem_len = if fix_info.smem_len > 0 {
            fix_info.smem_len as usize
        } else {
            line_length * height
        };

        let mut mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(smem_len)
                .map_mut(&file)
                .map_err(|e| format!("Failed to mmap framebuffer: {}", e))?
        };

        // Clear display to pure black
        mmap.fill(0);

        let is_bgr = if bpp == 16 {
            var_info.red.offset == 0 && var_info.blue.offset >= 10
        } else {
            var_info.red.offset == 0 && var_info.blue.offset >= 16
        };

        let offset_x = if width > SCREEN_WIDTH { (width - SCREEN_WIDTH) / 2 } else { 0 };
        let offset_y = if height > SCREEN_HEIGHT { (height - SCREEN_HEIGHT) / 2 } else { 0 };

        let tty = TtyManager::new();

        eprintln!(
            "[Pi Framebuffer] Opened {}: {}x{} @ {} bpp (stride: {}B, BGR: {}). Centering: ({}, {})",
            path, width, height, bpp, line_length, is_bgr, offset_x, offset_y
        );

        Ok(Self {
            _file: file,
            mmap,
            width,
            height,
            line_length,
            bpp,
            is_bgr,
            offset_x,
            offset_y,
            tty,
        })
    }

    pub fn is_open(&self) -> bool {
        !self.tty.poll_exit()
    }

    pub fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        let copy_width = SCREEN_WIDTH.min(self.width.saturating_sub(self.offset_x));
        let copy_height = SCREEN_HEIGHT.min(self.height.saturating_sub(self.offset_y));

        if self.bpp == 16 {
            // 16 bpp RGB565 / BGR565 format (2 bytes per pixel)
            let is_bgr565 = self.is_bgr;
            for y in 0..copy_height {
                let src_start = y * SCREEN_WIDTH;
                let row_offset = (self.offset_y + y) * self.line_length;
                let dst_byte_start = row_offset + self.offset_x * 2;
                let dst_byte_end = dst_byte_start + copy_width * 2;

                if dst_byte_end <= self.mmap.len() {
                    unsafe {
                        let dst_ptr = self.mmap.as_mut_ptr().add(dst_byte_start) as *mut u16;
                        for x in 0..copy_width {
                            let pixel = buffer[src_start + x];
                            let r = ((pixel >> 16) & 0xFF) as u16;
                            let g = ((pixel >> 8) & 0xFF) as u16;
                            let b = (pixel & 0xFF) as u16;

                            let val16 = if is_bgr565 {
                                ((b >> 3) << 11) | ((g >> 2) << 5) | (r >> 3)
                            } else {
                                ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
                            };

                            *dst_ptr.add(x) = val16;
                        }
                    }
                }
            }
        } else {
            // 24 bpp / 32 bpp ARGB / XRGB format (4 bytes per pixel)
            if !self.is_bgr {
                // Standard Little-Endian 32-bit XRGB (matches buffer in memory)
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let src_slice = &buffer[src_start..src_start + copy_width];
                    let dst_offset = (self.offset_y + y) * self.line_length + self.offset_x * 4;
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
                // BGRX byte swap for screens with Red at byte 0
                for y in 0..copy_height {
                    let src_start = y * SCREEN_WIDTH;
                    let dst_offset = (self.offset_y + y) * self.line_length + self.offset_x * 4;

                    if dst_offset + copy_width * 4 <= self.mmap.len() {
                        let dst_ptr = unsafe { self.mmap.as_mut_ptr().add(dst_offset) as *mut u32 };
                        for x in 0..copy_width {
                            let pixel = buffer[src_start + x];
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
        }

        Ok(())
    }
}
