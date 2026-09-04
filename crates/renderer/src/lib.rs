mod color;
mod cube;
mod framebuffer;
pub mod image;
mod lunar_color_map;
mod octasphere;
mod rasterizer;
mod scene_time;

use color::Srgb8;
pub use framebuffer::{Framebuffer, RenderError};
use glam::Vec3;
pub use lunar_color_map::LunarColorMap;
pub use scene_time::{InvalidSceneTime, SceneTime};

pub const ROTATION_PERIOD_SECONDS: f64 = 10.0;

const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);
const INITIAL_SUN_ANGLE_RADIANS: f32 = 30.0_f32.to_radians();

pub fn render_lunar_globe(
    width: u32,
    height: u32,
    scene_time: SceneTime,
    color_map: &LunarColorMap,
) -> Result<Framebuffer, RenderError> {
    let yaw = periodic_angle_radians(scene_time, ROTATION_PERIOD_SECONDS);
    let sun_direction = octasphere::SunDirection::new(Vec3::new(
        INITIAL_SUN_ANGLE_RADIANS.sin(),
        0.0,
        -INITIAL_SUN_ANGLE_RADIANS.cos(),
    ))
    .expect("initial Sun direction should be finite and nonzero");

    octasphere::render(width, height, BACKGROUND, yaw, color_map, sun_direction)
}

pub fn render_cube(
    width: u32,
    height: u32,
    scene_time: SceneTime,
) -> Result<Framebuffer, RenderError> {
    let yaw = 30.0_f32.to_radians() + periodic_angle_radians(scene_time, ROTATION_PERIOD_SECONDS);

    cube::render_at_yaw(width, height, BACKGROUND, yaw)
}

