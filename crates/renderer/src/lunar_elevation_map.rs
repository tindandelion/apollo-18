use crate::globe_location::GlobeLocation;
use crate::image::ElevationImage;
use glam::Vec3;

const LUNAR_REFERENCE_RADIUS_KM: f32 = 1737.4;

#[derive(Debug, Clone, PartialEq)]
pub struct LunarElevationMap {
    image: ElevationImage,
}

impl LunarElevationMap {
    pub fn new(image: ElevationImage) -> Self {
        Self { image }
    }

    pub const fn width(&self) -> u32 {
        self.image.width()
    }

    pub const fn height(&self) -> u32 {
        self.image.height()
    }

    pub(crate) fn perturbed_radial(&self, globe_location: GlobeLocation) -> Vec3 {
        let location = globe_location.as_vec3();
        let coords = globe_location.longitude_latitude();
        let (x, y) = coords.nearest_texel(self.width(), self.height());
        let (eastward_slope, northward_slope) = self.physical_slopes(x, y, coords.latitude());

        location - eastward_slope * coords.east() - northward_slope * coords.north()
    }

    fn physical_slopes(&self, x: u32, y: u32, latitude: f32) -> (f32, f32) {
        let delta_longitude = std::f32::consts::TAU / self.width() as f32;
        let delta_latitude = std::f32::consts::PI / self.height() as f32;

        if y == 0 || y + 1 == self.height() {
            let northward_derivative = if self.height() == 1 {
                0.0
            } else if y == 0 {
                (self.image.sample(x, 0) - self.image.sample(x, 1)) / delta_latitude
            } else {
                (self.image.sample(x, self.height() - 2) - self.image.sample(x, self.height() - 1))
                    / delta_latitude
            };
            (0.0, northward_derivative / LUNAR_REFERENCE_RADIUS_KM)
        } else {
            let east_x = (x + 1) % self.width();
            let west_x = (x + self.width() - 1) % self.width();
            let longitude_derivative = (self.image.sample(east_x, y)
                - self.image.sample(west_x, y))
                / (2.0 * delta_longitude);
            let latitude_derivative = (self.image.sample(x, y - 1) - self.image.sample(x, y + 1))
                / (2.0 * delta_latitude);
            (
                longitude_derivative / (LUNAR_REFERENCE_RADIUS_KM * latitude.cos()),
                latitude_derivative / LUNAR_REFERENCE_RADIUS_KM,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ElevationImage;
    use glam::Vec3;

    fn elevation_map(width: u32, height: u32, samples: Vec<f32>) -> LunarElevationMap {
        LunarElevationMap::new(
            ElevationImage::new(width, height, samples).expect("valid synthetic elevation map"),
        )
    }

    fn globe_location(direction: Vec3) -> GlobeLocation {
        GlobeLocation::new(direction).expect("test direction should be finite and nonzero")
    }

    /// A constant elevation map does not tilt the globe location.
    #[test]
    fn flat_elevation_keeps_the_perturbed_radial() {
        let map = elevation_map(4, 3, vec![0.0; 12]);

        for direction in [Vec3::NEG_Z, Vec3::X, Vec3::Y, Vec3::NEG_Y] {
            let perturbed_radial = map.perturbed_radial(globe_location(direction));

            assert!((perturbed_radial - direction).length() < 1.0e-6);
        }
    }

    /// An equatorial east-west elevation ramp subtracts (Δh / (R Δλ)) along east from the globe location.
    #[test]
    fn equatorial_east_west_ramp_tilts_by_the_physical_slope() {
        let mut samples = vec![0.0; 12];
        samples[5] = -1.0;
        samples[7] = 1.0;
        let map = elevation_map(4, 3, samples);
        let slope_east = 2.0 / (std::f32::consts::PI * 1737.4);
        let expected = Vec3::new(-slope_east, 0.0, -1.0);

        let perturbed_radial = map.perturbed_radial(globe_location(Vec3::NEG_Z));

        assert!((perturbed_radial - expected).length() < 1.0e-5);
    }

    /// Longitude neighbors wrap so a ramp across the antimeridian still uses the opposite column.
    #[test]
    fn elevation_gradients_wrap_at_the_antimeridian() {
        let mut samples = vec![0.0; 12];
        samples[5] = 1.0;
        samples[7] = -1.0;
        let map = elevation_map(4, 3, samples);
        let slope_east = 2.0 / (std::f32::consts::PI * 1737.4);
        let expected = Vec3::new(slope_east, 0.0, 1.0);

        let perturbed_radial = map.perturbed_radial(globe_location(Vec3::Z));

        assert!((perturbed_radial - expected).length() < 1.0e-5);
    }

    /// Polar rows ignore longitude differences so a huge east-west jump cannot explode the eastward tilt.
    #[test]
    fn polar_rows_zero_the_eastward_slope() {
        let mut samples = vec![0.0; 12];
        samples[1] = -10.0;
        samples[3] = 10.0;
        let map = elevation_map(4, 3, samples);

        let perturbed_radial = map.perturbed_radial(globe_location(Vec3::Y));

        assert!(perturbed_radial.x.abs() < 1.0e-6);
    }

    /// Polar-row texels use a one-sided latitude difference in the local meridian.
    #[test]
    fn north_polar_row_uses_a_one_sided_latitude_difference() {
        let mut samples = vec![0.0; 12];
        samples[0] = 1.0;
        samples[1] = 1.0;
        samples[2] = 1.0;
        samples[3] = 1.0;
        let map = elevation_map(4, 3, samples);
        let location = Vec3::new(0.0, 1.0, -1.0).normalize();
        let slope_north = 3.0 / (std::f32::consts::PI * 1737.4);
        let north = Vec3::new(0.0, -location.z, location.y);
        let expected = location - slope_north * north;

        let perturbed_radial = map.perturbed_radial(globe_location(location));

        assert!((perturbed_radial - expected).length() < 1.0e-5);
    }
}
