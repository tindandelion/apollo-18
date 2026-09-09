use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::process::Command;

const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

/// The lunar binary renders the same deterministic first-month sequence on repeated runs.
#[test]
fn lunar_globe_binary_writes_a_deterministic_canonical_png_sequence() {
    let temporary_directory = tempfile::tempdir().expect("temporary directory should be created");
    let output_directory = temporary_directory.path().join("lunar");
    let repeated_output_directory = temporary_directory.path().join("repeated-lunar");

    run_binary(
        env!("CARGO_BIN_EXE_lunar-globe"),
        [
            OsStr::new("--fps"),
            OsStr::new("1"),
            OsStr::new("--num-frames"),
            OsStr::new("2"),
            output_directory.as_os_str(),
        ],
    );
    run_binary(
        env!("CARGO_BIN_EXE_lunar-globe"),
        [
            OsStr::new("--fps"),
            OsStr::new("1"),
            OsStr::new("--num-frames"),
            OsStr::new("2"),
            repeated_output_directory.as_os_str(),
        ],
    );

    assert_canonical_rgba_png(&output_directory.join("frame-0000.png"));
    assert_canonical_rgba_png(&output_directory.join("frame-0001.png"));
    assert_eq!(
        std::fs::read(output_directory.join("frame-0000.png")).expect("frame should be readable"),
        std::fs::read(repeated_output_directory.join("frame-0000.png"))
            .expect("repeated frame should be readable")
    );
    assert_eq!(
        std::fs::read(output_directory.join("frame-0001.png")).expect("frame should be readable"),
        std::fs::read(repeated_output_directory.join("frame-0001.png"))
            .expect("repeated frame should be readable")
    );
}

/// Incomplete ephemeris coverage fails before any numbered PNG is written.
#[test]
fn lunar_sequence_rejects_incomplete_coverage_before_writing_frames() {
    let temporary_directory = tempfile::tempdir().expect("temporary directory should be created");
    let output_directory = temporary_directory.path().join("incomplete-lunar");
    let ephemeris = br#"[
        {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0}
    ]"#;

    let result = apollo18_native::run_lunar_globe_sequence(
        [
            OsStr::new("--fps"),
            OsStr::new("1"),
            OsStr::new("--num-frames"),
            OsStr::new("1"),
            output_directory.as_os_str(),
        ],
        ephemeris,
    );

    assert!(
        result
            .expect_err("incomplete coverage should fail")
            .to_string()
            .contains("one complete mean synodic month")
    );
    assert!(!output_directory.join("frame-0000.png").exists());
}

#[test]
fn cube_binary_writes_a_decodable_canonical_png_sequence() {
    let temporary_directory = tempfile::tempdir().expect("temporary directory should be created");
    let output_directory = temporary_directory.path().join("cube");

    run_binary(
        env!("CARGO_BIN_EXE_cube"),
        [
            OsStr::new("--fps"),
            OsStr::new("1"),
            OsStr::new("--num-frames"),
            OsStr::new("1"),
            output_directory.as_os_str(),
        ],
    );

    assert_canonical_rgba_png(&output_directory.join("frame-0000.png"));
}

fn run_binary<'a>(binary: &str, arguments: impl IntoIterator<Item = &'a OsStr>) {
    let output = Command::new(binary)
        .args(arguments)
        .output()
        .expect("native milestone binary should execute");

    assert!(
        output.status.success(),
        "native milestone binary failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_canonical_rgba_png(path: &Path) {
    let file = File::open(path).expect("native milestone binary should write its PNG artifact");
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().expect("PNG header should decode");
    let mut pixels = vec![0; reader.output_buffer_size()];
    let output = reader
        .next_frame(&mut pixels)
        .expect("PNG pixels should decode");

    assert_eq!(output.width, CANONICAL_WIDTH);
    assert_eq!(output.height, CANONICAL_HEIGHT);
    assert_eq!(output.color_type, png::ColorType::Rgba);
    assert_eq!(output.bit_depth, png::BitDepth::Eight);
    assert_eq!(
        output.buffer_size(),
        (CANONICAL_WIDTH * CANONICAL_HEIGHT * 4) as usize
    );
}