fn periodic_angle_radians(scene_time: SceneTime, period_seconds: f64) -> f32 {
    let loop_time = scene_time.as_seconds().rem_euclid(period_seconds);
    let loop_fraction = (loop_time / period_seconds) as f32;
    std::f32::consts::TAU * loop_fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::LinearRgb;
    use crate::rasterizer::{FragmentShader, NdcVertex, Rasterizer};
    use glam::Vec3;
    use std::collections::HashSet;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::Path;
    use std::sync::OnceLock;

    const LUNAR_COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../assets/nasa/lroc_color_2k.jpg");
    const TRIANGLE_GOLDEN_PATH: &str = "tests/goldens/first_triangle.png";
    const CUBE_GOLDEN_PATH: &str = "tests/goldens/cube_at_zero_seconds.png";
    const GIBBOUS_LUNAR_GOLDEN_PATH: &str = "tests/goldens/gibbous_lunar_globe_at_zero_seconds.png";
    const ROTATED_LUNAR_GOLDEN_PATH: &str =
        "tests/goldens/gibbous_lunar_globe_at_two_point_five_seconds.png";
    const REALISTIC_RGB_TOLERANCE: u8 = 1;
    const REALISTIC_OUTLIER_PIXEL_BUDGET: usize = 16;
    const TRIANGLE_COLORS: [Srgb8; 3] = [Srgb8::RED, Srgb8::GREEN, Srgb8::BLUE];

    type TriangleNdcVertex = NdcVertex<LinearRgb>;

    struct TriangleColorShader;

    impl FragmentShader for TriangleColorShader {
        type Attribute = LinearRgb;

        fn shade(&self, colors: [Self::Attribute; 3], weights: [f32; 3]) -> LinearRgb {
            colors[0] * weights[0] + colors[1] * weights[1] + colors[2] * weights[2]
        }
    }

    fn render_triangles(width: u32, height: u32) -> Result<Framebuffer, RenderError> {
        let mut rasterizer = Rasterizer::new(width, height, BACKGROUND)?;

        let near = [
            scene_vertex(-0.8, -0.3, 0.2, TRIANGLE_COLORS[0]),
            scene_vertex(-0.25, 0.7, 0.2, TRIANGLE_COLORS[2]),
            scene_vertex(0.3, -0.55, 0.2, TRIANGLE_COLORS[1]),
        ];
        let far = [
            scene_vertex(-0.75, -0.65, 0.75, TRIANGLE_COLORS[2]),
            scene_vertex(0.0, 0.75, 0.75, TRIANGLE_COLORS[1]),
            scene_vertex(0.75, -0.65, 0.75, TRIANGLE_COLORS[0]),
        ];
        let back_facing = [
            scene_vertex(0.45, 0.25, 0.1, TRIANGLE_COLORS[0]),
            scene_vertex(0.9, 0.25, 0.1, TRIANGLE_COLORS[2]),
            scene_vertex(0.65, 0.8, 0.1, TRIANGLE_COLORS[1]),
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

    #[test]
    fn triangle_frame_has_expected_layout() {
        let frame = render_triangles(800, 800).expect("triangles should render");

        assert_frame_layout(&frame, 800, 800);
    }

    #[test]
    fn cube_frame_has_expected_layout() {
        let frame = render_cube(800, 800, scene_time(0.0)).expect("cube should render");

        assert_frame_layout(&frame, 800, 800);
    }

    #[test]
    fn lunar_frame_has_expected_layout() {
        let frame = render_test_lunar_globe(800, 800, scene_time(0.0));

        assert_frame_layout(&frame, 800, 800);
    }

    #[test]
    fn lunar_frame_contains_color_mapped_geometry() {
        let frame = render_test_lunar_globe(64, 64, scene_time(0.0));
        let colors = frame
            .pixels()
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<HashSet<_>>();

        assert!(colors.len() > 16);
    }

    #[test]
    fn lunar_pose_is_deterministic_periodic_and_time_driven() {
        let zero = render_test_lunar_globe(96, 96, scene_time(0.0));
        let one_period = render_test_lunar_globe(96, 96, scene_time(ROTATION_PERIOD_SECONDS));
        let quarter_turn = render_test_lunar_globe(96, 96, scene_time(2.5));
        let repeated_quarter_turn = render_test_lunar_globe(96, 96, scene_time(2.5));

        assert_eq!(zero, one_period);
        assert_eq!(quarter_turn, repeated_quarter_turn);
        assert_ne!(zero, quarter_turn);
    }

    #[test]
    fn scene_time_maps_to_globe_rotation_angle() {
        let start = scene_time(0.0);
        let quarter_period = scene_time(ROTATION_PERIOD_SECONDS / 4.0);
        let one_period = scene_time(ROTATION_PERIOD_SECONDS);

        let start_yaw = periodic_angle_radians(start, ROTATION_PERIOD_SECONDS);
        let quarter_yaw = periodic_angle_radians(quarter_period, ROTATION_PERIOD_SECONDS);
        let wrapped_yaw = periodic_angle_radians(one_period, ROTATION_PERIOD_SECONDS);

        assert_eq!(start_yaw, 0.0);
        assert_eq!(quarter_yaw, std::f32::consts::FRAC_PI_2);
        assert_eq!(wrapped_yaw, 0.0);
    }

    #[test]
    fn cube_frame_contains_rendered_geometry() {
        let frame = render_cube(64, 64, scene_time(0.0)).expect("cube should render");
        let background = [0x18, 0x18, 0x18, 0xff];

        assert!(
            frame
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel != background)
        );
    }

    #[test]
    fn cube_pose_is_deterministic_periodic_and_time_driven() {
        let zero = render_cube(96, 96, scene_time(0.0)).expect("initial cube should render");
        let one_period = render_cube(96, 96, scene_time(ROTATION_PERIOD_SECONDS))
            .expect("looped cube should render");
        let quarter_turn =
            render_cube(96, 96, scene_time(2.5)).expect("quarter-turn cube should render");
        let repeated_quarter_turn =
            render_cube(96, 96, scene_time(2.5)).expect("repeated quarter-turn cube should render");

        assert_eq!(zero, one_period);
        assert_eq!(quarter_turn, repeated_quarter_turn);
        assert_ne!(zero, quarter_turn);
        assert_face_visibility(
            &zero,
            [0xff_00_00, 0x00_ff_00, 0x00_00_ff],
            [0x00_ff_ff, 0xff_00_ff, 0xff_ff_00],
        );
        assert_face_visibility(
            &quarter_turn,
            [0x00_ff_ff, 0x00_ff_00, 0x00_00_ff],
            [0xff_00_00, 0xff_00_ff, 0xff_ff_00],
        );
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        assert_eq!(
            render_triangles(0, 800),
            Err(RenderError::EmptyFrame {
                width: 0,
                height: 800
            })
        );
        assert_eq!(
            render_triangles(800, 0),
            Err(RenderError::EmptyFrame {
                width: 800,
                height: 0
            })
        );
        assert_eq!(
            render_lunar_globe(0, 800, scene_time(0.0), lunar_color_map()),
            Err(RenderError::EmptyFrame {
                width: 0,
                height: 800
            })
        );
        assert_eq!(
            render_lunar_globe(800, 0, scene_time(0.0), lunar_color_map()),
            Err(RenderError::EmptyFrame {
                width: 800,
                height: 0
            })
        );
        assert_eq!(
            render_cube(0, 800, scene_time(0.0)),
            Err(RenderError::EmptyFrame {
                width: 0,
                height: 800
            })
        );
        assert_eq!(
            render_cube(800, 0, scene_time(0.0)),
            Err(RenderError::EmptyFrame {
                width: 800,
                height: 0
            })
        );
    }

    #[test]
    fn triangle_matches_golden_pixels() {
        let frame = render_triangles(800, 800).expect("triangles should render");

        assert_matches_golden(&frame, Path::new(TRIANGLE_GOLDEN_PATH));
    }

    #[test]
    fn gibbous_lunar_globe_matches_golden_pixels() {
        let frame = render_test_lunar_globe(800, 800, scene_time(0.0));

        assert_matches_realistic_golden(&frame, Path::new(GIBBOUS_LUNAR_GOLDEN_PATH));
    }

    #[test]
    fn rotated_gibbous_lunar_globe_matches_golden_pixels() {
        let frame = render_test_lunar_globe(800, 800, scene_time(2.5));

        assert_matches_realistic_golden(&frame, Path::new(ROTATED_LUNAR_GOLDEN_PATH));
    }

    /// A per-channel RGB difference of one is within tolerance and is not an outlier.
    #[test]
    fn realistic_golden_treats_one_rgb_step_as_a_match() {
        let expected = [10, 20, 30, 255];
        let actual = [11, 19, 30, 255];

        let comparison = compare_realistic_pixels(&actual, &expected);

        assert!(!comparison.exceeds_budget());
        assert_eq!(comparison.outlier_pixels, 0);
        assert_eq!(comparison.maximum_rgb_difference, 1);
    }

    /// Sixteen pixels may exceed the per-channel RGB tolerance.
    #[test]
    fn realistic_golden_allows_sixteen_outlier_pixels() {
        let expected = [10, 10, 10, 255].repeat(16);
        let mut actual = expected.clone();
        for pixel in 0..16 {
            actual[pixel * 4] = 30;
        }

        let comparison = compare_realistic_pixels(&actual, &expected);

        assert!(!comparison.exceeds_budget());
        assert_eq!(comparison.outlier_pixels, 16);
        assert_eq!(comparison.maximum_rgb_difference, 20);
    }

    /// Seventeen pixels over the RGB tolerance exceed the outlier budget.
    #[test]
    fn realistic_golden_rejects_seventeen_outlier_pixels() {
        let expected = [10, 10, 10, 255].repeat(17);
        let mut actual = expected.clone();
        for pixel in 0..17 {
            actual[pixel * 4] = 30;
        }

        let comparison = compare_realistic_pixels(&actual, &expected);

        assert!(comparison.exceeds_budget());
        assert_eq!(comparison.outlier_pixels, 17);
    }

    /// Alpha must match exactly and does not consume the outlier budget.
    #[test]
    fn realistic_golden_rejects_an_alpha_mismatch() {
        let expected = [10, 10, 10, 255];
        let actual = [10, 10, 10, 254];

        let comparison = compare_realistic_pixels(&actual, &expected);

        assert!(comparison.exceeds_budget());
        assert_eq!(comparison.outlier_pixels, 0);
        assert_eq!(comparison.alpha_mismatches, 1);
    }

    #[test]
    fn cube_matches_golden_pixels() {
        let frame =
            render_cube(800, 800, scene_time(0.0)).expect("cube at zero seconds should render");

        assert_matches_golden(&frame, Path::new(CUBE_GOLDEN_PATH));
    }

    #[test]
    fn canonical_lunar_color_map_has_recorded_dimensions_and_checksum() {
        use sha2::{Digest, Sha256};

        let digest = Sha256::digest(LUNAR_COLOR_MAP_JPEG);

        assert_eq!(
            (lunar_color_map().width(), lunar_color_map().height()),
            (2048, 1024)
        );
        assert_eq!(
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            "f7130a1822681fa7512d7dcfd40db8c10b9ba4f06777910348698260ed7a2170"
        );
    }

    fn lunar_color_map() -> &'static LunarColorMap {
        static COLOR_MAP: OnceLock<LunarColorMap> = OnceLock::new();
        COLOR_MAP.get_or_init(|| {
            let image = image::decode_jpeg(LUNAR_COLOR_MAP_JPEG)
                .expect("canonical lunar color map JPEG should decode");
            LunarColorMap::new(image)
        })
    }

    fn render_test_lunar_globe(width: u32, height: u32, scene_time: SceneTime) -> Framebuffer {
        render_lunar_globe(width, height, scene_time, lunar_color_map())
            .expect("lunar globe should render")
    }

    fn scene_time(seconds: f64) -> SceneTime {
        SceneTime::from_seconds(seconds).expect("test scene time should be valid")
    }

    fn assert_face_visibility(frame: &Framebuffer, visible: [u32; 3], hidden: [u32; 3]) {
        for color in visible {
            assert!(
                frame_contains_color(frame, color),
                "frame should contain #{color:06x}"
            );
        }
        for color in hidden {
            assert!(
                !frame_contains_color(frame, color),
                "frame should not contain #{color:06x}"
            );
        }
    }

    fn frame_contains_color(frame: &Framebuffer, color: u32) -> bool {
        let red = (color >> 16) as u8;
        let green = (color >> 8) as u8;
        let blue = color as u8;
        frame
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel == [red, green, blue, 0xff])
    }

    fn assert_frame_layout(frame: &Framebuffer, width: u32, height: u32) {
        assert_eq!(frame.width(), width);
        assert_eq!(frame.height(), height);
        assert_eq!(frame.pixels().len(), width as usize * height as usize * 4);
        assert!(frame.pixels().chunks_exact(4).all(|pixel| pixel[3] == 0xff));
    }

    fn assert_matches_golden(frame: &Framebuffer, golden_path: &Path) {
        if std::env::var_os("APOLLO18_UPDATE_GOLDENS").is_some() {
            write_png(golden_path, frame).expect("golden should be written");
        }

        let (width, height, pixels) = read_png(golden_path).expect("golden should be readable");
        assert_eq!(frame.width(), width);
        assert_eq!(frame.height(), height);
        assert_eq!(frame.pixels(), pixels);
    }

    struct RealisticGoldenComparison {
        maximum_rgb_difference: u8,
        outlier_pixels: usize,
        alpha_mismatches: usize,
        amplified: Vec<u8>,
    }

    impl RealisticGoldenComparison {
        fn exceeds_budget(&self) -> bool {
            self.outlier_pixels > REALISTIC_OUTLIER_PIXEL_BUDGET || self.alpha_mismatches > 0
        }
    }

    fn compare_realistic_pixels(actual: &[u8], expected: &[u8]) -> RealisticGoldenComparison {
        let mut maximum_rgb_difference = 0_u8;
        let mut outlier_pixels = 0_usize;
        let mut alpha_mismatches = 0_usize;
        let mut amplified = Vec::with_capacity(expected.len());

        for (actual, expected) in actual.chunks_exact(4).zip(expected.chunks_exact(4)) {
            let mut pixel_is_outlier = false;
            for channel in 0..3 {
                let difference = actual[channel].abs_diff(expected[channel]);
                maximum_rgb_difference = maximum_rgb_difference.max(difference);
                pixel_is_outlier |= difference > REALISTIC_RGB_TOLERANCE;
                amplified.push(difference.saturating_mul(16));
            }
            outlier_pixels += usize::from(pixel_is_outlier);
            alpha_mismatches += usize::from(actual[3] != expected[3]);
            amplified.push(0xff);
        }

        RealisticGoldenComparison {
            maximum_rgb_difference,
            outlier_pixels,
            alpha_mismatches,
            amplified,
        }
    }

    fn assert_matches_realistic_golden(frame: &Framebuffer, golden_path: &Path) {
        if std::env::var_os("APOLLO18_UPDATE_GOLDENS").is_some() {
            write_png(golden_path, frame).expect("golden should be written");
        }

        let (width, height, expected) =
            read_png(golden_path).expect("realistic golden should be readable");
        assert_eq!(frame.width(), width);
        assert_eq!(frame.height(), height);

        let comparison = compare_realistic_pixels(frame.pixels(), &expected);
        if comparison.exceeds_budget() {
            let summary = format!(
                "maximum RGB difference: {}\noutlier pixels: {} (budget \
                 {REALISTIC_OUTLIER_PIXEL_BUDGET})\nalpha mismatches: {}\n",
                comparison.maximum_rgb_difference,
                comparison.outlier_pixels,
                comparison.alpha_mismatches
            );
            write_golden_diff(golden_path, width, height, &comparison.amplified, &summary)
                .expect("realistic golden diff artifacts should be written");
            panic!("realistic lunar golden differs\n{summary}");
        }
    }

    fn write_golden_diff(
        golden_path: &Path,
        width: u32,
        height: u32,
        pixels: &[u8],
        summary: &str,
    ) -> Result<(), Box<dyn Error>> {
        let output_directory = Path::new("../../target/apollo18/golden-diffs");
        fs::create_dir_all(output_directory)?;
        let stem = golden_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("lunar-golden");
        write_rgba_png(
            &output_directory.join(format!("{stem}-amplified.png")),
            width,
            height,
            pixels,
        )?;
        fs::write(
            output_directory.join(format!("{stem}-summary.txt")),
            summary,
        )?;
        Ok(())
    }

    fn read_png(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let mut reader = decoder.read_info()?;
        let mut pixels = vec![0; reader.output_buffer_size()];
        let output = reader.next_frame(&mut pixels)?;
        pixels.truncate(output.buffer_size());

        assert_eq!(output.color_type, png::ColorType::Rgba);
        assert_eq!(output.bit_depth, png::BitDepth::Eight);

        Ok((output.width, output.height, pixels))
    }

    fn write_png(path: &Path, frame: &Framebuffer) -> Result<(), Box<dyn Error>> {
        write_rgba_png(path, frame.width(), frame.height(), frame.pixels())
    }

    fn write_rgba_png(
        path: &Path,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(pixels)?;
        Ok(())
    }
}
