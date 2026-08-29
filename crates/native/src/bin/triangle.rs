use apollo18_renderer::{RgbaFrame, render_triangle};
use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

const DEFAULT_OUTPUT_PATH: &str = "target/apollo18/triangle.png";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_PATH));

    let frame = render_triangle(CANONICAL_WIDTH, CANONICAL_HEIGHT)?;
    write_png(&output_path, &frame)?;
    println!("wrote {}", output_path.display());
    Ok(())
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
