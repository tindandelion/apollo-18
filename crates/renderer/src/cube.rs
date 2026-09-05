use crate::color::{LinearRgb, Srgb8};
use crate::framebuffer::{Framebuffer, RenderError};
use crate::rasterizer::{FragmentShader, NdcVertex, Rasterizer};

use glam::{Mat4, Vec3};

type CubeNdcVertex = NdcVertex<LinearRgb>;

struct VertexColorShader;

impl FragmentShader for VertexColorShader {
    type Attribute = LinearRgb;

    fn shade(&self, colors: [Self::Attribute; 3], barycentric_weights: [f32; 3]) -> LinearRgb {
        colors[0] * barycentric_weights[0]
            + colors[1] * barycentric_weights[1]
            + colors[2] * barycentric_weights[2]
    }
}

const HALF_EXTENT: f32 = 0.5;
const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 0.0, -3.0);
const CANONICAL_PITCH: f32 = -20.0_f32.to_radians();

pub(crate) fn render_at_yaw(
    width: u32,
    height: u32,
    background: Srgb8,
    yaw_radians: f32,
) -> Result<Framebuffer, RenderError> {
    let mut rasterizer = Rasterizer::new(width, height, background)?;
    let object_to_ndc = projection_transform()
        * view_transform(CAMERA_POSITION)
        * object_transform(yaw_radians, CANONICAL_PITCH);

    for face in cube_faces() {
        let vertices = face.corners.map(|position| {
            CubeNdcVertex::new(
                object_to_ndc.transform_point3(position),
                face.color.to_linear(),
            )
            .expect("canonical cube vertex should be inside the view volume")
        });
        rasterizer.draw_triangle([vertices[0], vertices[1], vertices[2]], &VertexColorShader);
        rasterizer.draw_triangle([vertices[0], vertices[2], vertices[3]], &VertexColorShader);
    }

    Ok(rasterizer.into_framebuffer())
}

#[derive(Debug, Clone, Copy)]
struct CubeFace {
    corners: [Vec3; 4],
    color: Srgb8,
}

fn cube_faces() -> [CubeFace; 6] {
    let low = -HALF_EXTENT;
    let high = HALF_EXTENT;

    [
        CubeFace {
            corners: [
                Vec3::new(low, low, low),
                Vec3::new(low, high, low),
                Vec3::new(high, high, low),
                Vec3::new(high, low, low),
            ],
            color: Srgb8::RED,
        },
        CubeFace {
            corners: [
                Vec3::new(high, low, high),
                Vec3::new(high, high, high),
                Vec3::new(low, high, high),
                Vec3::new(low, low, high),
            ],
            color: Srgb8::from_hex(0x00_ff_ff),
        },
        CubeFace {
            corners: [
                Vec3::new(high, low, low),
                Vec3::new(high, high, low),
                Vec3::new(high, high, high),
                Vec3::new(high, low, high),
            ],
            color: Srgb8::GREEN,
        },
        CubeFace {
            corners: [
                Vec3::new(low, low, high),
                Vec3::new(low, high, high),
                Vec3::new(low, high, low),
                Vec3::new(low, low, low),
            ],
            color: Srgb8::from_hex(0xff_00_ff),
        },
        CubeFace {
            corners: [
                Vec3::new(low, high, low),
                Vec3::new(low, high, high),
                Vec3::new(high, high, high),
                Vec3::new(high, high, low),
            ],
            color: Srgb8::BLUE,
        },
        CubeFace {
            corners: [
                Vec3::new(low, low, high),
                Vec3::new(low, low, low),
                Vec3::new(high, low, low),
                Vec3::new(high, low, high),
            ],
            color: Srgb8::from_hex(0xff_ff_00),
        },
    ]
}

fn object_transform(yaw_radians: f32, pitch_radians: f32) -> Mat4 {
    Mat4::from_rotation_x(pitch_radians) * Mat4::from_rotation_y(yaw_radians)
}

fn view_transform(camera_position: glam::Vec3) -> Mat4 {
    Mat4::from_translation(-camera_position)
}

fn projection_transform() -> Mat4 {
    Mat4::orthographic_lh(-1.25, 1.25, -1.25, 1.25, 2.0, 4.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn cube_faces_are_wound_clockwise_when_seen_from_outside() {
        let outward_normals = [
            Vec3::NEG_Z,
            Vec3::Z,
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
        ];

        for (face, expected_normal) in cube_faces().into_iter().zip(outward_normals) {
            let [first, second, third, _] = face.corners;
            let normal = (second - first).cross(third - first).normalize();
            assert_eq!(normal, expected_normal);
        }
    }

    #[test]
    fn object_transform_applies_yaw_before_pitch() {
        let transform = object_transform(90.0_f32.to_radians(), 90.0_f32.to_radians());
        let transformed = transform.transform_point3(Vec3::Z);

        approx::assert_relative_eq!(transformed, Vec3::X, epsilon = 1.0e-6);
    }

    #[test]
    fn view_transform_places_the_world_in_front_of_the_camera() {
        let transform = view_transform(Vec3::new(0.0, 0.0, -3.0));

        assert_eq!(
            transform.transform_point3(Vec3::ZERO),
            Vec3::new(0.0, 0.0, 3.0)
        );
    }

    #[test]
    fn orthographic_projection_maps_bounds_to_ndc() {
        let projection = projection_transform();

        assert_eq!(
            projection.transform_point3(Vec3::new(-1.25, 1.25, 2.0)),
            Vec3::new(-1.0, 1.0, 0.0)
        );
        assert_eq!(
            projection.transform_point3(Vec3::new(1.25, -1.25, 4.0)),
            Vec3::new(1.0, -1.0, 1.0)
        );
    }

    #[test]
    fn mirrored_views_show_expected_faces_and_hide_opposites() {
        let background = Srgb8::from_hex(0x18_18_18);
        let positive_yaw = render_at_yaw(96, 96, background, 30.0_f32.to_radians())
            .expect("positive-yaw cube should render");
        let negative_yaw = render_at_yaw(96, 96, background, -30.0_f32.to_radians())
            .expect("negative-yaw cube should render");

        assert_faces(
            &positive_yaw,
            [0xff_00_00, 0x00_ff_00, 0x00_00_ff],
            [0x00_ff_ff, 0xff_00_ff, 0xff_ff_00],
        );
        assert_faces(
            &negative_yaw,
            [0xff_00_00, 0xff_00_ff, 0x00_00_ff],
            [0x00_ff_ff, 0x00_ff_00, 0xff_ff_00],
        );
    }

    fn assert_faces(frame: &Framebuffer, visible: [u32; 3], hidden: [u32; 3]) {
        for color in visible {
            assert!(frame_contains(frame, Srgb8::from_hex(color)));
        }
        for color in hidden {
            assert!(!frame_contains(frame, Srgb8::from_hex(color)));
        }
    }

    fn frame_contains(frame: &Framebuffer, color: Srgb8) -> bool {
        let [red, green, blue] = color.channels();
        frame
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel == [red, green, blue, 0xff])
    }
}
