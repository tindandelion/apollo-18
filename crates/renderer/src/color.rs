use std::ops::{Add, Mul};
use std::sync::OnceLock;

const SRGB_ENCODE_TABLE_INTERVALS: usize = 4_096;
const SRGB_ENCODE_TABLE_SAMPLES: usize = SRGB_ENCODE_TABLE_INTERVALS + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Srgb8 {
    channels: [u8; 3],
}

impl Srgb8 {
    pub(crate) const RED: Self = Self::from_hex(0xff_00_00);
    pub(crate) const GREEN: Self = Self::from_hex(0x00_ff_00);
    pub(crate) const BLUE: Self = Self::from_hex(0x00_00_ff);

    pub(crate) const fn from_hex(value: u32) -> Self {
        Self::from_channels([
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8,
        ])
    }

    pub(crate) const fn from_channels(channels: [u8; 3]) -> Self {
        Self { channels }
    }

    pub(crate) const fn channels(self) -> [u8; 3] {
        self.channels
    }

    pub(crate) fn to_linear(self) -> LinearRgb {
        LinearRgb::new(
            srgb_to_linear(self.channels[0] as f32 / 255.0),
            srgb_to_linear(self.channels[1] as f32 / 255.0),
            srgb_to_linear(self.channels[2] as f32 / 255.0),
        )
    }

    pub(crate) fn init_lookup_table() {
        srgb_encode_table();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LinearRgb {
    channels: [f32; 3],
}

impl LinearRgb {
    const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self {
            channels: [red, green, blue],
        }
    }

    pub(crate) fn to_srgb8(self) -> Srgb8 {
        let table = srgb_encode_table();
        Srgb8 {
            channels: self.channels.map(|channel| table.encode(channel)),
        }
    }
}

impl Add for LinearRgb {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.channels[0] + other.channels[0],
            self.channels[1] + other.channels[1],
            self.channels[2] + other.channels[2],
        )
    }
}

impl Mul<f32> for LinearRgb {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        Self::new(
            self.channels[0] * factor,
            self.channels[1] * factor,
            self.channels[2] * factor,
        )
    }
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.003_130_8 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

struct SrgbEncodeTable {
    samples: [f32; SRGB_ENCODE_TABLE_SAMPLES],
}

impl SrgbEncodeTable {
    fn new() -> Self {
        let mut samples = [0.0; SRGB_ENCODE_TABLE_SAMPLES];
        for (index, sample) in samples.iter_mut().enumerate() {
            let linear = index as f32 / SRGB_ENCODE_TABLE_INTERVALS as f32;
            *sample = linear_to_srgb(linear);
        }
        Self { samples }
    }

    fn encode(&self, linear: f32) -> u8 {
        (self.interpolate(linear.clamp(0.0, 1.0)) * 255.0).round() as u8
    }

    fn interpolate(&self, linear: f32) -> f32 {
        let position = linear * SRGB_ENCODE_TABLE_INTERVALS as f32;
        let lower_index = (position as usize).min(SRGB_ENCODE_TABLE_INTERVALS - 1);
        let fraction = position - lower_index as f32;
        let lower = self.samples[lower_index];
        let upper = self.samples[lower_index + 1];
        lower + (upper - lower) * fraction
    }
}

fn srgb_encode_table() -> &'static SrgbEncodeTable {
    static TABLE: OnceLock<SrgbEncodeTable> = OnceLock::new();

    TABLE.get_or_init(SrgbEncodeTable::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barycentric_weights_interpolate_linear_color() {
        let color = Srgb8::RED.to_linear() * 0.5
            + Srgb8::GREEN.to_linear() * 0.25
            + Srgb8::BLUE.to_linear() * 0.25;

        assert_eq!(color, LinearRgb::new(0.5, 0.25, 0.25));
        assert_eq!(color.to_srgb8().channels(), [188, 137, 137]);
    }

    #[test]
    fn hex_color_uses_red_green_blue_byte_order() {
        assert_eq!(Srgb8::from_hex(0x12_34_56).channels(), [0x12, 0x34, 0x56]);
    }

    #[test]
    fn srgb_transfer_function_uses_linear_light() {
        assert!((srgb_to_linear(0.04045) - 0.003_130_805).abs() < 0.000_000_1);
        assert!((linear_to_srgb(0.003_130_8) - 0.040_449_936).abs() < 0.000_000_1);
    }

    #[test]
    fn linear_channels_encode_to_standard_srgb_output_codes() {
        assert_eq!(
            [
                encode_test_channel(0.0),
                encode_test_channel(0.003),
                encode_test_channel(0.003_130_8),
                encode_test_channel(0.003_2),
                encode_test_channel(0.18),
                encode_test_channel(0.5),
                encode_test_channel(1.0),
            ],
            [0, 10, 10, 11, 118, 188, 255]
        );
    }

    #[test]
    fn linear_channels_clamp_to_the_displayable_range() {
        assert_eq!(encode_test_channel(-1.0), 0);
        assert_eq!(encode_test_channel(2.0), 255);
    }

    #[test]
    fn srgb_encode_table_stays_within_one_quantized_output_code() {
        for step in 0..=65_536 {
            let linear = step as f32 / 65_536.0;
            let exact = (linear_to_srgb(linear) * 255.0).round() as u8;
            let encoded = encode_test_channel(linear);

            assert!(
                encoded.abs_diff(exact) <= 1,
                "linear channel {linear} encoded as {encoded}, expected {exact}"
            );
        }
    }

    fn encode_test_channel(linear: f32) -> u8 {
        LinearRgb::new(linear, 0.0, 0.0).to_srgb8().channels()[0]
    }
}
