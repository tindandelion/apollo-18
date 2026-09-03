use crate::color::LinearRgb;
use crate::image::SrgbImage;
use glam::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct LunarColorMap {
    width: u32,
    height: u32,
    pixels: Vec<LinearRgb>,
}

impl LunarColorMap {
    pub fn new(image: SrgbImage) -> Self {
        let width = image.width();
        let height = image.height();
        let mut pixels = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height {
            for x in 0..width {
                pixels.push(image.pixel(x, y).to_linear());
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn sample_linear(&self, radial_direction: Vec3) -> LinearRgb {
        let direction = radial_direction.normalize();
        let longitude = direction.x.atan2(-direction.z);
        let latitude = direction.y.asin();
        let horizontal = (0.5 + longitude / std::f32::consts::TAU).rem_euclid(1.0);
        let vertical = (0.5 - latitude / std::f32::consts::PI).clamp(0.0, 1.0);
        let x = ((horizontal * self.width() as f32).floor() as u32) % self.width();
        let y = (vertical * self.height() as f32).floor() as u32;
        let y = y.min(self.height() - 1);

        self.pixels[y as usize * self.width as usize + x as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Srgb8;

    fn coordinate_map() -> LunarColorMap {
        let pixels = [
            [0x10, 0x00, 0x00],
            [0x20, 0x00, 0x00],
            [0x30, 0x00, 0x00],
            [0x40, 0x00, 0x00],
            [0x50, 0x00, 0x00],
            [0x60, 0x00, 0x00],
            [0x70, 0x00, 0x00],
            [0x80, 0x00, 0x00],
            [0x90, 0x00, 0x00],
            [0xa0, 0x00, 0x00],
            [0xb0, 0x00, 0x00],
            [0xc0, 0x00, 0x00],
        ]
        .concat();
        LunarColorMap::new(SrgbImage::new(4, 3, pixels).expect("valid synthetic map"))
    }

    fn sampled_srgb(map: &LunarColorMap, direction: Vec3) -> [u8; 3] {
        map.sample_linear(direction).to_srgb8().channels()
    }

    #[test]
    fn zero_longitude_samples_the_horizontal_center() {
        let map = coordinate_map();

        assert_eq!(sampled_srgb(&map, Vec3::NEG_Z), [0x70, 0, 0]);
    }

    #[test]
    fn longitude_wraps_at_the_antimeridian() {
        let map = coordinate_map();
        let beside_negative_x = Vec3::new(-0.000_001, 0.0, 1.0);
        let beside_positive_x = Vec3::new(0.000_001, 0.0, 1.0);

        assert_eq!(sampled_srgb(&map, Vec3::Z), [0x50, 0, 0]);
        assert_eq!(sampled_srgb(&map, beside_negative_x), [0x50, 0, 0]);
        assert_eq!(sampled_srgb(&map, beside_positive_x), [0x80, 0, 0]);
    }

    #[test]
    fn latitude_clamps_to_polar_rows() {
        let map = coordinate_map();

        assert_eq!(sampled_srgb(&map, Vec3::Y), [0x10, 0, 0]);
        assert_eq!(sampled_srgb(&map, Vec3::NEG_Y), [0x90, 0, 0]);
    }

    #[test]
    fn nearest_neighbor_sampling_selects_one_source_pixel() {
        let map = coordinate_map();
        let direction = Vec3::new(0.1, 0.1, -1.0);

        assert_eq!(sampled_srgb(&map, direction), [0x70, 0, 0]);
    }

    #[test]
    fn sampled_srgb_round_trips_through_linear_light() {
        let image = SrgbImage::new(1, 1, vec![12, 128, 241]).expect("valid synthetic map");
        let map = LunarColorMap::new(image);

        assert_eq!(
            map.sample_linear(Vec3::NEG_Z).to_srgb8(),
            Srgb8::from_channels([12, 128, 241])
        );
    }
}
