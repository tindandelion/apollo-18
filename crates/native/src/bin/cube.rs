use apollo18_native::write_png;
use apollo18_renderer::render_cube;
use std::error::Error;
use std::path::PathBuf;

const DEFAULT_OUTPUT_PATH: &str = "target/apollo18/cube.png";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_PATH));

    let frame = render_cube(CANONICAL_WIDTH, CANONICAL_HEIGHT)?;
    write_png(&output_path, &frame)?;
    println!("wrote {}", output_path.display());
    Ok(())
}
