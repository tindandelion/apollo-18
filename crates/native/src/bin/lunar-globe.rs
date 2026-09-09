use apollo18_native::run_lunar_globe_sequence;
use std::error::Error;

const LUNAR_EPHEMERIS_JSON: &[u8] = include_bytes!("../../../../assets/nasa/mooninfo_2026.json");

fn main() -> Result<(), Box<dyn Error>> {
    run_lunar_globe_sequence(std::env::args_os().skip(1), LUNAR_EPHEMERIS_JSON)
}
