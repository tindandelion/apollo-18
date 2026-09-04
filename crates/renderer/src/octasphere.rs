use crate::color::{LinearRgb, Srgb8};
use crate::framebuffer::{Framebuffer, RenderError};
use crate::globe_location::GlobeLocation;
use crate::lunar_color_map::LunarColorMap;
use crate::lunar_elevation_map::LunarElevationMap;
use crate::rasterizer::{FragmentShader, NdcVertex, Rasterizer};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SunDirection(Vec3);

impl SunDirection {
    pub(crate) fn new(direction: Vec3) -> Option<Self> {
        if direction.is_finite() && direction.length_squared() > 0.0 {
            Some(Self(direction.normalize()))
        } else {
            None
        }
    }

    fn diffuse_intensity(self, lighting_normal: Vec3) -> f32 {
        lighting_normal.dot(self.0).max(0.0)
    }
}

type SphereNdcVertex = NdcVertex<GlobeLocation>;

struct LunarShader<'a> {
    color_map: &'a LunarColorMap,
    elevation_map: &'a LunarElevationMap,
    object_rotation: Mat4,
    sun_direction: SunDirection,
}

impl<'a> LunarShader<'a> {
    const fn new(
        color_map: &'a LunarColorMap,
        elevation_map: &'a LunarElevationMap,
        object_rotation: Mat4,
        sun_direction: SunDirection,
    ) -> Self {
        Self {
            color_map,
            elevation_map,
            object_rotation,
            sun_direction,
        }
    }
}

