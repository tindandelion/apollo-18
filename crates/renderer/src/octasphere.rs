use crate::color::Srgb8;
use crate::framebuffer::{Framebuffer, RenderError};
use crate::rasterizer::{NdcVertex, Rasterizer};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

const CANONICAL_SUBDIVISION_LEVEL: u32 = 5;
const GLOBE_RADIUS: f32 = 0.5;
const FRAME_OCCUPANCY: f32 = 0.9;
const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 0.0, -3.0);
const LUNAR_NORTH: Vec3 = Vec3::Y;
const ZERO_DEGREE_LONGITUDE: Vec3 = Vec3::NEG_Z;

pub(crate) fn render(
    width: u32,
    height: u32,
    background: Srgb8,
    yaw_radians: f32,
) -> Result<Framebuffer, RenderError> {
    let mut rasterizer = Rasterizer::new(width, height, background)?;
    let object_to_ndc = projection_transform(width, height)
        * Mat4::from_translation(-CAMERA_POSITION)
        * Mat4::from_rotation_y(yaw_radians)
        * Mat4::from_scale(Vec3::splat(GLOBE_RADIUS));
    let mesh = generate(CANONICAL_SUBDIVISION_LEVEL);

    for triangle in mesh.triangles {
        let vertices = triangle.map(|index| {
            let radial_direction = mesh.positions[index as usize];
            NdcVertex::new(
                object_to_ndc.transform_point3(radial_direction),
                color_for(radial_direction).to_linear(),
            )
            .expect("canonical octasphere vertex should be inside the view volume")
        });
        rasterizer.draw_triangle(vertices);
    }

    Ok(rasterizer.into_framebuffer())
}

fn color_for(radial_direction: Vec3) -> Srgb8 {
    let normalized_srgb = Vec3::new(
        (radial_direction.x + 1.0) * 0.5,
        (radial_direction.y + 1.0) * 0.5,
        (1.0 - radial_direction.z) * 0.5,
    );
    Srgb8::from_channels(
        normalized_srgb
            .to_array()
            .map(|channel| (channel * 255.0).round() as u8),
    )
}

fn projection_transform(width: u32, height: u32) -> Mat4 {
    let shortest_side = width.min(height) as f32;
    let half_width = GLOBE_RADIUS * width as f32 / (FRAME_OCCUPANCY * shortest_side);
    let half_height = GLOBE_RADIUS * height as f32 / (FRAME_OCCUPANCY * shortest_side);

    Mat4::orthographic_lh(-half_width, half_width, -half_height, half_height, 2.0, 4.0)
}

#[derive(Debug, Clone, PartialEq)]
struct Octasphere {
    positions: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
}

fn generate(subdivision_level: u32) -> Octasphere {
    let mut mesh = base_octahedron();

    for _ in 0..subdivision_level {
        mesh = subdivide(mesh);
    }

    mesh
}

fn base_octahedron() -> Octasphere {
    let positions = vec![
        LUNAR_NORTH,
        Vec3::NEG_Y,
        ZERO_DEGREE_LONGITUDE,
        Vec3::X,
        Vec3::Z,
        Vec3::NEG_X,
    ];
    let triangles = vec![
        [0, 3, 2],
        [0, 4, 3],
        [0, 5, 4],
        [0, 2, 5],
        [1, 2, 3],
        [1, 3, 4],
        [1, 4, 5],
        [1, 5, 2],
    ];

    Octasphere {
        positions,
        triangles,
    }
}

fn subdivide(mut mesh: Octasphere) -> Octasphere {
    let mut midpoint_indices = HashMap::new();
    let mut triangles = Vec::with_capacity(mesh.triangles.len() * 4);

    for [first, second, third] in mesh.triangles {
        let first_second =
            midpoint_index(&mut mesh.positions, &mut midpoint_indices, first, second);
        let second_third =
            midpoint_index(&mut mesh.positions, &mut midpoint_indices, second, third);
        let third_first = midpoint_index(&mut mesh.positions, &mut midpoint_indices, third, first);

        triangles.extend([
            [first, first_second, third_first],
            [first_second, second, second_third],
            [third_first, second_third, third],
            [first_second, second_third, third_first],
        ]);
    }

    Octasphere {
        positions: mesh.positions,
        triangles,
    }
}

