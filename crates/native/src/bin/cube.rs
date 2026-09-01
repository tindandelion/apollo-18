use apollo18_native::write_png;
use apollo18_renderer::{SceneTime, render_cube};
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_OUTPUT_DIRECTORY: &str = "target/apollo18/cube/frames";
const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_args(std::env::args_os().skip(1))?;
    let mut rendering_time = Duration::ZERO;

    for frame_index in 0..options.frame_count {
        let scene_time = SceneTime::for_frame(frame_index, options.frames_per_second);
        let render_started = Instant::now();
        let frame = render_cube(CANONICAL_WIDTH, CANONICAL_HEIGHT, scene_time)?;
        rendering_time += render_started.elapsed();

        write_png(&frame_path(&options.output_directory, frame_index), &frame)?;

        let completed_frames = frame_index + 1;
        if completed_frames % options.frames_per_second.get() == 0 {
            println!("rendered {completed_frames}/{} frames", options.frame_count);
        }
    }

    let rendering_fps = f64::from(options.frame_count) / rendering_time.as_secs_f64();
    println!(
        "rendered {} frames in {:.3}s ({rendering_fps:.2} rendering FPS); wrote {}",
        options.frame_count,
        rendering_time.as_secs_f64(),
        options.output_directory.display()
    );
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    frames_per_second: NonZeroU32,
    frame_count: u32,
    output_directory: PathBuf,
}

fn parse_args<I, S>(arguments: I) -> Result<Options, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut arguments = arguments.into_iter().map(Into::into);
    let mut frames_per_second = None;
    let mut frame_count = None;
    let mut output_directory = None;

    while let Some(argument) = arguments.next() {
        if argument == "--fps" {
            if frames_per_second.is_some() {
                return Err(CliError::new("--fps may only be supplied once"));
            }
            frames_per_second = Some(parse_nonzero_u32(arguments.next(), "--fps")?);
        } else if argument == "--num-frames" {
            if frame_count.is_some() {
                return Err(CliError::new("--num-frames may only be supplied once"));
            }
            frame_count = Some(parse_nonzero_u32(arguments.next(), "--num-frames")?.get());
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(CliError::new(format!(
                "unknown option: {}",
                argument.to_string_lossy()
            )));
        } else if output_directory.is_none() {
            output_directory = Some(PathBuf::from(argument));
        } else {
            return Err(CliError::new("only one output directory may be supplied"));
        }
    }

    Ok(Options {
        frames_per_second: frames_per_second
            .ok_or_else(|| CliError::new("required option --fps is missing"))?,
        frame_count: frame_count
            .ok_or_else(|| CliError::new("required option --num-frames is missing"))?,
        output_directory: output_directory
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIRECTORY)),
    })
}

fn parse_nonzero_u32(value: Option<OsString>, option: &str) -> Result<NonZeroU32, CliError> {
    let value = value.ok_or_else(|| CliError::new(format!("{option} requires a value")))?;
    let value = value
        .to_str()
        .ok_or_else(|| CliError::new(format!("{option} must be a positive integer")))?;
    let parsed = value
        .parse::<u32>()
        .map_err(|_| CliError::new(format!("{option} must be a positive integer")))?;

    NonZeroU32::new(parsed)
        .ok_or_else(|| CliError::new(format!("{option} must be a positive integer")))
}

#[derive(Debug)]
struct CliError(String);

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}\nusage: cube --fps <FPS> --num-frames <COUNT> [OUTPUT_DIRECTORY]",
            self.0
        )
    }
}

impl Error for CliError {}

fn frame_path(output_directory: &Path, frame_index: u32) -> PathBuf {
    output_directory.join(format!("frame-{frame_index:04}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_sequence_options_are_parsed() {
        let options = parse_args(["--fps", "24", "--num-frames", "120", "custom-frames"])
            .expect("arguments should be valid");

        assert_eq!(options.frames_per_second.get(), 24);
        assert_eq!(options.frame_count, 120);
        assert_eq!(options.output_directory, Path::new("custom-frames"));
    }

    #[test]
    fn missing_zero_and_unknown_sequence_options_are_rejected() {
        for arguments in [
            vec!["--fps", "30"],
            vec!["--fps", "0", "--num-frames", "300"],
            vec!["--fps", "30", "--num-frames", "0"],
            vec!["--fps", "30", "--num-frames", "300", "--wat"],
        ] {
            assert!(parse_args(arguments).is_err());
        }
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
