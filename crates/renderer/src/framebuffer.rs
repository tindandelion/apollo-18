use crate::color::Srgb8;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub(crate) fn new(width: u32, height: u32, background: Srgb8) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::EmptyFrame { width, height });
        }

        Srgb8::init_lookup_table();

        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .and_then(|count| count.checked_mul(4))
            .ok_or(RenderError::FrameTooLarge { width, height })?;

        let mut frame = Self {
            width,
            height,
            pixels: vec![0; pixel_count],
        };
        frame.clear(background);
        Ok(frame)
    }

    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, color: Srgb8) {
        let offset = ((y as usize * self.width as usize) + x as usize) * 4;
        Self::write_opaque_pixel(&mut self.pixels[offset..offset + 4], color);
    }

    fn clear(&mut self, color: Srgb8) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            Self::write_opaque_pixel(pixel, color);
        }
    }

    fn write_opaque_pixel(pixel: &mut [u8], color: Srgb8) {
        pixel[..3].copy_from_slice(&color.channels());
        pixel[3] = 0xff;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    EmptyFrame { width: u32, height: u32 },
    FrameTooLarge { width: u32, height: u32 },
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFrame { width, height } => {
                write!(f, "frame dimensions must be non-zero, got {width}x{height}")
            }
            Self::FrameTooLarge { width, height } => {
                write!(f, "frame dimensions are too large, got {width}x{height}")
            }
        }
    }
}

impl Error for RenderError {}