fn midpoint_index(
    positions: &mut Vec<Vec3>,
    midpoint_indices: &mut HashMap<(u32, u32), u32>,
    first: u32,
    second: u32,
) -> u32 {
    let edge = if first < second {
        (first, second)
    } else {
        (second, first)
    };

    *midpoint_indices.entry(edge).or_insert_with(|| {
        let midpoint = (positions[first as usize] + positions[second as usize]).normalize();
        let index = positions.len() as u32;
        positions.push(midpoint);
        index
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn subdivision_has_expected_topology_counts() {
        for level in 0..=5 {
            let mesh = generate(level);
            let expected_triangles = 8 * 4_usize.pow(level);
            let expected_vertices = expected_triangles / 2 + 2;

            assert_eq!(mesh.triangles.len(), expected_triangles);
            assert_eq!(mesh.positions.len(), expected_vertices);
        }
    }

    #[test]
    fn generated_vertices_are_unit_radial_directions() {
        let mesh = generate(5);

        assert!(
            mesh.positions
                .iter()
                .all(|position| (position.length() - 1.0).abs() < 1.0e-6)
        );
    }

    #[test]
    fn generated_triangles_keep_outward_winding() {
        let mesh = generate(5);

        for [first, second, third] in &mesh.triangles {
            let first = mesh.positions[*first as usize];
            let second = mesh.positions[*second as usize];
            let third = mesh.positions[*third as usize];
            let normal = (second - first).cross(third - first);
            let triangle_center = first + second + third;

            assert!(normal.dot(triangle_center) > 0.0);
        }
    }

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(generate(4), generate(4));
    }

    #[test]
    fn every_canonical_vertex_has_a_distinct_programmatic_color() {
        let mesh = generate(CANONICAL_SUBDIVISION_LEVEL);
        let colors = mesh
            .positions
            .iter()
            .copied()
            .map(color_for)
            .map(Srgb8::channels)
            .collect::<HashSet<_>>();

        assert_eq!(colors.len(), mesh.positions.len());
        assert_eq!(color_for(Vec3::NEG_Z).channels(), [128, 128, 255]);
        assert_eq!(color_for(Vec3::Y).channels(), [128, 255, 128]);
    }

    #[test]
    fn canonical_axes_define_lunar_orientation() {
        let mesh = generate(0);

        assert_eq!(LUNAR_NORTH, Vec3::Y);
        assert_eq!(ZERO_DEGREE_LONGITUDE, Vec3::NEG_Z);
        assert!(mesh.positions.contains(&LUNAR_NORTH));
        assert!(mesh.positions.contains(&ZERO_DEGREE_LONGITUDE));
    }

    #[test]
    fn projection_preserves_circle_and_uses_ninety_percent_of_short_side() {
        for (width, height) in [(800, 800), (1200, 600), (600, 1200)] {
            let projection = projection_transform(width, height);
            let left = projection.transform_point3(Vec3::new(-GLOBE_RADIUS, 0.0, 3.0));
            let right = projection.transform_point3(Vec3::new(GLOBE_RADIUS, 0.0, 3.0));
            let top = projection.transform_point3(Vec3::new(0.0, GLOBE_RADIUS, 3.0));
            let bottom = projection.transform_point3(Vec3::new(0.0, -GLOBE_RADIUS, 3.0));
            let pixel_width = (right.x - left.x) * width as f32 / 2.0;
            let pixel_height = (top.y - bottom.y) * height as f32 / 2.0;
            let expected = width.min(height) as f32 * FRAME_OCCUPANCY;

            assert!((pixel_width - expected).abs() < 1.0e-4);
            assert!((pixel_height - expected).abs() < 1.0e-4);
        }
    }
}
