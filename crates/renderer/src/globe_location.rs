use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlobeLocation(Vec3);

impl GlobeLocation {
    pub(crate) fn new(direction: Vec3) -> Option<Self> {
        if direction.is_finite() && direction.length_squared() > 0.0 {
            Some(Self(direction.normalize()))
        } else {
            None
        }
    }

    pub(crate) fn interpolate(locations: [Self; 3], weights: [f32; 3]) -> Option<Self> {
        Self::new(
            locations[0].0 * weights[0] + locations[1].0 * weights[1] + locations[2].0 * weights[2],
        )
    }

    pub(crate) const fn as_vec3(self) -> Vec3 {
        self.0
    }

    pub(crate) fn longitude_latitude(self) -> LongitudeLatitude {
        LongitudeLatitude {
            longitude: self.0.x.atan2(-self.0.z),
            latitude: self.0.y.asin(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LongitudeLatitude {
    longitude: f32,
    latitude: f32,
}

impl LongitudeLatitude {
    pub(crate) const fn latitude(self) -> f32 {
        self.latitude
    }

    pub(crate) fn nearest_texel(self, width: u32, height: u32) -> (u32, u32) {
        let horizontal = (0.5 + self.longitude / std::f32::consts::TAU).rem_euclid(1.0);
        let vertical = (0.5 - self.latitude / std::f32::consts::PI).clamp(0.0, 1.0);
        let x = ((horizontal * width as f32).floor() as u32) % width;
        let y = (vertical * height as f32).floor() as u32;
        (x, y.min(height - 1))
    }

    pub(crate) fn east(self) -> Vec3 {
        Vec3::new(self.longitude.cos(), 0.0, self.longitude.sin())
    }

    pub(crate) fn north(self) -> Vec3 {
        Vec3::new(
            -self.longitude.sin() * self.latitude.sin(),
            self.latitude.cos(),
            self.longitude.cos() * self.latitude.sin(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globe_location(direction: Vec3) -> GlobeLocation {
        GlobeLocation::new(direction).expect("test direction should be finite and nonzero")
    }

    fn sampled_texel(direction: Vec3, width: u32, height: u32) -> (u32, u32) {
        globe_location(direction)
            .longitude_latitude()
            .nearest_texel(width, height)
    }

    /// Zero-degree longitude (`-Z`) lands in the horizontal center column.
    #[test]
    fn zero_longitude_selects_the_horizontal_center_texel() {
        let direction = Vec3::NEG_Z;

        let texel = sampled_texel(direction, 4, 3);

        assert_eq!(texel, (2, 1));
    }

    /// The antimeridian wraps: `+Z` and a step west share column 0, a step east is the last column.
    #[test]
    fn longitude_wraps_texels_at_the_antimeridian() {
        let beside_negative_x = Vec3::new(-0.000_001, 0.0, 1.0);
        let beside_positive_x = Vec3::new(0.000_001, 0.0, 1.0);

        let antimeridian = sampled_texel(Vec3::Z, 4, 3);
        let west = sampled_texel(beside_negative_x, 4, 3);
        let east = sampled_texel(beside_positive_x, 4, 3);

        assert_eq!(antimeridian, (0, 1));
        assert_eq!(west, (0, 1));
        assert_eq!(east, (3, 1));
    }

    /// Polar directions clamp to the first and last rows instead of wrapping latitude.
    #[test]
    fn latitude_clamps_texels_to_polar_rows() {
        let north = sampled_texel(Vec3::Y, 4, 3);
        let south = sampled_texel(Vec3::NEG_Y, 4, 3);

        assert_eq!(north, (0, 0));
        assert_eq!(south, (0, 2));
    }

    /// Nearest-neighbor lookup selects exactly one texel, not a blend of neighbors.
    #[test]
    fn nearest_neighbor_selects_one_texel() {
        let direction = Vec3::new(0.1, 0.1, -1.0);

        let texel = sampled_texel(direction, 4, 3);

        assert_eq!(texel, (2, 1));
    }

    /// Zero-degree longitude faces `+X` east and `+Y` north.
    #[test]
    fn zero_longitude_equator_has_canonical_east_and_north() {
        let coords = globe_location(Vec3::NEG_Z).longitude_latitude();

        let east = coords.east();
        let north = coords.north();

        assert!((east - Vec3::X).length() < 1.0e-6);
        assert!((north - Vec3::Y).length() < 1.0e-6);
    }

    /// Nonzero finite vectors are stored as unit globe locations.
    #[test]
    fn globe_location_normalizes_nonzero_vectors() {
        let scaled = GlobeLocation::new(Vec3::X * 2.0).expect("nonzero direction");
        let unnormalized =
            GlobeLocation::new(Vec3::new(0.1, 0.1, -1.0)).expect("nonzero direction");

        assert_eq!(scaled, GlobeLocation::new(Vec3::X).expect("unit X"));
        assert!((unnormalized.0.length() - 1.0).abs() < 1.0e-6);
        assert_eq!(unnormalized.0, Vec3::new(0.1, 0.1, -1.0).normalize());
    }

    /// Zero and non-finite vectors are not globe locations.
    #[test]
    fn globe_location_rejects_zero_and_non_finite_vectors() {
        for invalid in [
            Vec3::ZERO,
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::new(0.0, f32::INFINITY, 0.0),
            Vec3::new(0.0, 0.0, f32::NEG_INFINITY),
        ] {
            assert!(GlobeLocation::new(invalid).is_none());
        }
    }

    /// Barycentric interpolation of globe locations is renormalized to unit length.
    #[test]
    fn interpolated_globe_location_is_unit_length() {
        let first = globe_location(Vec3::X);
        let second = globe_location(Vec3::NEG_Z);
        let interpolated = GlobeLocation::interpolate([first, second, second], [0.5, 0.5, 0.0])
            .expect("nonzero interpolated globe location");

        let expected = (Vec3::X + Vec3::NEG_Z).normalize();
        assert!((interpolated.0.length() - 1.0).abs() < 1.0e-6);
        assert!((interpolated.0 - expected).length() < 1.0e-6);
    }
}
