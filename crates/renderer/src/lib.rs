mod color;
mod cube;
mod framebuffer;
mod globe_location;
pub mod image;
mod lunar_color_map;
mod lunar_elevation_map;
mod octasphere;
mod rasterizer;
mod scene_time;

use color::Srgb8;
pub use framebuffer::{Framebuffer, RenderError};
use glam::Vec3;
pub use lunar_color_map::LunarColorMap;
pub use lunar_elevation_map::LunarElevationMap;
pub use scene_time::{InvalidSceneTime, SceneTime};

const CUBE_ROTATION_PERIOD_SECONDS: f64 = 10.0;
const LUNAR_PHASE_PERIOD_SECONDS: f64 = 10.0;
const BACKGROUND: Srgb8 = Srgb8::from_hex(0x18_18_18);

pub fn render_lunar_globe(
    width: u32,
    height: u32,
    scene_time: SceneTime,
    color_map: &LunarColorMap,
    elevation_map: &LunarElevationMap,
) -> Result<Framebuffer, RenderError> {
    let sun_direction = octasphere::SunDirection::new(lunar_phase_sun_direction(scene_time))
        .expect("lunar phase Sun direction should be finite and nonzero");

    octasphere::render(
        width,
        height,
        BACKGROUND,
        0.0,
        color_map,
        elevation_map,
        sun_direction,
    )
}

pub fn render_cube(
    width: u32,
    height: u32,
    scene_time: SceneTime,
) -> Result<Framebuffer, RenderError> {
    let yaw =
        30.0_f32.to_radians() + periodic_angle_radians(scene_time, CUBE_ROTATION_PERIOD_SECONDS);

    cube::render_at_yaw(width, height, BACKGROUND, yaw)
}

fn lunar_phase_sun_direction(scene_time: SceneTime) -> Vec3 {
    let angle = periodic_angle_radians(scene_time, LUNAR_PHASE_PERIOD_SECONDS);

    Vec3::new(-angle.sin(), 0.0, -angle.cos())
}

