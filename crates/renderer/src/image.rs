use crate::color::Srgb8;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrgbImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl SrgbImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, InvalidSrgbImage> {
        let expected_length = expected_rgb_bytes(width, height);
        if width == 0 || height == 0 || expected_length != Some(pixels.len()) {
            return Err(InvalidSrgbImage {
                width,
                height,
                pixel_bytes: pixels.len(),
            });
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn pixel(&self, x: u32, y: u32) -> Srgb8 {
        let offset = (y as usize * self.width as usize + x as usize) * 3;
        Srgb8::from_channels(
            self.pixels[offset..offset + 3]
                .try_into()
                .expect("validated sRGB image pixel should contain three channels"),
        )
    }
}

fn expected_rgb_bytes(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSrgbImage {
    width: u32,
    height: u32,
    pixel_bytes: usize,
}

impl fmt::Display for InvalidSrgbImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match expected_rgb_bytes(self.width, self.height) {
            Some(expected) => write!(
                formatter,
                "sRGB image dimensions {}x{} require {} RGB bytes, received {}",
                self.width, self.height, expected, self.pixel_bytes
            ),
            None => write!(
                formatter,
                "sRGB image dimensions {}x{} exceed the addressable RGB size",
                self.width, self.height
            ),
        }
    }
}

impl Error for InvalidSrgbImage {}

#[derive(Debug)]
pub struct JpegDecodeError(image::ImageError);

impl fmt::Display for JpegDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "JPEG decoding failed: {}", self.0)
    }
}

impl Error for JpegDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

pub fn decode_jpeg(bytes: &[u8]) -> Result<SrgbImage, JpegDecodeError> {
    let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg)
        .map_err(JpegDecodeError)?
        .into_rgb8();
    let (width, height) = decoded.dimensions();

    Ok(SrgbImage::new(width, height, decoded.into_raw())
        .expect("decoded JPEG dimensions and RGB storage should agree"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MALFORMED_JPEG_FIXTURE: &[u8] = &[
        0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0xff,
        0xc0, 0x00, 0x11, 0x08, 0x00, 0x01, 0x00, 0x01, 0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01,
        0x03, 0x11, 0x01, 0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xc4, 0x00, 0x14, 0x10,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xff, 0xda, 0x00, 0x0c, 0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3f,
        0x00, 0x00, 0xff, 0xd9,
    ];

    #[test]
    fn decodes_jpeg_bytes_into_owned_srgb_pixels() {
        let image = decode_jpeg(include_bytes!("../../../assets/nasa/lroc_color_2k.jpg"))
            .expect("canonical JPEG should decode");

        assert_eq!((image.width(), image.height()), (2048, 1024));
        assert_ne!(image.pixel(1024, 512).channels(), [0, 0, 0]);
    }

    #[test]
    fn rejects_invalid_jpeg_bytes() {
        assert!(decode_jpeg(b"not a JPEG").is_err());
        assert!(decode_jpeg(MALFORMED_JPEG_FIXTURE).is_err());
    }

    #[test]
    fn validates_dimensions_and_rgb_storage() {
        assert!(SrgbImage::new(0, 1, Vec::new()).is_err());
        assert!(SrgbImage::new(1, 0, Vec::new()).is_err());
        assert!(SrgbImage::new(1, 1, vec![0; 2]).is_err());
        assert!(SrgbImage::new(1, 1, vec![0; 3]).is_ok());
    }
}
