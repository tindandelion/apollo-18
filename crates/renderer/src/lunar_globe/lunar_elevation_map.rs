use super::globe_location::GeoCoords;
use crate::image::ElevationImage;
use glam::Vec3;

const LUNAR_REFERENCE_RADIUS_KM: f32 = 1737.4;

#[derive(Debug, Clone, PartialEq)]
pub struct LunarElevationMap {
    width: u32,
    height: u32,
    terrain_normals: Vec<Vec3>,
}

impl LunarElevationMap {
    pub fn new(image: ElevationImage) -> Self {
        let width = image.width();
        let height = image.height();
        let mut terrain_normals = Vec::with_capacity(width as usize * height as usize);

        for y in 0..height {
            for x in 0..width {
                terrain_normals.push(Self::derive_terrain_normal(&image, x, y));
            }
        }

        Self {
            width,
            height,
            terrain_normals,
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn terrain_normal(&self, geo_coords: GeoCoords) -> Vec3 {
        let (x, y) = geo_coords.nearest_texel(self.width, self.height);

        self.terrain_normals[(y * self.width + x) as usize]
    }

    fn derive_terrain_normal(image: &ElevationImage, x: u32, y: u32) -> Vec3 {
        let width = image.width();
        let height = image.height();
        let longitude = ((x as f32 + 0.5) / width as f32 - 0.5) * std::f32::consts::TAU;
        let latitude = (0.5 - (y as f32 + 0.5) / height as f32) * std::f32::consts::PI;
        let (latitude_sine, horizontal_radius) = latitude.sin_cos();
        let (longitude_sine, longitude_cosine) = longitude.sin_cos();
        let location = Vec3::new(
            longitude_sine * horizontal_radius,
            latitude_sine,
            -longitude_cosine * horizontal_radius,
        );
        let east = Vec3::new(longitude_cosine, 0.0, longitude_sine);
        let north = east.cross(location);
        let (eastward_slope, northward_slope) =
            Self::physical_slopes(image, x, y, horizontal_radius);

        (location - eastward_slope * east - northward_slope * north).normalize()
    }

    fn physical_slopes(
        image: &ElevationImage,
        x: u32,
        y: u32,
        horizontal_radius: f32,
    ) -> (f32, f32) {
        let width = image.width();
        let height = image.height();
        let delta_longitude = std::f32::consts::TAU / width as f32;
        let delta_latitude = std::f32::consts::PI / height as f32;

        if y == 0 || y + 1 == height {
            let northward_derivative = if height == 1 {
                0.0
            } else if y == 0 {
                (image.sample(x, 0) - image.sample(x, 1)) / delta_latitude
            } else {
                (image.sample(x, height - 2) - image.sample(x, height - 1)) / delta_latitude
            };
            (0.0, northward_derivative / LUNAR_REFERENCE_RADIUS_KM)
        } else {
            let east_x = (x + 1) % width;
            let west_x = (x + width - 1) % width;
            let longitude_derivative =
                (image.sample(east_x, y) - image.sample(west_x, y)) / (2.0 * delta_longitude);
            let latitude_derivative =
                (image.sample(x, y - 1) - image.sample(x, y + 1)) / (2.0 * delta_latitude);
            (
                longitude_derivative / (LUNAR_REFERENCE_RADIUS_KM * horizontal_radius),
                latitude_derivative / LUNAR_REFERENCE_RADIUS_KM,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::globe_location::GlobeLocation;
    use super::*;
    use crate::image::ElevationImage;
    use glam::Vec3;

    fn elevation_map(width: u32, height: u32, samples: Vec<f32>) -> LunarElevationMap {
        LunarElevationMap::new(
            ElevationImage::new(width, height, samples).expect("valid synthetic elevation map"),
        )
    }

    fn geo_coords(direction: Vec3) -> GeoCoords {
        GlobeLocation::new(direction)
            .expect("test direction should be finite and nonzero")
            .geo_coords()
    }

    /// A flat map caches the normalized globe location at each texel center.
    #[test]
    fn flat_elevation_keeps_texel_center_normals_radial() {
        let map = elevation_map(4, 2, vec![0.0; 8]);
        let latitude = -45.0_f32.to_radians();
        let longitude = 45.0_f32.to_radians();
        let expected = Vec3::new(
            longitude.sin() * latitude.cos(),
            latitude.sin(),
            -longitude.cos() * latitude.cos(),
        );

        let terrain_normal = map.terrain_normal(geo_coords(Vec3::NEG_Z));

        approx::assert_relative_eq!(terrain_normal, expected, epsilon = 1.0e-6);
    }

    /// An equatorial east-west elevation ramp tilts the cached texel-center normal by its physical slope.
    #[test]
    fn equatorial_east_west_ramp_tilts_by_the_physical_slope() {
        let mut samples = vec![0.0; 12];
        samples[5] = -1.0;
        samples[7] = 1.0;
        let map = elevation_map(4, 3, samples);
        let angle = 45.0_f32.to_radians();
        let location = Vec3::new(angle.sin(), 0.0, -angle.cos());
        let east = Vec3::new(angle.cos(), 0.0, angle.sin());
        let slope_east = 2.0 / (std::f32::consts::PI * 1737.4);
        let expected = (location - slope_east * east).normalize();

        let terrain_normal = map.terrain_normal(geo_coords(Vec3::NEG_Z));

        approx::assert_relative_eq!(terrain_normal, expected, epsilon = 1.0e-6);
    }

    /// Longitude neighbors wrap so an antimeridian texel uses the last and first columns.
    #[test]
    fn elevation_gradients_wrap_at_the_antimeridian() {
        let mut samples = vec![0.0; 12];
        samples[5] = 1.0;
        samples[7] = -1.0;
        let map = elevation_map(4, 3, samples);
        let longitude = -135.0_f32.to_radians();
        let location = Vec3::new(longitude.sin(), 0.0, -longitude.cos());
        let east = Vec3::new(longitude.cos(), 0.0, longitude.sin());
        let slope_east = 2.0 / (std::f32::consts::PI * 1737.4);
        let expected = (location - slope_east * east).normalize();

        let terrain_normal = map.terrain_normal(geo_coords(Vec3::Z));

        approx::assert_relative_eq!(terrain_normal, expected, epsilon = 1.0e-6);
    }

    /// A near-polar non-polar row uses its texel center's horizontal radius for physical eastward distance.
    #[test]
    fn near_polar_east_west_ramp_uses_the_horizontal_radius() {
        let mut samples = vec![0.0; 4 * 180];
        samples[4 + 1] = -1.0;
        samples[4 + 3] = 1.0;
        let map = elevation_map(4, 180, samples);
        let latitude = 88.5_f32.to_radians();
        let longitude = 45.0_f32.to_radians();
        let horizontal_radius = latitude.cos();
        let location = Vec3::new(
            longitude.sin() * horizontal_radius,
            latitude.sin(),
            -longitude.cos() * horizontal_radius,
        );
        let east = Vec3::new(longitude.cos(), 0.0, longitude.sin());
        let slope_east = 2.0 / (std::f32::consts::PI * 1737.4 * horizontal_radius);
        let expected = (location - slope_east * east).normalize();

        let terrain_normal = map.terrain_normal(geo_coords(location));

        approx::assert_relative_eq!(terrain_normal, expected, epsilon = 1.0e-6);
    }

    /// Polar rows ignore longitude differences when deriving their cached normals.
    #[test]
    fn polar_rows_zero_the_eastward_slope() {
        let flat_map = elevation_map(4, 3, vec![0.0; 12]);
        let mut varied_samples = vec![0.0; 12];
        varied_samples[1] = -10.0;
        varied_samples[3] = 10.0;
        let varied_map = elevation_map(4, 3, varied_samples);

        let flat_normal = flat_map.terrain_normal(geo_coords(Vec3::Y));
        let varied_normal = varied_map.terrain_normal(geo_coords(Vec3::Y));

        approx::assert_relative_eq!(varied_normal, flat_normal, epsilon = 1.0e-6);
    }

    /// Polar-row texels use a one-sided latitude difference in their local meridian.
    #[test]
    fn north_polar_row_uses_a_one_sided_latitude_difference() {
        let mut samples = vec![0.0; 12];
        samples[0] = 1.0;
        samples[1] = 1.0;
        samples[2] = 1.0;
        samples[3] = 1.0;
        let map = elevation_map(4, 3, samples);
        let latitude = 60.0_f32.to_radians();
        let longitude = -135.0_f32.to_radians();
        let location = Vec3::new(
            longitude.sin() * latitude.cos(),
            latitude.sin(),
            -longitude.cos() * latitude.cos(),
        );
        let east = Vec3::new(longitude.cos(), 0.0, longitude.sin());
        let north = east.cross(location);
        let slope_north = 3.0 / (std::f32::consts::PI * 1737.4);
        let expected = (location - slope_north * north).normalize();

        let terrain_normal = map.terrain_normal(geo_coords(Vec3::Y));

        approx::assert_relative_eq!(terrain_normal, expected, epsilon = 1.0e-6);
    }

    /// Construction preserves source dimensions and creates one finite unit normal per texel.
    #[test]
    fn cache_has_source_dimensions_and_finite_unit_normals() {
        let map = elevation_map(3, 2, vec![0.0, 1.0, 2.0, 2.0, 1.0, 0.0]);

        let dimensions = (map.width(), map.height());

        assert_eq!(dimensions, (3, 2));
        assert_eq!(map.terrain_normals.len(), 6);
        assert!(map.terrain_normals.iter().all(|normal| normal.is_finite()));
        assert!(
            map.terrain_normals
                .iter()
                .all(|normal| (normal.length() - 1.0).abs() < 1.0e-6)
        );
    }

    /// Constructing the same elevation map twice produces the same terrain-normal cache.
    #[test]
    fn construction_is_deterministic() {
        let samples = vec![0.0, 1.0, 2.0, 2.0, 1.0, 0.0];
        let first = elevation_map(3, 2, samples.clone());

        let second = elevation_map(3, 2, samples);

        assert_eq!(first, second);
    }

    /// Nearby globe locations in one source texel select exactly the same cached normal.
    #[test]
    fn nearest_texel_selection_returns_one_cached_normal() {
        let map = elevation_map(4, 3, vec![0.0; 12]);

        let first = map.terrain_normal(geo_coords(Vec3::new(0.01, 0.01, -1.0)));
        let second = map.terrain_normal(geo_coords(Vec3::new(0.2, 0.2, -1.0)));

        assert_eq!(first, second);
    }
}
