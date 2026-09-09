mod cube;
pub mod image;
pub mod lunar_globe;
mod rasterizer;
mod scene_time;
mod triangle;

use rasterizer::Srgb8;
pub use rasterizer::{Framebuffer, RenderError};
pub use scene_time::{InvalidSceneTime, SceneTime};

const CUBE_ROTATION_PERIOD_SECONDS: f64 = 10.0;
const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);

pub fn render_triangle_showcase(width: u32, height: u32) -> Result<Framebuffer, RenderError> {
    triangle::render(width, height, BACKGROUND)
}

pub fn render_cube(
    width: u32,
    height: u32,
    scene_time: SceneTime,
) -> Result<Framebuffer, RenderError> {
    let yaw =
        30.0_f32.to_radians() + periodic_angle_radians(scene_time, CUBE_ROTATION_PERIOD_SECONDS);

    cube::render_at_yaw(width, height, BACKGROUND, yaw)
}

fn periodic_angle_radians(scene_time: SceneTime, period_seconds: f64) -> f32 {
    std::f32::consts::TAU * scene_time.cycle_fraction(period_seconds) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cube scene time maps linearly to yaw and wraps after one rotation period.
    #[test]
    fn scene_time_maps_to_rotation_angle() {
        let start = SceneTime::from_seconds(0.0).expect("scene time should be valid");
        let quarter_period = SceneTime::from_seconds(CUBE_ROTATION_PERIOD_SECONDS / 4.0)
            .expect("scene time should be valid");
        let one_period = SceneTime::from_seconds(CUBE_ROTATION_PERIOD_SECONDS)
            .expect("scene time should be valid");

        let start_yaw = periodic_angle_radians(start, CUBE_ROTATION_PERIOD_SECONDS);
        let quarter_yaw = periodic_angle_radians(quarter_period, CUBE_ROTATION_PERIOD_SECONDS);
        let wrapped_yaw = periodic_angle_radians(one_period, CUBE_ROTATION_PERIOD_SECONDS);

        assert_eq!(start_yaw, 0.0);
        assert_eq!(quarter_yaw, std::f32::consts::FRAC_PI_2);
        assert_eq!(wrapped_yaw, 0.0);
    }
}
