use apollo18_native::run_frame_sequence;
use apollo18_renderer::{
    LunarColorMap, LunarElevationMap, image::decode_float_tiff, image::decode_jpeg,
    render_lunar_globe,
};
use std::error::Error;
use std::path::Path;

const DEFAULT_OUTPUT_DIRECTORY: &str = "target/apollo18/lunar-globe/frames";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;
const LUNAR_COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../../assets/nasa/lroc_color_2k.jpg");
const LUNAR_ELEVATION_MAP_TIFF: &[u8] = include_bytes!("../../../../assets/nasa/ldem_4.tif");

fn main() -> Result<(), Box<dyn Error>> {
    let color_map = LunarColorMap::new(decode_jpeg(LUNAR_COLOR_MAP_JPEG)?);
    let elevation_map = LunarElevationMap::new(decode_float_tiff(LUNAR_ELEVATION_MAP_TIFF)?);
    run_frame_sequence(
        "lunar-globe",
        Path::new(DEFAULT_OUTPUT_DIRECTORY),
        std::env::args_os().skip(1),
        |scene_time| {
            render_lunar_globe(
                CANONICAL_WIDTH,
                CANONICAL_HEIGHT,
                scene_time,
                &color_map,
                &elevation_map,
            )
        },
    )
}
