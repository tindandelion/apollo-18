use crate::rasterizer::{
    FragmentShader, Framebuffer, LinearRgb, NdcVertex, Rasterizer, RenderError, Srgb8,
};
use glam::Vec3;

const COLORS: [Srgb8; 3] = [Srgb8::RED, Srgb8::GREEN, Srgb8::BLUE];

type TriangleNdcVertex = NdcVertex<LinearRgb>;

struct TriangleColorShader;

impl FragmentShader for TriangleColorShader {
    type Attribute = LinearRgb;

    fn shade(&self, colors: [Self::Attribute; 3], weights: [f32; 3]) -> Srgb8 {
        (colors[0] * weights[0] + colors[1] * weights[1] + colors[2] * weights[2]).to_srgb8()
    }
}

pub(crate) fn render(
    width: u32,
    height: u32,
    background: Srgb8,
) -> Result<Framebuffer, RenderError> {
    let mut rasterizer = Rasterizer::new(width, height, background)?;

    let near = [
        scene_vertex(-0.8, -0.3, 0.2, COLORS[0]),
        scene_vertex(-0.25, 0.7, 0.2, COLORS[2]),
        scene_vertex(0.3, -0.55, 0.2, COLORS[1]),
    ];
    let far = [
        scene_vertex(-0.75, -0.65, 0.75, COLORS[2]),
        scene_vertex(0.0, 0.75, 0.75, COLORS[1]),
        scene_vertex(0.75, -0.65, 0.75, COLORS[0]),
    ];
    let back_facing = [
        scene_vertex(0.45, 0.25, 0.1, COLORS[0]),
        scene_vertex(0.9, 0.25, 0.1, COLORS[2]),
        scene_vertex(0.65, 0.8, 0.1, COLORS[1]),
    ];

    rasterizer.draw_triangle(near, &TriangleColorShader);
    rasterizer.draw_triangle(far, &TriangleColorShader);
    rasterizer.draw_triangle(back_facing, &TriangleColorShader);

    Ok(rasterizer.into_framebuffer())
}

fn scene_vertex(x: f32, y: f32, depth: f32, color: Srgb8) -> TriangleNdcVertex {
    TriangleNdcVertex::new(Vec3::new(x, y, depth), color.to_linear())
        .expect("scene NDC vertex should be valid")
}
