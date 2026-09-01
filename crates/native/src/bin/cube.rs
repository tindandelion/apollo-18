use apollo18_native::write_png;
use apollo18_renderer::{CUBE_ROTATION_PERIOD_SECONDS, SceneTime, render_cube};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_OUTPUT_DIRECTORY: &str = "target/apollo18/cube/frames";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;
const FRAMES_PER_SECOND: u32 = 30;
const FRAME_COUNT: u32 = (CUBE_ROTATION_PERIOD_SECONDS * FRAMES_PER_SECOND as f64) as u32;

fn main() -> Result<(), Box<dyn Error>> {
    let output_directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIRECTORY));
    let mut rendering_time = Duration::ZERO;

    for frame_index in 0..FRAME_COUNT {
        let scene_time = scene_time_for_frame(frame_index);
        let render_started = Instant::now();
        let frame = render_cube(CANONICAL_WIDTH, CANONICAL_HEIGHT, scene_time)?;
        rendering_time += render_started.elapsed();

        write_png(&frame_path(&output_directory, frame_index), &frame)?;

        let completed_frames = frame_index + 1;
        if completed_frames % FRAMES_PER_SECOND == 0 {
            println!("rendered {completed_frames}/{FRAME_COUNT} frames");
        }
    }

    let rendering_fps = f64::from(FRAME_COUNT) / rendering_time.as_secs_f64();
    println!(
        "rendered {FRAME_COUNT} frames in {:.3}s ({rendering_fps:.2} rendering FPS); wrote {}",
        rendering_time.as_secs_f64(),
        output_directory.display()
    );
    Ok(())
}

fn scene_time_for_frame(frame_index: u32) -> SceneTime {
    let seconds = f64::from(frame_index) / f64::from(FRAMES_PER_SECOND);
    SceneTime::from_seconds(seconds).expect("frame index should produce valid scene time")
}

fn frame_path(output_directory: &Path, frame_index: u32) -> PathBuf {
    output_directory.join(format!("frame-{frame_index:04}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_schedule_samples_one_loop_without_duplicate_endpoint() {
        assert_eq!(FRAME_COUNT, 300);
        assert_eq!(scene_time_for_frame(0).as_seconds(), 0.0);
        assert_eq!(scene_time_for_frame(75).as_seconds(), 2.5);
        assert!((scene_time_for_frame(299).as_seconds() - 299.0 / 30.0).abs() < f64::EPSILON);
        assert!(scene_time_for_frame(299).as_seconds() < CUBE_ROTATION_PERIOD_SECONDS);
    }

    #[test]
    fn sequence_paths_use_fixed_width_frame_numbers() {
        assert_eq!(DEFAULT_OUTPUT_DIRECTORY, "target/apollo18/cube/frames");
        let output_directory = Path::new("frames");

        assert_eq!(
            frame_path(output_directory, 0),
            output_directory.join("frame-0000.png")
        );
        assert_eq!(
            frame_path(output_directory, 299),
            output_directory.join("frame-0299.png")
        );
    }
}
