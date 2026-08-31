mod color;
mod framebuffer;
mod rasterizer;

use color::Srgb8;
pub use framebuffer::{Framebuffer, RenderError};
use glam::Vec3;
use rasterizer::{NdcVertex, Rasterizer};

const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);
const TRIANGLE_COLORS: [Srgb8; 3] = [Srgb8::RED, Srgb8::GREEN, Srgb8::BLUE];

pub fn render_triangles(width: u32, height: u32) -> Result<Framebuffer, RenderError> {
    let mut rasterizer = Rasterizer::new(width, height, BACKGROUND)?;

    let near = [
        scene_vertex(-0.8, -0.3, 0.2, TRIANGLE_COLORS[0]),
        scene_vertex(0.3, -0.55, 0.2, TRIANGLE_COLORS[1]),
        scene_vertex(-0.25, 0.7, 0.2, TRIANGLE_COLORS[2]),
    ];
    let far = [
        scene_vertex(-0.75, -0.65, 0.75, TRIANGLE_COLORS[2]),
        scene_vertex(0.75, -0.65, 0.75, TRIANGLE_COLORS[0]),
        scene_vertex(0.0, 0.75, 0.75, TRIANGLE_COLORS[1]),
    ];
    let back_facing = [
        scene_vertex(0.45, 0.25, 0.1, TRIANGLE_COLORS[0]),
        scene_vertex(0.65, 0.8, 0.1, TRIANGLE_COLORS[1]),
        scene_vertex(0.9, 0.25, 0.1, TRIANGLE_COLORS[2]),
    ];

    rasterizer.draw_triangle(near);
    rasterizer.draw_triangle(far);
    rasterizer.draw_triangle(back_facing);

    Ok(rasterizer.into_framebuffer())
}

fn scene_vertex(x: f32, y: f32, depth: f32, color: Srgb8) -> NdcVertex {
    NdcVertex::new(Vec3::new(x, y, depth), color.to_linear())
        .expect("scene NDC vertex should be valid")
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
        let frame = render_triangles(800, 800).expect("triangles should render");

        assert_eq!(frame.width(), 800);
        assert_eq!(frame.height(), 800);
        assert_eq!(frame.pixels().len(), 800 * 800 * 4);
        assert!(frame.pixels().chunks_exact(4).all(|pixel| pixel[3] == 0xff));
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        assert_eq!(
            render_triangles(0, 800),
            Err(RenderError::EmptyFrame {
                width: 0,
                height: 800
            })
        );
        assert_eq!(
            render_triangles(800, 0),
            Err(RenderError::EmptyFrame {
                width: 800,
                height: 0
            })
        );
    }

    #[test]
    fn triangle_matches_golden_pixels() {
        let frame = render_triangles(800, 800).expect("triangles should render");
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
