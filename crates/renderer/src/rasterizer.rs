use crate::color::LinearRgb;
use crate::framebuffer::Framebuffer;
use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vertex {
    position: Vec2,
    color: LinearRgb,
}

impl Vertex {
    pub(crate) const fn new(position: Vec2, color: LinearRgb) -> Self {
        Self { position, color }
    }
}

pub(crate) fn fill_triangle(frame: &mut Framebuffer, mut vertices: [Vertex; 3]) {
    let mut area = edge(
        vertices[0].position,
        vertices[1].position,
        vertices[2].position,
    );
    if area == 0.0 {
        return;
    }
    if area < 0.0 {
        vertices.swap(1, 2);
        area = -area;
    }

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
        .min(frame.width() as f32) as u32;
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
        .min(frame.height() as f32) as u32;

    let edges = [
        Edge::new(vertices[1].position, vertices[2].position),
        Edge::new(vertices[2].position, vertices[0].position),
        Edge::new(vertices[0].position, vertices[1].position),
    ];
    let colors = [vertices[0].color, vertices[1].color, vertices[2].color];

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let values = edge_values(&edges, sample);
            if covers_sample(&edges, values) {
                let weights = barycentric_weights(values, area);
                let color = LinearRgb::interpolate(colors, weights);
                frame.set_pixel(x, y, color.to_srgb8());
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
    fn both_windings_render_identically_without_detaching_colors() {
        let vertices = test_triangle();
        let reversed = [vertices[0], vertices[2], vertices[1]];

        assert_eq!(render_triangles(&[vertices]), render_triangles(&[reversed]));
    }

    #[test]
    fn zero_area_triangle_performs_no_writes() {
        let triangle = [
            vertex(1.0, 1.0, Srgb8::RED),
            vertex(2.0, 2.0, Srgb8::GREEN),
            vertex(3.0, 3.0, Srgb8::BLUE),
        ];

        assert_eq!(render_triangles(&[triangle]), frame());
    }

    #[test]
    fn fully_and_partially_offscreen_triangles_are_bounded() {
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

        let extreme = [
            vertex(-f32::MAX, -f32::MAX, Srgb8::RED),
            vertex(f32::MAX, -f32::MAX, Srgb8::GREEN),
            vertex(0.0, f32::MAX, Srgb8::BLUE),
        ];
        let extreme_frame = render_triangles(&[extreme]);
        assert_eq!(extreme_frame.pixels().len(), 6 * 6 * 4);
        assert!(
            extreme_frame
                .pixels()
                .chunks_exact(4)
                .all(|pixel| pixel[3] == 0xff)
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

    fn test_triangle() -> [Vertex; 3] {
        [
            vertex(3.0, 1.0, Srgb8::RED),
            vertex(1.0, 5.0, Srgb8::GREEN),
            vertex(5.0, 5.0, Srgb8::BLUE),
        ]
    }

    fn vertex(x: f32, y: f32, color: Srgb8) -> Vertex {
        Vertex::new(Vec2::new(x, y), color.to_linear())
    }

    fn frame() -> Framebuffer {
        Framebuffer::new(6, 6, BACKGROUND).expect("test frame should be valid")
    }

    fn render_triangles(triangles: &[[Vertex; 3]]) -> Framebuffer {
        let mut frame = frame();
        for triangle in triangles {
            fill_triangle(&mut frame, *triangle);
        }
        frame
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
