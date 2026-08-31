mod color;
mod framebuffer;
mod rasterizer;

use color::Srgb8;
pub use framebuffer::{Framebuffer, RenderError};
use glam::Vec2;
use rasterizer::{Vertex, fill_triangle};

const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);
const TRIANGLE_COLORS: [Srgb8; 3] = [Srgb8::RED, Srgb8::GREEN, Srgb8::BLUE];

pub fn render_triangle(width: u32, height: u32) -> Result<Framebuffer, RenderError> {
    let mut frame = Framebuffer::new(width, height, BACKGROUND)?;
    let vertices = [
        Vertex::new(
            Vec2::new(0.5 * width as f32, 0.2 * height as f32),
            TRIANGLE_COLORS[0].to_linear(),
        ),
        Vertex::new(
            Vec2::new(0.2 * width as f32, 0.8 * height as f32),
            TRIANGLE_COLORS[1].to_linear(),
        ),
        Vertex::new(
            Vec2::new(0.8 * width as f32, 0.8 * height as f32),
            TRIANGLE_COLORS[2].to_linear(),
        ),
    ];
    fill_triangle(&mut frame, vertices);

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::Path;

    const GOLDEN_PATH: &str = "tests/goldens/first_triangle.png";

    #[test]
    fn triangle_frame_has_expected_layout() {
        let frame = render_triangle(800, 800).expect("triangle should render");

        assert_eq!(frame.width(), 800);
        assert_eq!(frame.height(), 800);
        assert_eq!(frame.pixels().len(), 800 * 800 * 4);
        assert!(frame.pixels().chunks_exact(4).all(|pixel| pixel[3] == 0xff));
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        assert_eq!(
            render_triangle(0, 800),
            Err(RenderError::EmptyFrame {
                width: 0,
                height: 800
            })
        );
        assert_eq!(
            render_triangle(800, 0),
            Err(RenderError::EmptyFrame {
                width: 800,
                height: 0
            })
        );
    }

    #[test]
    fn triangle_matches_golden_pixels() {
        let frame = render_triangle(800, 800).expect("triangle should render");
        let golden_path = Path::new(GOLDEN_PATH);

        if std::env::var_os("APOLLO18_UPDATE_GOLDENS").is_some() {
            write_png(golden_path, &frame).expect("golden should be written");
        }

        let (width, height, pixels) = read_png(golden_path).expect("golden should be readable");
        assert_eq!(frame.width(), width);
        assert_eq!(frame.height(), height);
        assert_eq!(frame.pixels(), pixels);
    }

    fn read_png(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let mut reader = decoder.read_info()?;
        let mut pixels = vec![0; reader.output_buffer_size()];
        let output = reader.next_frame(&mut pixels)?;
        pixels.truncate(output.buffer_size());

        assert_eq!(output.color_type, png::ColorType::Rgba);
        assert_eq!(output.bit_depth, png::BitDepth::Eight);

        Ok((output.width, output.height, pixels))
    }

    fn write_png(path: &Path, frame: &Framebuffer) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, frame.width(), frame.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(frame.pixels())?;
        Ok(())
    }
}
