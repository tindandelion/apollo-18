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

    pub(crate) fn interpolate(colors: [Self; 3], weights: [f32; 3]) -> Self {
        Self::new(
            colors[0].channels[0] * weights[0]
                + colors[1].channels[0] * weights[1]
                + colors[2].channels[0] * weights[2],
            colors[0].channels[1] * weights[0]
                + colors[1].channels[1] * weights[1]
                + colors[2].channels[1] * weights[2],
            colors[0].channels[2] * weights[0]
                + colors[1].channels[2] * weights[1]
                + colors[2].channels[2] * weights[2],
        )
    }

    pub(crate) fn to_srgb8(self) -> Srgb8 {
        Srgb8 {
            channels: [
                encode_linear_channel(self.channels[0]),
                encode_linear_channel(self.channels[1]),
                encode_linear_channel(self.channels[2]),
            ],
        }
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

fn encode_linear_channel(channel: f32) -> u8 {
    (linear_to_srgb(channel.clamp(0.0, 1.0)) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barycentric_weights_interpolate_linear_color() {
        let color = LinearRgb::interpolate(
            [
                Srgb8::RED.to_linear(),
                Srgb8::GREEN.to_linear(),
                Srgb8::BLUE.to_linear(),
            ],
            [0.5, 0.25, 0.25],
        );

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
        assert_eq!(encode_linear_channel(0.0), 0);
        assert_eq!(encode_linear_channel(0.5), 188);
        assert_eq!(encode_linear_channel(1.0), 255);
    }
}
