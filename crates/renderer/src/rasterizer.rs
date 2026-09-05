mod shader;
mod triangle;
mod vertex;

use crate::color::LinearRgb;
use crate::framebuffer::Framebuffer;
pub(crate) use shader::FragmentShader;
use triangle::rasterize_fragments;
pub(crate) use vertex::NdcVertex;
use vertex::ScreenVertex;

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

    pub(crate) fn draw_triangle<S: FragmentShader>(
        &mut self,
        vertices: [NdcVertex<S::Attribute>; 3],
        shader: &S,
    ) {
        let screen_vertices = vertices
            .map(|vertex| vertex.into_screen(self.framebuffer.width(), self.framebuffer.height()));
        self.rasterize_triangle(screen_vertices, shader);
    }

    fn rasterize_triangle<S: FragmentShader>(
        &mut self,
        vertices: [ScreenVertex<S::Attribute>; 3],
        shader: &S,
    ) {
        let positions = vertices.map(|vertex| vertex.position);
        let depths = vertices.map(|vertex| vertex.depth);
        let attributes = vertices.map(|vertex| vertex.attribute);
        let width = self.framebuffer.width();
        let height = self.framebuffer.height();

        rasterize_fragments(width, height, positions, depths, |x, y, depth, weights| {
            let color = shader.shade(attributes, weights);
            self.write_if_nearer(x, y, depth, color);
        });
    }

    pub(crate) fn into_framebuffer(self) -> Framebuffer {
        self.framebuffer
    }

    fn write_if_nearer(&mut self, x: u32, y: u32, depth: f32, color: LinearRgb) {
        let index = y as usize * self.framebuffer.width() as usize + x as usize;
        if depth < self.depth_buffer[index] {
            self.framebuffer.set_pixel(x, y, color.to_srgb8());
            self.depth_buffer[index] = depth;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Srgb8;
    use glam::{Vec2, Vec3};

    const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);

    type TestNdcVertex = NdcVertex<LinearRgb>;
    type TestScreenVertex = ScreenVertex<LinearRgb>;

    struct TestColorShader;

    impl FragmentShader for TestColorShader {
        type Attribute = LinearRgb;

        fn shade(&self, colors: [Self::Attribute; 3], _weights: [f32; 3]) -> LinearRgb {
            colors[0]
        }
    }

    #[test]
    fn screen_space_area_culls_back_faces_and_degenerate_triangles() {
        let front_facing = ndc_triangle(0.5, Srgb8::RED);
        let counter_clockwise = [front_facing[0], front_facing[2], front_facing[1]];
        let degenerate = [front_facing[0], front_facing[0], front_facing[1]];
        let precision_collapsed = [
            ndc_vertex(0.0, -0.8, 0.5, Srgb8::RED),
            ndc_vertex(0.0, 0.8, 0.5, Srgb8::RED),
            ndc_vertex(f32::from_bits(1), -0.8, 0.5, Srgb8::RED),
        ];

        let mut visible = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        visible.draw_triangle(front_facing, &TestColorShader);
        assert_ne!(visible.into_framebuffer(), frame());

        let mut culled = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        culled.draw_triangle(counter_clockwise, &TestColorShader);
        culled.draw_triangle(degenerate, &TestColorShader);
        culled.draw_triangle(precision_collapsed, &TestColorShader);
        assert_eq!(culled.into_framebuffer(), frame());
    }

    #[test]
    fn nearer_triangle_wins_independent_of_submission_order() {
        let near = ndc_triangle(0.2, Srgb8::RED);
        let far = ndc_triangle(0.8, Srgb8::BLUE);

        let mut near_then_far = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        near_then_far.draw_triangle(near, &TestColorShader);
        near_then_far.draw_triangle(far, &TestColorShader);

        let mut far_then_near = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        far_then_near.draw_triangle(far, &TestColorShader);
        far_then_near.draw_triangle(near, &TestColorShader);

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
                TestScreenVertex::new(Vec2::new(1.0, 1.0), 0.0, Srgb8::RED.to_linear()),
                TestScreenVertex::new(Vec2::new(5.0, 1.0), 0.8, Srgb8::RED.to_linear()),
                TestScreenVertex::new(Vec2::new(1.0, 5.0), 0.4, Srgb8::RED.to_linear()),
            ],
        );

        let depth = rasterizer.depth_buffer[7];

        approx::assert_relative_eq!(depth, 0.15, epsilon = f32::EPSILON);
    }

    #[test]
    fn strict_depth_test_keeps_first_fragment_on_an_exact_tie() {
        let first = ndc_triangle(0.5, Srgb8::RED);
        let tied = ndc_triangle(0.5, Srgb8::BLUE);
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");

        rasterizer.draw_triangle(first, &TestColorShader);
        let framebuffer_before = rasterizer.framebuffer.clone();
        let depth_before = rasterizer.depth_buffer.clone();
        rasterizer.draw_triangle(tied, &TestColorShader);

        assert_eq!(rasterizer.framebuffer, framebuffer_before);
        assert_eq!(rasterizer.depth_buffer, depth_before);
    }

    #[test]
    fn complete_normalized_depth_range_is_accepted() {
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        rasterizer.draw_triangle(ndc_triangle(1.0, Srgb8::BLUE), &TestColorShader);
        assert_eq!(pixel(&rasterizer.framebuffer, 2, 2), opaque(Srgb8::BLUE));

        rasterizer.draw_triangle(ndc_triangle(0.0, Srgb8::RED), &TestColorShader);
        assert_eq!(pixel(&rasterizer.framebuffer, 2, 2), opaque(Srgb8::RED));
        assert_eq!(rasterizer.depth_buffer[2 * 6 + 2], 0.0);
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

    fn ndc_triangle(depth: f32, color: Srgb8) -> [TestNdcVertex; 3] {
        [
            ndc_vertex(-0.8, -0.8, depth, color),
            ndc_vertex(0.0, 0.8, depth, color),
            ndc_vertex(0.8, -0.8, depth, color),
        ]
    }

    fn ndc_vertex(x: f32, y: f32, depth: f32, color: Srgb8) -> TestNdcVertex {
        TestNdcVertex::new(Vec3::new(x, y, depth), color.to_linear())
            .expect("test NDC vertex should be valid")
    }

    fn vertex(x: f32, y: f32, color: Srgb8) -> TestScreenVertex {
        TestScreenVertex::new(Vec2::new(x, y), 0.5, color.to_linear())
    }

    fn frame() -> Framebuffer {
        Framebuffer::new(6, 6, BACKGROUND).expect("test frame should be valid")
    }

    fn render_triangles(triangles: &[[TestScreenVertex; 3]]) -> Framebuffer {
        let mut rasterizer = Rasterizer::new(6, 6, BACKGROUND).expect("valid rasterizer");
        for triangle in triangles {
            fill_screen_triangle(&mut rasterizer, *triangle);
        }
        rasterizer.into_framebuffer()
    }

    fn fill_screen_triangle(rasterizer: &mut Rasterizer, vertices: [TestScreenVertex; 3]) {
        rasterizer.rasterize_triangle(vertices, &TestColorShader);
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
