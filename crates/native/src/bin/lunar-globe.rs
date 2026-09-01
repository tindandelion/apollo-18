use apollo18_native::run_frame_sequence;
use apollo18_renderer::render_lunar_globe;
use std::error::Error;
use std::path::Path;

const DEFAULT_OUTPUT_DIRECTORY: &str = "target/apollo18/lunar-globe/frames";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    run_frame_sequence(
        "lunar-globe",
        Path::new(DEFAULT_OUTPUT_DIRECTORY),
        std::env::args_os().skip(1),
        |scene_time| render_lunar_globe(CANONICAL_WIDTH, CANONICAL_HEIGHT, scene_time),
    )
}
