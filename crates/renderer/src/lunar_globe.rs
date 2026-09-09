mod globe_location;
mod lunar_appearance;
mod lunar_color_map;
mod lunar_elevation_map;
mod lunar_ephemeris;
mod lunar_phase_animation;
mod octasphere;

pub use lunar_appearance::{InvalidSunDirection, LunarAppearance, SunDirection};
pub use lunar_color_map::LunarColorMap;
pub use lunar_elevation_map::LunarElevationMap;
pub use lunar_ephemeris::{AstronomicalInstant, EphemerisError, LunarEphemeris};
pub use lunar_phase_animation::{
    AnimationCoverageError, EphemerisSpanAnimation, SynodicMonthAnimation,
};

use crate::BACKGROUND;
use crate::rasterizer::{Framebuffer, RenderError};

pub fn render_lunar_globe(
    width: u32,
    height: u32,
    appearance: LunarAppearance,
    color_map: &LunarColorMap,
    elevation_map: &LunarElevationMap,
) -> Result<Framebuffer, RenderError> {
    octasphere::render(
        width,
        height,
        BACKGROUND,
        color_map,
        elevation_map,
        appearance,
    )
}
