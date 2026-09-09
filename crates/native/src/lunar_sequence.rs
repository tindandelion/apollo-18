use crate::run_frame_sequence;
use apollo18_renderer::{
    LunarColorMap, LunarElevationMap, LunarEphemeris, SynodicMonthAnimation,
    image::decode_float_tiff, image::decode_jpeg, render_lunar_globe,
};
use std::error::Error;
use std::ffi::OsString;
use std::path::Path;

const DEFAULT_OUTPUT_DIRECTORY: &str = "target/apollo18/lunar-globe/frames";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;
const LUNAR_COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../assets/nasa/lroc_color_2k.jpg");
const LUNAR_ELEVATION_MAP_TIFF: &[u8] = include_bytes!("../../../assets/nasa/ldem_4.tif");

pub fn run_lunar_globe_sequence<I, S>(
    arguments: I,
    ephemeris_json: &[u8],
) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let ephemeris = LunarEphemeris::from_nasa_json(ephemeris_json)?;
    let animation = SynodicMonthAnimation::new(ephemeris)?;
    let color_map = LunarColorMap::new(decode_jpeg(LUNAR_COLOR_MAP_JPEG)?);
    let elevation_map = LunarElevationMap::new(decode_float_tiff(LUNAR_ELEVATION_MAP_TIFF)?);

    run_frame_sequence(
        "lunar-globe",
        Path::new(DEFAULT_OUTPUT_DIRECTORY),
        arguments,
        |scene_time| {
            render_lunar_globe(
                CANONICAL_WIDTH,
                CANONICAL_HEIGHT,
                animation.lunar_appearance(scene_time),
                &color_map,
                &elevation_map,
            )
        },
    )
}
