use glam::{Mat4, Vec3};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LunarAppearance {
    object_to_world: Mat4,
    sun_direction: SunDirection,
}

impl LunarAppearance {
    pub const fn new(sun_direction: SunDirection) -> Self {
        Self {
            object_to_world: Mat4::IDENTITY,
            sun_direction,
        }
    }

    pub const fn sun_direction(self) -> SunDirection {
        self.sun_direction
    }

    pub(crate) const fn with_object_to_world(
        object_to_world: Mat4,
        sun_direction: SunDirection,
    ) -> Self {
        Self {
            object_to_world,
            sun_direction,
        }
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
    use crate::{LunarColorMap, LunarElevationMap, render_lunar_globe};

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

        let appearance = LunarAppearance::new(sun_direction);

        assert_eq!(appearance.sun_direction(), sun_direction);
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
            SunDirection::new(Vec3::NEG_Z).expect("Sun direction should be valid"),
        );

        let first = render_lunar_globe(32, 24, appearance, &color_map, &elevation_map)
            .expect("lunar globe should render");
        let second = render_lunar_globe(32, 24, appearance, &color_map, &elevation_map)
            .expect("lunar globe should render");

        assert_eq!(first, second);
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
            LunarAppearance::new(front_sun),
            &color_map,
            &elevation_map,
        )
        .expect("lunar globe should render");
        let changed_sun = render_lunar_globe(
            32,
            24,
            LunarAppearance::new(side_sun),
            &color_map,
            &elevation_map,
        )
        .expect("lunar globe should render");

        assert_ne!(changed_sun, baseline);
    }
}
