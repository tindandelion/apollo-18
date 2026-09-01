use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::process::Command;

const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;

#[test]
fn triangle_binary_writes_a_decodable_canonical_png() {
    let temporary_directory = tempfile::tempdir().expect("temporary directory should be created");
    let output_path = temporary_directory.path().join("triangle.png");

    run_binary(env!("CARGO_BIN_EXE_triangle"), [output_path.as_os_str()]);

    assert_canonical_rgba_png(&output_path);
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
