use glam::{Mat4, Vec3};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarAppearance {
    object_to_world: Mat4,
    sun_direction: SunDirection,
}

impl LunarAppearance {
    pub const fn new(object_to_world: Mat4, sun_direction: SunDirection) -> Self {
        Self {
            object_to_world,
            sun_direction,
        }
    }

    pub const fn sun_direction(self) -> SunDirection {
        self.sun_direction
    }

    pub(crate) const fn object_to_world(self) -> Mat4 {
        self.object_to_world
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunDirection(Vec3);

impl SunDirection {
    pub fn new(direction: Vec3) -> Result<Self, InvalidSunDirection> {
        let largest_component = direction.abs().max_element();
        if !direction.is_finite() || largest_component == 0.0 {
            return Err(InvalidSunDirection);
        }

        Ok(Self((direction / largest_component).normalize()))
    }

    pub const fn as_vec3(self) -> Vec3 {
        self.0
    }

    pub(crate) fn diffuse_intensity(self, lighting_normal: Vec3) -> f32 {
        lighting_normal.dot(self.0).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSunDirection;

impl Display for InvalidSunDirection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("Sun direction must be finite and nonzero")
    }
}

impl Error for InvalidSunDirection {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::{ElevationImage, SrgbImage};
    use crate::lunar_globe::{LunarColorMap, LunarElevationMap, render_lunar_globe};

    /// A Sun direction normalizes finite nonzero input.
    #[test]
    fn sun_direction_normalizes_finite_nonzero_input() {
        let input = Vec3::new(0.0, 0.0, -2.0);

        let direction = SunDirection::new(input).expect("finite nonzero direction should be valid");

        assert_eq!(direction.0, Vec3::NEG_Z);
    }

    /// A Sun direction rejects zero and non-finite input.
    #[test]
    fn sun_direction_rejects_invalid_input() {
        let zero = Vec3::ZERO;
        let non_finite = Vec3::NAN;

        let zero_result = SunDirection::new(zero);
        let non_finite_result = SunDirection::new(non_finite);

        assert!(zero_result.is_err());
        assert!(non_finite_result.is_err());
    }

    /// A Sun direction safely normalizes every finite nonzero magnitude.
    #[test]
    fn sun_direction_normalizes_extreme_finite_input() {
        let input = Vec3::splat(f32::MAX);

        let direction = SunDirection::new(input).expect("finite nonzero direction should be valid");

        assert!((direction.0.length() - 1.0).abs() < 1.0e-6);
    }

    /// Lunar appearance preserves its validated Sun direction.
    #[test]
    fn lunar_appearance_preserves_sun_direction() {
        let sun_direction = SunDirection::new(Vec3::NEG_Z).expect("Sun direction should be valid");

        let appearance = LunarAppearance::new(Mat4::IDENTITY, sun_direction);

        assert_eq!(appearance.sun_direction(), sun_direction);
    }

    fn white_color_map() -> LunarColorMap {
        LunarColorMap::new(
            SrgbImage::new(4, 3, vec![255; 4 * 3 * 3]).expect("color map should be valid"),
        )
    }

    fn sloped_elevation_map() -> LunarElevationMap {
        let mut samples = vec![0.0; 4 * 3];
        samples[5] = -3_000.0;
        samples[7] = 3_000.0;
        LunarElevationMap::new(
            ElevationImage::new(4, 3, samples).expect("elevation map should be valid"),
        )
    }

    fn center_pixel(framebuffer: &crate::rasterizer::Framebuffer) -> [u8; 4] {
        let x = framebuffer.width() / 2;
        let y = framebuffer.height() / 2;
        let offset = ((y * framebuffer.width() + x) * 4) as usize;
        framebuffer.pixels()[offset..offset + 4]
            .try_into()
            .expect("pixel should have four channels")
    }

    /// Equal explicit lunar rendering inputs produce equal framebuffers.
    #[test]
    fn explicit_lunar_appearance_renders_deterministically() {
        let color_map = LunarColorMap::new(
            SrgbImage::new(8, 4, vec![128; 8 * 4 * 3]).expect("color map should be valid"),
        );
        let elevation_map = LunarElevationMap::new(
            ElevationImage::new(8, 4, vec![0.0; 8 * 4]).expect("elevation map should be valid"),
        );
        let appearance = LunarAppearance::new(
            Mat4::IDENTITY,
            SunDirection::new(Vec3::NEG_Z).expect("Sun direction should be valid"),
        );

        let first = render_lunar_globe(32, 24, appearance, &color_map, &elevation_map)
            .expect("lunar globe should render");
        let second = render_lunar_globe(32, 24, appearance, &color_map, &elevation_map)
            .expect("lunar globe should render");

        assert_eq!(first, second);
    }

    /// Rotating the lunar globe pose and Sun together preserves terrain-normal illumination.
    #[test]
    fn rotated_lunar_appearance_preserves_terrain_normal_illumination() {
        let color_map = white_color_map();
        let elevation_map = sloped_elevation_map();
        let object_space_sun = Vec3::NEG_X;
        let rotation = Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let identity_appearance = LunarAppearance::new(
            Mat4::IDENTITY,
            SunDirection::new(object_space_sun).expect("Sun direction should be valid"),
        );
        let rotated_appearance = LunarAppearance::new(
            rotation,
            SunDirection::new(rotation.transform_vector3(object_space_sun))
                .expect("rotated Sun direction should be valid"),
        );

        let identity = render_lunar_globe(33, 33, identity_appearance, &color_map, &elevation_map)
            .expect("identity appearance should render");
        let rotated = render_lunar_globe(33, 33, rotated_appearance, &color_map, &elevation_map)
            .expect("rotated appearance should render");

        assert_eq!(center_pixel(&identity), center_pixel(&rotated));
        assert!(center_pixel(&identity)[0] > 0);
    }

    /// Reversing the Sun leaves the same sloped terrain normal unlit.
    #[test]
    fn terrain_normal_facing_away_from_sun_is_unlit() {
        let color_map = white_color_map();
        let elevation_map = sloped_elevation_map();
        let appearance = LunarAppearance::new(
            Mat4::IDENTITY,
            SunDirection::new(Vec3::X).expect("Sun direction should be valid"),
        );

        let framebuffer = render_lunar_globe(33, 33, appearance, &color_map, &elevation_map)
            .expect("lunar appearance should render");

        assert_eq!(center_pixel(&framebuffer), [0, 0, 0, 255]);
    }

    /// Terrain-normal shading can illuminate rim fragments on an otherwise new-Moon globe.
    #[test]
    fn terrain_normals_preserve_new_moon_rim_highlights() {
        let color_map = LunarColorMap::new(
            SrgbImage::new(8, 4, vec![255; 8 * 4 * 3]).expect("color map should be valid"),
        );
        let samples = (0..8 * 4)
            .map(|index| {
                let longitude =
                    (index % 8) as f32 / 8.0 * std::f32::consts::TAU - std::f32::consts::PI;
                longitude.cos() * 10_000.0
            })
            .collect();
        let elevation_map = LunarElevationMap::new(
            ElevationImage::new(8, 4, samples).expect("elevation map should be valid"),
        );
        let appearance = LunarAppearance::new(
            Mat4::IDENTITY,
            SunDirection::new(Vec3::Z).expect("Sun direction should be valid"),
        );

        let framebuffer = render_lunar_globe(65, 65, appearance, &color_map, &elevation_map)
            .expect("new-Moon appearance should render");
        let highlighted_pixel_count = framebuffer
            .pixels()
            .chunks_exact(4)
            .filter(|pixel| pixel[..3] != [0, 0, 0] && pixel[..3] != [24, 24, 24])
            .count();

        assert!(highlighted_pixel_count > 0);
    }

    /// Sun direction affects rendered lunar appearance independently of identity pose.
    #[test]
    fn sun_direction_affects_rendered_appearance() {
        let color_map = LunarColorMap::new(
            SrgbImage::new(8, 4, vec![128; 8 * 4 * 3]).expect("color map should be valid"),
        );
        let elevation_map = LunarElevationMap::new(
            ElevationImage::new(8, 4, vec![0.0; 8 * 4]).expect("elevation map should be valid"),
        );
        let front_sun = SunDirection::new(Vec3::NEG_Z).expect("Sun direction should be valid");
        let side_sun = SunDirection::new(Vec3::NEG_X).expect("Sun direction should be valid");

        let baseline = render_lunar_globe(
            32,
            24,
            LunarAppearance::new(Mat4::IDENTITY, front_sun),
            &color_map,
            &elevation_map,
        )
        .expect("lunar globe should render");
        let changed_sun = render_lunar_globe(
            32,
            24,
            LunarAppearance::new(Mat4::IDENTITY, side_sun),
            &color_map,
            &elevation_map,
        )
        .expect("lunar globe should render");

        assert_ne!(changed_sun, baseline);
    }
}
