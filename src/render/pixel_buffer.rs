//! Conversion from the renderer's RGBA pixmap to the framebuffer pixel format.

use crate::configs::display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use tiny_skia::Pixmap;

pub struct PixelBuffer {
    pixels: Vec<u32>,
}

impl PixelBuffer {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn update_from_pixmap(&mut self, pixmap: &Pixmap) {
        for (pixel, rgba) in self.pixels.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            let [red, green, blue, _alpha] = [rgba[0], rgba[1], rgba[2], rgba[3]];
            *pixel = u32::from(red) << 16 | u32::from(green) << 8 | u32::from(blue);
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.pixels
    }
}
