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

    pub(crate) fn tangent_frame(self) -> (Vec3, Vec3) {
        let horizontal_radius = self.0.x.hypot(self.0.z);
        let east = if horizontal_radius > 0.0 {
            Vec3::new(-self.0.z, 0.0, self.0.x) / horizontal_radius
        } else {
            Vec3::NEG_X
        };
        let north = east.cross(self.0);

        (east, north)
    }

    pub(crate) fn geo_coords(self) -> GeoCoords {
        GeoCoords {
            globe_location: self,
            longitude: self.0.x.atan2(-self.0.z),
            latitude: self.0.y.asin(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GeoCoords {
    globe_location: GlobeLocation,
    longitude: f32,
    latitude: f32,
}

impl GeoCoords {
    pub(crate) const fn globe_location(self) -> GlobeLocation {
        self.globe_location
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globe_location(direction: Vec3) -> GlobeLocation {
        GlobeLocation::new(direction).expect("test direction should be finite and nonzero")
    }

    fn sampled_texel(direction: Vec3, width: u32, height: u32) -> (u32, u32) {
        globe_location(direction)
            .geo_coords()
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
        let location = globe_location(Vec3::NEG_Z);

        let (east, north) = location.tangent_frame();

        approx::assert_relative_eq!(east, Vec3::X, epsilon = 1.0e-6);
        approx::assert_relative_eq!(north, Vec3::Y, epsilon = 1.0e-6);
    }

    /// A general globe location has an orthonormal tangent frame.
    #[test]
    fn tangent_frame_is_orthonormal() {
        let location = globe_location(Vec3::new(1.0, 2.0, -3.0));

        let (east, north) = location.tangent_frame();

        approx::assert_relative_eq!(east.length(), 1.0, epsilon = 1.0e-6);
        approx::assert_relative_eq!(north.length(), 1.0, epsilon = 1.0e-6);
        approx::assert_relative_eq!(east.dot(location.as_vec3()), 0.0, epsilon = 1.0e-6);
        approx::assert_relative_eq!(north.dot(location.as_vec3()), 0.0, epsilon = 1.0e-6);
        approx::assert_relative_eq!(east.dot(north), 0.0, epsilon = 1.0e-6);
    }

    /// Exact poles use the antimeridian's east direction and opposite meridional north tangents.
    #[test]
    fn poles_have_a_defined_tangent_frame() {
        let north_pole = globe_location(Vec3::Y);
        let south_pole = globe_location(Vec3::NEG_Y);

        let north_frame = north_pole.tangent_frame();
        let south_frame = south_pole.tangent_frame();

        assert_eq!(north_frame, (Vec3::NEG_X, Vec3::NEG_Z));
        assert_eq!(south_frame, (Vec3::NEG_X, Vec3::Z));
    }

    /// Nonzero finite vectors are stored as unit globe locations.
    #[test]
    fn globe_location_normalizes_nonzero_vectors() {
        let scaled = GlobeLocation::new(Vec3::X * 2.0).expect("nonzero direction");
        let unnormalized =
            GlobeLocation::new(Vec3::new(0.1, 0.1, -1.0)).expect("nonzero direction");

        assert_eq!(scaled, GlobeLocation::new(Vec3::X).expect("unit X"));
        approx::assert_relative_eq!(unnormalized.0.length(), 1.0, epsilon = 1.0e-6);
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
        approx::assert_relative_eq!(interpolated.0.length(), 1.0, epsilon = 1.0e-6);
        approx::assert_relative_eq!(interpolated.0, expected, epsilon = 1.0e-6);
    }
}
