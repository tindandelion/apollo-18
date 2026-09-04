use crate::color::LinearRgb;
use crate::globe_location::GeoCoords;
use crate::image::SrgbImage;

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

    pub(crate) fn sample_linear(&self, geo_coords: GeoCoords) -> LinearRgb {
        let (x, y) = geo_coords.nearest_texel(self.width(), self.height());

        self.pixels[y as usize * self.width as usize + x as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Srgb8;
    use crate::globe_location::GlobeLocation;
    use glam::Vec3;

    fn geo_coords(direction: Vec3) -> GeoCoords {
        GlobeLocation::new(direction)
            .expect("test direction should be finite and nonzero")
            .geo_coords()
    }

    /// Sampled sRGB bytes round-trip through linear light unchanged.
    #[test]
    fn sampled_srgb_round_trips_through_linear_light() {
        let image = SrgbImage::new(1, 1, vec![12, 128, 241]).expect("valid synthetic map");
        let map = LunarColorMap::new(image);

        let sampled = map.sample_linear(geo_coords(Vec3::NEG_Z)).to_srgb8();

        assert_eq!(sampled, Srgb8::from_channels([12, 128, 241]));
    }
}