impl FragmentShader for LunarShader<'_> {
    type Attribute = GlobeLocation;

    fn shade(&self, attributes: [Self::Attribute; 3], barycentric_weights: [f32; 3]) -> LinearRgb {
        let globe_location = GlobeLocation::interpolate(attributes, barycentric_weights)
            .expect("covered fragments interpolate a nonzero globe location");
        let perturbed_radial = self.elevation_map.perturbed_radial(globe_location);
        let lighting_normal = self
            .object_rotation
            .transform_vector3(perturbed_radial)
            .normalize();
        let diffuse_intensity = self.sun_direction.diffuse_intensity(lighting_normal);

        self.color_map.sample_linear(globe_location) * diffuse_intensity
    }
}

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
    color_map: &LunarColorMap,
    elevation_map: &LunarElevationMap,
    sun_direction: SunDirection,
) -> Result<Framebuffer, RenderError> {
    let mut rasterizer = Rasterizer::new(width, height, background)?;
    let object_rotation = Mat4::from_rotation_y(yaw_radians);
    let object_to_ndc = projection_transform(width, height)
        * Mat4::from_translation(-CAMERA_POSITION)
        * object_rotation
        * Mat4::from_scale(Vec3::splat(GLOBE_RADIUS));
    let mesh = generate(CANONICAL_SUBDIVISION_LEVEL);
    let shader = LunarShader::new(color_map, elevation_map, object_rotation, sun_direction);

    for triangle in mesh.triangles {
        let vertices = triangle.map(|index| {
            let globe_location =
                GlobeLocation::new(mesh.positions[index as usize]).expect("unit globe location");
            SphereNdcVertex::new(
                object_to_ndc.transform_point3(mesh.positions[index as usize]),
                globe_location,
            )
            .expect("canonical octasphere vertex should be inside the view volume")
        });
        rasterizer.draw_triangle(vertices, &shader);
    }

    Ok(rasterizer.into_framebuffer())
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
    use crate::image::{ElevationImage, SrgbImage};

    fn flat_elevation_map() -> LunarElevationMap {
        LunarElevationMap::new(
            ElevationImage::new(1, 1, vec![0.0]).expect("valid synthetic elevation map"),
        )
    }

    fn identity_shader<'a>(
        color_map: &'a LunarColorMap,
        elevation_map: &'a LunarElevationMap,
        sun_direction: Vec3,
    ) -> LunarShader<'a> {
        LunarShader::new(
            color_map,
            elevation_map,
            Mat4::IDENTITY,
            SunDirection::new(sun_direction).expect("valid Sun direction"),
        )
    }

    /// Flat elevation and identity rotation keep Lambert at 1 facing the Sun and 0 facing away.
    #[test]
    fn lambertian_shading_is_full_facing_the_sun_and_zero_facing_away() {
        let color_map = LunarColorMap::new(
            SrgbImage::new(1, 1, vec![255, 255, 255]).expect("valid synthetic map"),
        );
        let elevation_map = flat_elevation_map();
        let shader = identity_shader(&color_map, &elevation_map, Vec3::NEG_Z);

        let facing_sun = GlobeLocation::new(Vec3::NEG_Z).expect("valid globe location");
        let perpendicular = GlobeLocation::new(Vec3::X).expect("valid globe location");
        let facing_away = GlobeLocation::new(Vec3::Z).expect("valid globe location");

        assert_eq!(
            shader
                .shade([facing_sun; 3], [1.0, 0.0, 0.0])
                .to_srgb8()
                .channels(),
            [255, 255, 255]
        );
        for unlit in [perpendicular, facing_away] {
            assert_eq!(
                shader
                    .shade([unlit; 3], [1.0, 0.0, 0.0])
                    .to_srgb8()
                    .channels(),
                [0, 0, 0]
            );
        }
    }

    /// Interpolated globe locations are renormalized so Lambert does not reveal tessellation.
    #[test]
    fn interpolated_globe_location_is_normalized_for_smooth_lighting() {
        let color_map = LunarColorMap::new(
            SrgbImage::new(1, 1, vec![255, 255, 255]).expect("valid synthetic map"),
        );
        let elevation_map = flat_elevation_map();
        let halfway = (Vec3::X + Vec3::NEG_Z).normalize();
        let shader = identity_shader(&color_map, &elevation_map, halfway);
        let attributes = [
            GlobeLocation::new(Vec3::X).expect("valid globe location"),
            GlobeLocation::new(Vec3::NEG_Z).expect("valid globe location"),
            GlobeLocation::new(Vec3::NEG_Z).expect("valid globe location"),
        ];

        assert_eq!(
            shader
                .shade(attributes, [0.5, 0.5, 0.0])
                .to_srgb8()
                .channels(),
            [255, 255, 255]
        );
    }

    /// Different fragments of one triangle sample different longitudes of a ramped color map.
    #[test]
    fn lunar_shader_interpolates_globe_location_per_fragment() {
        let pixels = (0..8)
            .flat_map(|x| [(x * 30) as u8, 0, 0])
            .collect::<Vec<_>>();
        let color_map =
            LunarColorMap::new(SrgbImage::new(8, 1, pixels).expect("valid synthetic map"));
        let elevation_map = flat_elevation_map();
        let vertices = [
            SphereNdcVertex::new(
                Vec3::new(-0.8, -0.8, 0.5),
                GlobeLocation::new(Vec3::NEG_Z).expect("valid globe location"),
            )
            .expect("valid lunar vertex"),
            SphereNdcVertex::new(
                Vec3::new(0.0, 0.8, 0.5),
                GlobeLocation::new(Vec3::X).expect("valid globe location"),
            )
            .expect("valid lunar vertex"),
            SphereNdcVertex::new(
                Vec3::new(0.8, -0.8, 0.5),
                GlobeLocation::new(Vec3::NEG_X).expect("valid globe location"),
            )
            .expect("valid lunar vertex"),
        ];
        let mut rasterizer =
            Rasterizer::new(6, 6, Srgb8::from_hex(0x18_18_18)).expect("valid rasterizer");

        rasterizer.draw_triangle(
            vertices,
            &identity_shader(&color_map, &elevation_map, Vec3::NEG_Z),
        );
        let framebuffer = rasterizer.into_framebuffer();
        let first = pixel(&framebuffer, 2, 2);
        let second = pixel(&framebuffer, 3, 3);

        assert_ne!(first, second);
        assert!(first[0] > 0 && second[0] > 0);
        assert_eq!((first[3], second[3]), (0xff, 0xff));
    }

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

    /// Generated octasphere vertices lie on the unit sphere.
    #[test]
    fn generated_vertices_are_unit_globe_locations() {
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

    fn pixel(frame: &Framebuffer, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * frame.width() + x) * 4) as usize;
        frame.pixels()[offset..offset + 4]
            .try_into()
            .expect("pixel should contain four channels")
    }
}
