mod sequence;

use apollo18_renderer::Framebuffer;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

pub use sequence::run_frame_sequence;

pub fn write_png(path: &Path, frame: &Framebuffer) -> Result<(), Box<dyn Error>> {
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
