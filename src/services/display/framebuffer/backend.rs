//! Linux framebuffer backend for Raspberry Pi OS TTY consoles.

use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::models::display::framebuffer::{
    FBIOGET_FSCREENINFO, FBIOGET_VSCREENINFO, FBIOPUT_VSCREENINFO, FbFixScreeninfo, FbVarScreeninfo,
};
use crate::services::input::{TtyManager, is_app_running};
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

pub struct FramebufferBackend {
    _file: File,
    mmap: MmapMut,
    pub width: usize,
    pub height: usize,
    pub line_length: usize,
    pub bpp: usize,
    is_bgr: bool,
    x_map: Vec<usize>,
    y_map: Vec<usize>,
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

        if var_info.bits_per_pixel != 32 {
            let mut request = var_info;
            request.bits_per_pixel = 32;
            request.activate = 0;
            if unsafe { libc::ioctl(fd, FBIOPUT_VSCREENINFO, &mut request) } == 0 {
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
            return Err(format!(
                "Invalid framebuffer resolution: {}x{}",
                width, height
            ));
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
        mmap.fill(0);

        let is_bgr = if bpp == 16 {
            var_info.red.offset == 0 && var_info.blue.offset >= 10
        } else {
            var_info.red.offset == 0 && var_info.blue.offset >= 16
        };
        let x_map = (0..width).map(|x| (x * SCREEN_WIDTH) / width).collect();
        let y_map = (0..height).map(|y| (y * SCREEN_HEIGHT) / height).collect();
        let tty = TtyManager::new();

        eprintln!(
            "[Pi Framebuffer] Opened {}: {}x{} @ {} bpp (stride: {}B, BGR: {}). Fullscreen scaling active.",
            path, width, height, bpp, line_length, is_bgr
        );
        Ok(Self {
            _file: file,
            mmap,
            width,
            height,
            line_length,
            bpp,
            is_bgr,
            x_map,
            y_map,
            tty,
        })
    }

    pub fn is_open(&self) -> bool {
        is_app_running()
    }

    pub fn poll_key(&mut self) -> Option<u8> {
        self.tty.poll_key()
    }

    pub fn present(&mut self, buffer: &[u32]) -> Result<(), String> {
        let is_1to1 = self.width == SCREEN_WIDTH && self.height == SCREEN_HEIGHT;
        if self.bpp == 16 {
            self.present_rgb565(buffer, is_1to1);
        } else {
            self.present_rgb32(buffer, is_1to1);
        }
        Ok(())
    }

    fn present_rgb565(&mut self, buffer: &[u32], is_1to1: bool) {
        let is_bgr565 = self.is_bgr;
        for y in 0..self.height {
            let src_y = if is_1to1 { y } else { self.y_map[y] };
            let dst_offset = y * self.line_length;
            if dst_offset + self.width * 2 > self.mmap.len() {
                continue;
            }
            unsafe {
                let dst_ptr = self.mmap.as_mut_ptr().add(dst_offset) as *mut u16;
                for x in 0..self.width {
                    let src_x = if is_1to1 { x } else { self.x_map[x] };
                    *dst_ptr.add(x) = rgb565(buffer[src_y * SCREEN_WIDTH + src_x], is_bgr565);
                }
            }
        }
    }

    fn present_rgb32(&mut self, buffer: &[u32], is_1to1: bool) {
        if is_1to1 && !self.is_bgr {
            for y in 0..SCREEN_HEIGHT {
                let src_start = y * SCREEN_WIDTH;
                let dst_offset = y * self.line_length;
                let byte_count = SCREEN_WIDTH * 4;
                if dst_offset + byte_count <= self.mmap.len() {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            buffer[src_start..src_start + SCREEN_WIDTH].as_ptr() as *const u8,
                            self.mmap.as_mut_ptr().add(dst_offset),
                            byte_count,
                        );
                    }
                }
            }
            return;
        }

        for y in 0..self.height {
            let src_y = if is_1to1 { y } else { self.y_map[y] };
            let dst_offset = y * self.line_length;
            if dst_offset + self.width * 4 > self.mmap.len() {
                continue;
            }
            unsafe {
                let dst_ptr = self.mmap.as_mut_ptr().add(dst_offset) as *mut u32;
                for x in 0..self.width {
                    let src_x = if is_1to1 { x } else { self.x_map[x] };
                    let pixel = buffer[src_y * SCREEN_WIDTH + src_x];
                    *dst_ptr.add(x) = if self.is_bgr {
                        (pixel & 0x0000FF00)
                            | ((pixel & 0x00FF0000) >> 16)
                            | ((pixel & 0x000000FF) << 16)
                            | 0xFF000000
                    } else {
                        pixel | 0xFF000000
                    };
                }
            }
        }
    }
}

fn rgb565(pixel: u32, is_bgr: bool) -> u16 {
    let r = ((pixel >> 16) & 0xFF) as u16;
    let g = ((pixel >> 8) & 0xFF) as u16;
    let b = (pixel & 0xFF) as u16;
    if is_bgr {
        ((b >> 3) << 11) | ((g >> 2) << 5) | (r >> 3)
    } else {
        ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
    }
}
