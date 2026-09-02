use glam::Vec2;

pub(super) fn rasterize_fragments(
    width: u32,
    height: u32,
    positions: [Vec2; 3],
    depths: [f32; 3],
    mut shade: impl FnMut(u32, u32, f32, [f32; 3]),
) {
    let area = edge(positions[0], positions[1], positions[2]);
    if area <= 0.0 {
        return;
    }

    let min_x = positions
        .map(|position| position.x)
        .into_iter()
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_x = positions
        .map(|position| position.x)
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(width as f32) as u32;
    let min_y = positions
        .map(|position| position.y)
        .into_iter()
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_y = positions
        .map(|position| position.y)
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(height as f32) as u32;
    let edges = [
        Edge::new(positions[1], positions[2]),
        Edge::new(positions[2], positions[0]),
        Edge::new(positions[0], positions[1]),
    ];

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let values = edge_values(&edges, sample);
            if covers_sample(&edges, values) {
                let weights = barycentric_weights(values, area);
                shade(x, y, interpolate(depths, weights), weights);
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
}
