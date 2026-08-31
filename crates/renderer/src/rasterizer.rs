use crate::color::LinearRgb;
use crate::framebuffer::Framebuffer;
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
pub(crate) struct NdcVertex {
    position: Vec3,
    color: LinearRgb,
}

impl NdcVertex {
    pub(crate) fn new(position: Vec3, color: LinearRgb) -> Option<Self> {
        if position.is_finite()
            && (-1.0..=1.0).contains(&position.x)
            && (-1.0..=1.0).contains(&position.y)
            && (0.0..=1.0).contains(&position.z)
        {
            Some(Self { position, color })
        } else {
            None
        }
    }

    fn to_screen(self, width: u32, height: u32) -> ScreenVertex {
        ScreenVertex {
            position: Vec2::new(
                (self.position.x + 1.0) * width as f32 / 2.0,
                (1.0 - self.position.y) * height as f32 / 2.0,
            ),
            depth: self.position.z,
            color: self.color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScreenVertex {
    position: Vec2,
    depth: f32,
    color: LinearRgb,
}

impl ScreenVertex {
    #[cfg(test)]
    const fn new(position: Vec2, depth: f32, color: LinearRgb) -> Self {
        Self {
            position,
            depth,
            color,
        }
    }
}

pub(crate) struct Rasterizer {
    framebuffer: Framebuffer,
    depth_buffer: Vec<f32>,
}

impl Rasterizer {
    pub(crate) fn new(
        width: u32,
        height: u32,
        background: crate::color::Srgb8,
    ) -> Result<Self, crate::framebuffer::RenderError> {
        let framebuffer = Framebuffer::new(width, height, background)?;
        let depth_buffer = vec![f32::INFINITY; framebuffer.pixels().len() / 4];
        Ok(Self {
            framebuffer,
            depth_buffer,
        })
    }

    pub(crate) fn draw_triangle(&mut self, vertices: [NdcVertex; 3]) {
        let mut screen_vertices = vertices
            .map(|vertex| vertex.to_screen(self.framebuffer.width(), self.framebuffer.height()));
        let screen_area = edge(
            screen_vertices[0].position,
            screen_vertices[1].position,
            screen_vertices[2].position,
        );
        if screen_area >= 0.0 {
            return;
        }

        screen_vertices.swap(1, 2);
        self.fill_triangle(screen_vertices, -screen_area);
    }

    pub(crate) fn into_framebuffer(self) -> Framebuffer {
        self.framebuffer
    }

    fn fill_triangle(&mut self, vertices: [ScreenVertex; 3], area: f32) {
        let min_x = vertices
            .iter()
            .copied()
            .map(|vertex| vertex.position.x)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_x = vertices
            .iter()
            .copied()
            .map(|vertex| vertex.position.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.framebuffer.width() as f32) as u32;
        let min_y = vertices
            .iter()
            .copied()
            .map(|vertex| vertex.position.y)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_y = vertices
            .iter()
            .copied()
            .map(|vertex| vertex.position.y)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.framebuffer.height() as f32) as u32;

        let edges = [
            Edge::new(vertices[1].position, vertices[2].position),
            Edge::new(vertices[2].position, vertices[0].position),
            Edge::new(vertices[0].position, vertices[1].position),
        ];
        let colors = [vertices[0].color, vertices[1].color, vertices[2].color];
        let depths = [vertices[0].depth, vertices[1].depth, vertices[2].depth];

        for y in min_y..max_y {
            for x in min_x..max_x {
                let sample = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let values = edge_values(&edges, sample);
                if covers_sample(&edges, values) {
                    let weights = barycentric_weights(values, area);
                    let depth = interpolate(depths, weights);
                    let index = y as usize * self.framebuffer.width() as usize + x as usize;
                    if depth < self.depth_buffer[index] {
                        let color = LinearRgb::interpolate(colors, weights);
                        self.framebuffer.set_pixel(x, y, color.to_srgb8());
                        self.depth_buffer[index] = depth;
                    }
                }
            }
        }
    }
}

fn edge_values(edges: &[Edge; 3], point: Vec2) -> [f32; 3] {
    [
        edges[0].value(point),
        edges[1].value(point),
        edges[2].value(point),
    ]
}

fn covers_sample(edges: &[Edge; 3], values: [f32; 3]) -> bool {
    (0..3).all(|index| edges[index].contains(values[index]))
}

fn barycentric_weights(edge_values: [f32; 3], area: f32) -> [f32; 3] {
    edge_values.map(|value| value / area)
}

fn interpolate(values: [f32; 3], weights: [f32; 3]) -> f32 {
    values[0] * weights[0] + values[1] * weights[1] + values[2] * weights[2]
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    start: Vec2,
    end: Vec2,
    includes_on_edge_samples: bool,
}

impl Edge {
    fn new(start: Vec2, end: Vec2) -> Self {
        let direction = end - start;
        Self {
            start,
            end,
            includes_on_edge_samples: direction.y < 0.0
                || (direction.y == 0.0 && direction.x > 0.0),
        }
    }

    fn value(self, point: Vec2) -> f32 {
        edge(self.start, self.end, point)
    }

    fn contains(self, value: f32) -> bool {
        value > 0.0 || (value == 0.0 && self.includes_on_edge_samples)
    }
}

fn edge(start: Vec2, end: Vec2, point: Vec2) -> f32 {
    (end - start).perp_dot(point - start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Srgb8;

    const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);

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
            .to_screen(800, 600);
        let bottom_right = NdcVertex::new(Vec3::new(1.0, -1.0, 1.0), color)
            .expect("valid NDC vertex")
            .to_screen(800, 600);

        assert_eq!(top_left.position, Vec2::new(0.0, 0.0));
        assert_eq!(top_left.depth, 0.0);
        assert_eq!(bottom_right.position, Vec2::new(800.0, 600.0));
        assert_eq!(bottom_right.depth, 1.0);
    }

    #[test]
    fn screen_space_area_culls_back_faces_and_degenerate_triangles() {
        let front_facing = ndc_triangle(0.5, Srgb8::RED);
        let clockwise = [front_facing[0], front_facing[2], front_facing[1]];
        let degenerate = [front_facing[0], front_facing[0], front_facing[1]];
        let precision_collapsed = [
            ndc_vertex(0.0, -0.8, 0.5, Srgb8::RED),
            ndc_vertex(f32::from_bits(1), -0.8, 0.5, Srgb8::RED),
            ndc_vertex(0.0, 0.8, 0.5, Srgb8::RED),
        ];

        let mut visible = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        visible.draw_triangle(front_facing);
        assert_ne!(visible.into_framebuffer(), frame());

        let mut culled = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        culled.draw_triangle(clockwise);
        culled.draw_triangle(degenerate);
        culled.draw_triangle(precision_collapsed);
        assert_eq!(culled.into_framebuffer(), frame());
    }

    #[test]
    fn nearer_triangle_wins_independent_of_submission_order() {
        let near = ndc_triangle(0.2, Srgb8::RED);
        let far = ndc_triangle(0.8, Srgb8::BLUE);

        let mut near_then_far = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        near_then_far.draw_triangle(near);
        near_then_far.draw_triangle(far);

        let mut far_then_near = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        far_then_near.draw_triangle(far);
        far_then_near.draw_triangle(near);

        let near_then_far = near_then_far.into_framebuffer();
        let far_then_near = far_then_near.into_framebuffer();
        assert_eq!(near_then_far, far_then_near);
        assert_eq!(pixel(&near_then_far, 2, 2), opaque(Srgb8::RED));
    }

    #[test]
    fn depth_is_affinely_interpolated_from_screen_barycentrics() {
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        fill_screen_triangle(
            &mut rasterizer,
            [
                ScreenVertex::new(Vec2::new(1.0, 1.0), 0.0, Srgb8::RED.to_linear()),
                ScreenVertex::new(Vec2::new(5.0, 1.0), 0.8, Srgb8::RED.to_linear()),
                ScreenVertex::new(Vec2::new(1.0, 5.0), 0.4, Srgb8::RED.to_linear()),
            ],
        );

        let depth = rasterizer.depth_buffer[7];
        assert!((depth - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn strict_depth_test_keeps_first_fragment_on_an_exact_tie() {
        let first = ndc_triangle(0.5, Srgb8::RED);
        let tied = ndc_triangle(0.5, Srgb8::BLUE);
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");

        rasterizer.draw_triangle(first);
        let framebuffer_before = rasterizer.framebuffer.clone();
        let depth_before = rasterizer.depth_buffer.clone();
        rasterizer.draw_triangle(tied);

        assert_eq!(rasterizer.framebuffer, framebuffer_before);
        assert_eq!(rasterizer.depth_buffer, depth_before);
    }

    #[test]
    fn complete_normalized_depth_range_is_accepted() {
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        rasterizer.draw_triangle(ndc_triangle(1.0, Srgb8::BLUE));
        assert_eq!(pixel(&rasterizer.framebuffer, 2, 2), opaque(Srgb8::BLUE));

        rasterizer.draw_triangle(ndc_triangle(0.0, Srgb8::RED));
        assert_eq!(pixel(&rasterizer.framebuffer, 2, 2), opaque(Srgb8::RED));
        assert_eq!(rasterizer.depth_buffer[2 * 6 + 2], 0.0);
    }

    #[test]
    fn edge_values_normalize_to_barycentric_weights() {
        let vertices = [
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, 2.0),
        ];
        let area = edge(vertices[0], vertices[1], vertices[2]);
        let edges = [
            Edge::new(vertices[1], vertices[2]),
            Edge::new(vertices[2], vertices[0]),
            Edge::new(vertices[0], vertices[1]),
        ];
        let values = edge_values(&edges, Vec2::new(0.5, 0.5));
        let weights = barycentric_weights(values, area);

        assert_eq!(values, [2.0, 1.0, 1.0]);
        assert_eq!(weights, [0.5, 0.25, 0.25]);
        assert_eq!(weights.iter().sum::<f32>(), 1.0);
    }

    #[test]
    fn screen_traversal_is_bounded_to_the_framebuffer() {
        let fully_offscreen = [
            vertex(1_000_000.0, 1_000_000.0, Srgb8::RED),
            vertex(1_000_004.0, 1_000_000.0, Srgb8::GREEN),
            vertex(1_000_000.0, 1_000_004.0, Srgb8::BLUE),
        ];
        assert_eq!(render_triangles(&[fully_offscreen]), frame());

        let partially_offscreen = [
            vertex(-2.0, -2.0, Srgb8::RED),
            vertex(4.0, 0.0, Srgb8::GREEN),
            vertex(0.0, 4.0, Srgb8::BLUE),
        ];
        let partial_frame = render_triangles(&[partially_offscreen]);
        assert!(
            partial_frame
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel != opaque(BACKGROUND))
        );
    }

    #[test]
    fn top_left_rule_gives_shared_edge_to_one_triangle() {
        let red = Srgb8::RED;
        let green = Srgb8::GREEN;
        let top_left = vertex(1.0, 1.0, red);
        let top_right = vertex(5.0, 1.0, red);
        let bottom_right = vertex(5.0, 5.0, red);
        let bottom_left = vertex(1.0, 5.0, green);
        let first = [top_left, top_right, bottom_right];
        let second = [
            vertex(1.0, 1.0, green),
            vertex(5.0, 5.0, green),
            bottom_left,
        ];

        let forward = render_triangles(&[first, second]);
        let reverse = render_triangles(&[second, first]);
        assert_eq!(forward, reverse);

        for coordinate in 1..5 {
            assert_ne!(pixel(&forward, coordinate, coordinate), opaque(BACKGROUND));
            assert_eq!(pixel(&forward, coordinate, coordinate), opaque(red));
        }
        for y in 1..5 {
            for x in 1..5 {
                assert_ne!(pixel(&forward, x, y), opaque(BACKGROUND));
            }
        }
    }

    fn ndc_triangle(depth: f32, color: Srgb8) -> [NdcVertex; 3] {
        [
            ndc_vertex(-0.8, -0.8, depth, color),
            ndc_vertex(0.8, -0.8, depth, color),
            ndc_vertex(0.0, 0.8, depth, color),
        ]
    }

    fn ndc_vertex(x: f32, y: f32, depth: f32, color: Srgb8) -> NdcVertex {
        NdcVertex::new(Vec3::new(x, y, depth), color.to_linear())
            .expect("test NDC vertex should be valid")
    }

    fn vertex(x: f32, y: f32, color: Srgb8) -> ScreenVertex {
        ScreenVertex::new(Vec2::new(x, y), 0.5, color.to_linear())
    }

    fn frame() -> Framebuffer {
        Framebuffer::new(6, 6, BACKGROUND).expect("test frame should be valid")
    }

    fn render_triangles(triangles: &[[ScreenVertex; 3]]) -> Framebuffer {
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        for triangle in triangles {
            fill_screen_triangle(&mut rasterizer, *triangle);
        }
        rasterizer.into_framebuffer()
    }

    fn fill_screen_triangle(rasterizer: &mut Rasterizer, vertices: [ScreenVertex; 3]) {
        let area = edge(
            vertices[0].position,
            vertices[1].position,
            vertices[2].position,
        );
        assert!(
            area > 0.0,
            "screen-space test triangle should be counter-clockwise"
        );
        rasterizer.fill_triangle(vertices, area);
    }

    fn opaque(color: Srgb8) -> [u8; 4] {
        let [red, green, blue] = color.channels();
        [red, green, blue, 0xff]
    }

    fn pixel(frame: &Framebuffer, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * frame.width() + x) * 4) as usize;
        frame.pixels()[offset..offset + 4]
            .try_into()
            .expect("pixel should contain four channels")
    }
}
