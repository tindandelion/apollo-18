use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub(crate) struct NdcVertex<T> {
    position: Vec3,
    attribute: T,
}

impl<T> NdcVertex<T> {
    pub(crate) fn new(position: Vec3, attribute: T) -> Option<Self> {
        if Self::position_is_valid(position) {
            Some(Self {
                position,
                attribute,
            })
        } else {
            None
        }
    }

    fn position_is_valid(position: Vec3) -> bool {
        position.is_finite()
            && (-1.0..=1.0).contains(&position.x)
            && (-1.0..=1.0).contains(&position.y)
            && (0.0..=1.0).contains(&position.z)
    }

    pub(super) fn into_screen(self, width: u32, height: u32) -> ScreenVertex<T> {
        ScreenVertex {
            position: ndc_to_screen(self.position, width, height),
            depth: self.position.z,
            attribute: self.attribute,
        }
    }
}

fn ndc_to_screen(position: Vec3, width: u32, height: u32) -> Vec2 {
    Vec2::new(
        (position.x + 1.0) * width as f32 / 2.0,
        (1.0 - position.y) * height as f32 / 2.0,
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ScreenVertex<T> {
    pub(super) position: Vec2,
    pub(super) depth: f32,
    pub(super) attribute: T,
}

impl<T> ScreenVertex<T> {
    #[cfg(test)]
    pub(super) const fn new(position: Vec2, depth: f32, attribute: T) -> Self {
        Self {
            position,
            depth,
            attribute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Srgb8;

    #[test]
    fn ndc_vertex_rejects_positions_outside_the_view_volume() {
        let color = Srgb8::RED.to_linear();
        assert!(NdcVertex::new(Vec3::new(-1.0, 1.0, 0.0), color).is_some());
        assert!(NdcVertex::new(Vec3::new(1.0, -1.0, 1.0), color).is_some());

        for invalid in [
            (f32::NAN, 0.0, 0.5),
            (0.0, f32::INFINITY, 0.5),
            (0.0, 0.0, f32::NEG_INFINITY),
            (f32::MAX, 0.0, 0.5),
            (f32::MIN, 0.0, 0.5),
            (-1.001, 0.0, 0.5),
            (1.001, 0.0, 0.5),
            (0.0, -1.001, 0.5),
            (0.0, 1.001, 0.5),
            (0.0, 0.0, -0.001),
            (0.0, 0.0, 1.001),
        ] {
            assert!(NdcVertex::new(Vec3::new(invalid.0, invalid.1, invalid.2), color).is_none());
        }
    }

    #[test]
    fn viewport_conversion_maps_ndc_corners_and_inverts_y() {
        let color = Srgb8::RED.to_linear();
        let top_left = NdcVertex::new(Vec3::new(-1.0, 1.0, 0.0), color)
            .expect("valid NDC vertex")
            .into_screen(800, 600);
        let bottom_right = NdcVertex::new(Vec3::new(1.0, -1.0, 1.0), color)
            .expect("valid NDC vertex")
            .into_screen(800, 600);

        assert_eq!(top_left.position, Vec2::new(0.0, 0.0));
        assert_eq!(top_left.depth, 0.0);
        assert_eq!(bottom_right.position, Vec2::new(800.0, 600.0));
        assert_eq!(bottom_right.depth, 1.0);
    }
}
