use crate::error::Error;
use apollo18_renderer::{
    image::{decode_jpeg, decode_lunar_elevation_tiff},
    lunar_globe::{LunarColorMap, LunarElevationMap, LunarEphemeris},
};

const LUNAR_COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../assets/nasa/lroc_color_2k.jpg");
const LUNAR_ELEVATION_MAP_TIFF: &[u8] = include_bytes!("../../../assets/nasa/ldem_4_uint.tif");
const LUNAR_EPHEMERIS_JSON: &[u8] = include_bytes!("../../../assets/nasa/mooninfo_2026.json");

pub(crate) struct BundledAssets {
    pub(crate) color_map: LunarColorMap,
    pub(crate) elevation_map: LunarElevationMap,
    pub(crate) ephemeris: LunarEphemeris,
}

impl BundledAssets {
    pub(crate) fn load() -> Result<Self, Error> {
        let color_map = LunarColorMap::new(decode_jpeg(LUNAR_COLOR_MAP_JPEG).map_err(|error| {
            Error::with_context("could not decode the bundled lunar color map", error)
        })?);
        let elevation_map = LunarElevationMap::new(
            decode_lunar_elevation_tiff(LUNAR_ELEVATION_MAP_TIFF).map_err(|error| {
                Error::with_context("could not decode the bundled lunar elevation map", error)
            })?,
        );
        let ephemeris = LunarEphemeris::from_nasa_json(LUNAR_EPHEMERIS_JSON).map_err(|error| {
            Error::with_context("could not decode the bundled lunar ephemeris", error)
        })?;

        Ok(Self {
            color_map,
            elevation_map,
            ephemeris,
        })
    }
}
