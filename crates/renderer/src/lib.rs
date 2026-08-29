use glam::Vec2;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const BACKGROUND: Rgba8 = Rgba8::new(0x18, 0x18, 0x18, 0xff);
const TRIANGLE_COLOR: Rgba8 = Rgba8::new(0xd8, 0xd8, 0xd8, 0xff);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaFrame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl RgbaFrame {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
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

pub fn render_triangle(width: u32, height: u32) -> Result<RgbaFrame, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::EmptyFrame { width, height });
    }

    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .and_then(|count| count.checked_mul(4))
        .ok_or(RenderError::FrameTooLarge { width, height })?;

    let mut frame = RgbaFrame {
        width,
        height,
        pixels: vec![0; pixel_count],
    };
    clear(&mut frame, BACKGROUND);

    let triangle = [
        Point::new(0.5 * width as f32, 0.2 * height as f32),
        Point::new(0.2 * width as f32, 0.8 * height as f32),
        Point::new(0.8 * width as f32, 0.8 * height as f32),
    ];
    fill_triangle(&mut frame, triangle, TRIANGLE_COLOR);

    Ok(frame)
}

fn clear(frame: &mut RgbaFrame, color: Rgba8) {
    for pixel in frame.pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&color.channels);
    }
}

fn fill_triangle(frame: &mut RgbaFrame, mut vertices: [Point; 3], color: Rgba8) {
    let area = edge(vertices[0], vertices[1], vertices[2]);
    if area == 0.0 {
        return;
    }
    if area < 0.0 {
        vertices.swap(1, 2);
    }

    let min_x = vertices
        .iter()
        .copied()
        .map(Point::x)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_x = vertices
        .iter()
        .copied()
        .map(Point::x)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(frame.width as f32) as u32;
    let min_y = vertices
        .iter()
        .copied()
        .map(Point::y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_y = vertices
        .iter()
        .copied()
        .map(Point::y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(frame.height as f32) as u32;

    let edges = [
        Edge::new(vertices[0], vertices[1]),
        Edge::new(vertices[1], vertices[2]),
        Edge::new(vertices[2], vertices[0]),
    ];

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample = Point::new(x as f32 + 0.5, y as f32 + 0.5);
            if edges.iter().all(|candidate| candidate.contains(sample)) {
                set_pixel(frame, x, y, color);
            }
        }
    }
}

fn set_pixel(frame: &mut RgbaFrame, x: u32, y: u32, color: Rgba8) {
    let offset = ((y as usize * frame.width as usize) + x as usize) * 4;
    frame.pixels[offset..offset + 4].copy_from_slice(&color.channels);
}

#[derive(Debug, Clone, Copy)]
struct Rgba8 {
    channels: [u8; 4],
}

impl Rgba8 {
    const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            channels: [red, green, blue, alpha],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Point(Vec2);

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    fn x(self) -> f32 {
        self.0.x
    }

    fn y(self) -> f32 {
        self.0.y
    }
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    start: Point,
    end: Point,
    includes_on_edge_samples: bool,
}

impl Edge {
    fn new(start: Point, end: Point) -> Self {
        let direction = end.0 - start.0;
        Self {
            start,
            end,
            includes_on_edge_samples: direction.y < 0.0
                || (direction.y == 0.0 && direction.x > 0.0),
        }
    }

    fn contains(self, point: Point) -> bool {
        let value = edge(self.start, self.end, point);
        value > 0.0 || (value == 0.0 && self.includes_on_edge_samples)
    }
}

fn edge(start: Point, end: Point, point: Point) -> f32 {
    (end.0 - start.0).perp_dot(point.0 - start.0)
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let expected = read_png(golden_path).expect("golden should be readable");
        assert_eq!(frame, expected);
    }

    fn read_png(path: &Path) -> Result<RgbaFrame, Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let mut reader = decoder.read_info()?;
        let mut pixels = vec![0; reader.output_buffer_size()];
        let output = reader.next_frame(&mut pixels)?;
        pixels.truncate(output.buffer_size());

        assert_eq!(output.color_type, png::ColorType::Rgba);
        assert_eq!(output.bit_depth, png::BitDepth::Eight);

        Ok(RgbaFrame {
            width: output.width,
            height: output.height,
            pixels,
        })
    }

    fn write_png(path: &Path, frame: &RgbaFrame) -> Result<(), Box<dyn Error>> {
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