fn periodic_angle_radians(scene_time: SceneTime, period_seconds: f64) -> f32 {
    let loop_time = scene_time.as_seconds().rem_euclid(period_seconds);
    let loop_fraction = (loop_time / period_seconds) as f32;
    std::f32::consts::TAU * loop_fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::Path;

    const REALISTIC_RGB_TOLERANCE: u8 = 1;
    const REALISTIC_OUTLIER_PIXEL_BUDGET: usize = 16;

    mod triangle {
        use super::*;
        use crate::color::LinearRgb;
        use crate::rasterizer::{FragmentShader, NdcVertex, Rasterizer};

        const GOLDEN_PATH: &str = "tests/goldens/first_triangle.png";
        const COLORS: [Srgb8; 3] = [Srgb8::RED, Srgb8::GREEN, Srgb8::BLUE];

        type TriangleNdcVertex = NdcVertex<LinearRgb>;

        struct TriangleColorShader;

        impl FragmentShader for TriangleColorShader {
            type Attribute = LinearRgb;

            fn shade(&self, colors: [Self::Attribute; 3], weights: [f32; 3]) -> LinearRgb {
                colors[0] * weights[0] + colors[1] * weights[1] + colors[2] * weights[2]
            }
        }

        /// A rendered triangle scene has the requested tightly packed RGBA layout.
        #[test]
        fn frame_has_expected_layout() {
            let frame = render_triangles(800, 800).expect("triangles should render");

            assert_frame_layout(&frame, 800, 800);
        }

        /// Empty triangle-scene framebuffer dimensions are rejected.
        #[test]
        fn empty_dimensions_are_rejected() {
            let empty_width = render_triangles(0, 800);
            let empty_height = render_triangles(800, 0);

            assert_eq!(
                empty_width,
                Err(RenderError::EmptyFrame {
                    width: 0,
                    height: 800
                })
            );
            assert_eq!(
                empty_height,
                Err(RenderError::EmptyFrame {
                    width: 800,
                    height: 0
                })
            );
        }

        /// The canonical triangle scene matches its exact reviewed pixels.
        #[test]
        fn matches_golden_pixels() {
            let frame = render_triangles(800, 800).expect("triangles should render");

            assert_matches_golden(&frame, Path::new(GOLDEN_PATH));
        }

        fn render_triangles(width: u32, height: u32) -> Result<Framebuffer, RenderError> {
            let mut rasterizer = Rasterizer::new(width, height, BACKGROUND)?;

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
    }

    mod cube {
        use super::*;

        const GOLDEN_PATH: &str = "tests/goldens/cube_at_zero_seconds.png";

        /// A rendered cube has the requested tightly packed RGBA layout.
        #[test]
        fn frame_has_expected_layout() {
            let frame = render_cube(800, 800, scene_time(0.0)).expect("cube should render");

            assert_frame_layout(&frame, 800, 800);
        }

        /// A rendered cube replaces background pixels with geometry.
        #[test]
        fn frame_contains_rendered_geometry() {
            let background = [0x18, 0x18, 0x18, 0xff];

            let frame = render_cube(64, 64, scene_time(0.0)).expect("cube should render");

            assert!(
                frame
                    .pixels()
                    .chunks_exact(4)
                    .any(|pixel| pixel != background)
            );
        }

        /// Cube scene time maps linearly to yaw and wraps after one rotation period.
        #[test]
        fn scene_time_maps_to_rotation_angle() {
            let start = scene_time(0.0);
            let quarter_period = scene_time(CUBE_ROTATION_PERIOD_SECONDS / 4.0);
            let one_period = scene_time(CUBE_ROTATION_PERIOD_SECONDS);

            let start_yaw = periodic_angle_radians(start, CUBE_ROTATION_PERIOD_SECONDS);
            let quarter_yaw = periodic_angle_radians(quarter_period, CUBE_ROTATION_PERIOD_SECONDS);
            let wrapped_yaw = periodic_angle_radians(one_period, CUBE_ROTATION_PERIOD_SECONDS);

            assert_eq!(start_yaw, 0.0);
            assert_eq!(quarter_yaw, std::f32::consts::FRAC_PI_2);
            assert_eq!(wrapped_yaw, 0.0);
        }

        /// Cube rendering repeats after one rotation period and is deterministic at a given time.
        #[test]
        fn pose_is_deterministic_periodic_and_time_driven() {
            let zero = render_cube(96, 96, scene_time(0.0)).expect("initial cube should render");
            let one_period = render_cube(96, 96, scene_time(CUBE_ROTATION_PERIOD_SECONDS))
                .expect("looped cube should render");
            let quarter_turn =
                render_cube(96, 96, scene_time(2.5)).expect("quarter-turn cube should render");
            let repeated_quarter_turn = render_cube(96, 96, scene_time(2.5))
                .expect("repeated quarter-turn cube should render");

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

        /// Empty cube framebuffer dimensions are rejected.
        #[test]
        fn empty_dimensions_are_rejected() {
            let scene_time = scene_time(0.0);

            let empty_width = render_cube(0, 800, scene_time);
            let empty_height = render_cube(800, 0, scene_time);

            assert_eq!(
                empty_width,
                Err(RenderError::EmptyFrame {
                    width: 0,
                    height: 800
                })
            );
            assert_eq!(
                empty_height,
                Err(RenderError::EmptyFrame {
                    width: 800,
                    height: 0
                })
            );
        }

        /// The canonical cube render matches its exact reviewed pixels.
        #[test]
        fn matches_golden_pixels() {
            let frame =
                render_cube(800, 800, scene_time(0.0)).expect("cube should render at zero seconds");

            assert_matches_golden(&frame, Path::new(GOLDEN_PATH));
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
    }

    mod lunar_globe {
        use super::*;
        use std::collections::HashSet;
        use std::sync::OnceLock;

        const COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../assets/nasa/lroc_color_2k.jpg");
        const ELEVATION_MAP_TIFF: &[u8] = include_bytes!("../../../assets/nasa/ldem_4.tif");
        const FULL_PHASE_GOLDEN_PATH: &str = "tests/goldens/lunar_phase_full_at_zero_seconds.png";
        const GIBBOUS_PHASE_GOLDEN_PATH: &str =
            "tests/goldens/lunar_phase_gibbous_at_one_point_two_five_seconds.png";
        const QUARTER_PHASE_GOLDEN_PATH: &str =
            "tests/goldens/lunar_phase_quarter_at_two_point_five_seconds.png";
        const CRESCENT_PHASE_GOLDEN_PATH: &str =
            "tests/goldens/lunar_phase_crescent_at_three_point_seven_five_seconds.png";
        const NEW_PHASE_GOLDEN_PATH: &str = "tests/goldens/lunar_phase_new_at_five_seconds.png";

        /// A rendered lunar globe has the requested tightly packed RGBA layout.
        #[test]
        fn frame_has_expected_layout() {
            let frame = render_test_lunar_globe(800, 800, scene_time(0.0));

            assert_frame_layout(&frame, 800, 800);
        }

        /// A rendered lunar globe contains varied colors from the lunar color map.
        #[test]
        fn frame_contains_color_mapped_geometry() {
            let frame = render_test_lunar_globe(64, 64, scene_time(0.0));

            let colors = frame
                .pixels()
                .chunks_exact(4)
                .map(|pixel| [pixel[0], pixel[1], pixel[2]])
                .collect::<HashSet<_>>();

            assert!(colors.len() > 16);
        }

        /// Lunar phase rendering repeats after one complete cycle and changes within the cycle.
        #[test]
        fn phase_is_deterministic_periodic_and_time_driven() {
            let full = render_test_lunar_globe(96, 96, scene_time(0.0));
            let repeated_full =
                render_test_lunar_globe(96, 96, scene_time(LUNAR_PHASE_PERIOD_SECONDS));
            let quarter = render_test_lunar_globe(96, 96, scene_time(2.5));
            let repeated_quarter = render_test_lunar_globe(96, 96, scene_time(2.5));

            assert_eq!(full, repeated_full);
            assert_eq!(quarter, repeated_quarter);
            assert_ne!(full, quarter);
        }

        /// Equal scene times selected at different frame rates render the same lunar phase.
        #[test]
        fn phase_is_independent_of_frame_rate() {
            let thirty_fps = std::num::NonZeroU32::new(30).expect("30 is nonzero");
            let sixty_fps = std::num::NonZeroU32::new(60).expect("60 is nonzero");
            let time_at_thirty_fps = SceneTime::for_frame(75, thirty_fps);
            let time_at_sixty_fps = SceneTime::for_frame(150, sixty_fps);

            let frame_at_thirty_fps = render_test_lunar_globe(96, 96, time_at_thirty_fps);
            let frame_at_sixty_fps = render_test_lunar_globe(96, 96, time_at_sixty_fps);

            assert_eq!(time_at_thirty_fps, time_at_sixty_fps);
            assert_eq!(frame_at_thirty_fps, frame_at_sixty_fps);
        }

        /// The phase cycle starts full, wanes across the left side, reaches new, and returns across the right side.
        #[test]
        fn scene_time_maps_to_key_phase_sun_directions() {
            let full = scene_time(0.0);
            let left_lit_quarter = scene_time(2.5);
            let new = scene_time(5.0);
            let right_lit_quarter = scene_time(7.5);
            let repeated_full = scene_time(10.0);

            let directions = [
                lunar_phase_sun_direction(full),
                lunar_phase_sun_direction(left_lit_quarter),
                lunar_phase_sun_direction(new),
                lunar_phase_sun_direction(right_lit_quarter),
                lunar_phase_sun_direction(repeated_full),
            ];

            approx::assert_relative_eq!(directions[0], Vec3::NEG_Z, epsilon = 1.0e-6);
            approx::assert_relative_eq!(directions[1], Vec3::NEG_X, epsilon = 1.0e-6);
            approx::assert_relative_eq!(directions[2], Vec3::Z, epsilon = 1.0e-6);
            approx::assert_relative_eq!(directions[3], Vec3::X, epsilon = 1.0e-6);
            approx::assert_relative_eq!(directions[4], Vec3::NEG_Z, epsilon = 1.0e-6);
        }

        /// Empty lunar-globe framebuffer dimensions are rejected.
        #[test]
        fn empty_dimensions_are_rejected() {
            let scene_time = scene_time(0.0);
            let color_map = lunar_color_map();
            let elevation_map = lunar_elevation_map();

            let empty_width = render_lunar_globe(0, 800, scene_time, color_map, elevation_map);
            let empty_height = render_lunar_globe(800, 0, scene_time, color_map, elevation_map);

            assert_eq!(
                empty_width,
                Err(RenderError::EmptyFrame {
                    width: 0,
                    height: 800
                })
            );
            assert_eq!(
                empty_height,
                Err(RenderError::EmptyFrame {
                    width: 800,
                    height: 0
                })
            );
        }

        /// The canonical full Moon render matches its reviewed pixels.
        #[test]
        fn full_phase_matches_golden_pixels() {
            let frame = render_test_lunar_globe(800, 800, scene_time(0.0));

            assert_matches_realistic_golden(&frame, Path::new(FULL_PHASE_GOLDEN_PATH));
        }

        /// The canonical gibbous Moon render matches its reviewed pixels.
        #[test]
        fn gibbous_phase_matches_golden_pixels() {
            let frame = render_test_lunar_globe(800, 800, scene_time(1.25));

            assert_matches_realistic_golden(&frame, Path::new(GIBBOUS_PHASE_GOLDEN_PATH));
        }

        /// The canonical quarter Moon render matches its reviewed pixels.
        #[test]
        fn quarter_phase_matches_golden_pixels() {
            let frame = render_test_lunar_globe(800, 800, scene_time(2.5));

            assert_matches_realistic_golden(&frame, Path::new(QUARTER_PHASE_GOLDEN_PATH));
        }

        /// The canonical crescent Moon render matches its reviewed pixels.
        #[test]
        fn crescent_phase_matches_golden_pixels() {
            let frame = render_test_lunar_globe(800, 800, scene_time(3.75));

            assert_matches_realistic_golden(&frame, Path::new(CRESCENT_PHASE_GOLDEN_PATH));
        }

        /// The canonical new Moon render matches its reviewed pixels.
        #[test]
        fn new_phase_matches_golden_pixels() {
            let frame = render_test_lunar_globe(800, 800, scene_time(5.0));

            assert_matches_realistic_golden(&frame, Path::new(NEW_PHASE_GOLDEN_PATH));
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

        /// The canonical lunar color map retains its recorded dimensions and bytes.
        #[test]
        fn canonical_color_map_has_recorded_dimensions_and_checksum() {
            use sha2::{Digest, Sha256};

            let digest = Sha256::digest(COLOR_MAP_JPEG);

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

        /// A committed 1440×720 elevation TIFF must keep the recorded SHA-256 checksum.
        #[test]
        fn canonical_elevation_map_has_recorded_dimensions_and_checksum() {
            use sha2::{Digest, Sha256};

            let digest = Sha256::digest(ELEVATION_MAP_TIFF);

            assert_eq!(
                (
                    lunar_elevation_map().width(),
                    lunar_elevation_map().height()
                ),
                (1440, 720)
            );
            assert_eq!(
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                "d876c867612e8941d775a005b2bc1ebaef5c15f97e04a43022a71fc21f5c9d65"
            );
        }

        fn lunar_color_map() -> &'static LunarColorMap {
            static COLOR_MAP: OnceLock<LunarColorMap> = OnceLock::new();
            COLOR_MAP.get_or_init(|| {
                let image = image::decode_jpeg(COLOR_MAP_JPEG)
                    .expect("canonical lunar color map JPEG should decode");
                LunarColorMap::new(image)
            })
        }

        fn lunar_elevation_map() -> &'static LunarElevationMap {
            static ELEVATION_MAP: OnceLock<LunarElevationMap> = OnceLock::new();
            ELEVATION_MAP.get_or_init(|| {
                let image = image::decode_float_tiff(ELEVATION_MAP_TIFF)
                    .expect("canonical lunar elevation map TIFF should decode");
                LunarElevationMap::new(image)
            })
        }

        fn render_test_lunar_globe(width: u32, height: u32, scene_time: SceneTime) -> Framebuffer {
            render_lunar_globe(
                width,
                height,
                scene_time,
                lunar_color_map(),
                lunar_elevation_map(),
            )
            .expect("lunar globe should render")
        }
    }

    fn scene_time(seconds: f64) -> SceneTime {
        SceneTime::from_seconds(seconds).expect("test scene time should be valid")
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
