use crate::lunar_appearance::{LunarAppearance, SunDirection};
use glam::{Mat4, Vec3};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use time::{PrimitiveDateTime, macros::format_description};

const SECONDS_PER_HOUR: i64 = 3_600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AstronomicalInstant(f64);

impl AstronomicalInstant {
    pub const fn from_unix_seconds(unix_seconds: i64) -> Self {
        Self(unix_seconds as f64)
    }

    pub(crate) const fn add(self, seconds: f64) -> Self {
        Self(self.0 + seconds)
    }
}

#[derive(Debug, Clone)]
pub struct LunarEphemeris {
    first_timestamp: i64,
    samples: Vec<EphemerisSample>,
}

impl LunarEphemeris {
    pub fn from_nasa_json(source: &[u8]) -> Result<Self, EphemerisError> {
        let records: Vec<NasaRecord> = serde_json::from_slice(source)
            .map_err(|error| EphemerisError::InvalidJson(error.to_string()))?;
        if records.is_empty() {
            return Err(EphemerisError::EmptyData);
        }

        let mut timestamps = Vec::with_capacity(records.len());
        let mut samples = Vec::with_capacity(records.len());
        for (index, record) in records.into_iter().enumerate() {
            let timestamp = parse_timestamp(&record.time)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            let subsolar_point = LunarCoordinates::new(record.subsolar.lon, record.subsolar.lat)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            let subearth_point = LunarCoordinates::new(record.subearth.lon, record.subearth.lat)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            let position_angle = LunarPositionAngle::new(record.posangle)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            timestamps.push(timestamp);
            samples.push(EphemerisSample {
                subsolar_point,
                subearth_point,
                position_angle,
            });
        }

        for (index, pair) in timestamps.windows(2).enumerate() {
            let interval = pair[1] - pair[0];
            if interval == 0 {
                return Err(EphemerisError::DuplicateTimestamp { index: index + 1 });
            }
            if interval != SECONDS_PER_HOUR {
                return Err(EphemerisError::NonHourlyTimestamp { index: index + 1 });
            }
        }

        Ok(Self {
            first_timestamp: timestamps[0],
            samples,
        })
    }

    pub(crate) fn covers(&self, start: AstronomicalInstant, end: AstronomicalInstant) -> bool {
        let first_timestamp = self.first_timestamp as f64;
        let last_timestamp =
            (self.first_timestamp + (self.samples.len() - 1) as i64 * SECONDS_PER_HOUR) as f64;

        first_timestamp <= start.0 && last_timestamp >= end.0
    }

    pub(crate) fn lunar_appearance_at(
        &self,
        instant: AstronomicalInstant,
    ) -> Option<LunarAppearance> {
        self.nearest_sample_at(instant)
            .map(EphemerisSample::lunar_appearance)
    }

    fn nearest_sample_at(&self, instant: AstronomicalInstant) -> Option<EphemerisSample> {
        let target_hours = (instant.0 - self.first_timestamp as f64) / SECONDS_PER_HOUR as f64;
        let last_index = (self.samples.len() - 1) as f64;
        if !(0.0..=last_index).contains(&target_hours) {
            return None;
        }

        self.samples.get(target_hours.round() as usize).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EphemerisSample {
    subsolar_point: LunarCoordinates,
    subearth_point: LunarCoordinates,
    position_angle: LunarPositionAngle,
}

impl EphemerisSample {
    fn object_to_world(self) -> Mat4 {
        let center_longitude =
            Mat4::from_rotation_y(self.subearth_point.longitude_degrees.to_radians() as f32);
        let center_latitude =
            Mat4::from_rotation_x(-self.subearth_point.latitude_degrees.to_radians() as f32);
        let align_with_celestial_north =
            Mat4::from_rotation_z(self.position_angle.degrees.to_radians() as f32);

        align_with_celestial_north * center_latitude * center_longitude
    }

    fn lunar_appearance(self) -> LunarAppearance {
        let object_to_world = self.object_to_world();
        let sun_direction = object_to_world.transform_vector3(self.subsolar_point.globe_location());
        let sun_direction = SunDirection::new(sun_direction)
            .expect("validated ephemeris coordinates produce a valid Sun direction");

        LunarAppearance::new(object_to_world, sun_direction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LunarPositionAngle {
    degrees: f64,
}

impl LunarPositionAngle {
    fn new(degrees: f64) -> Result<Self, &'static str> {
        if !degrees.is_finite() || !(0.0..360.0).contains(&degrees) {
            return Err("lunar position angle must be finite and in the range [0, 360) degrees");
        }

        Ok(Self { degrees })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LunarCoordinates {
    pub(crate) longitude_degrees: f64,
    pub(crate) latitude_degrees: f64,
}

impl LunarCoordinates {
    pub(crate) fn new(longitude_degrees: f64, latitude_degrees: f64) -> Result<Self, &'static str> {
        if !longitude_degrees.is_finite() || !(-180.0..=180.0).contains(&longitude_degrees) {
            return Err("lunar longitude must be finite and between -180 and 180 degrees");
        }
        if !latitude_degrees.is_finite() || !(-90.0..=90.0).contains(&latitude_degrees) {
            return Err("lunar latitude must be finite and between -90 and 90 degrees");
        }

        Ok(Self {
            longitude_degrees,
            latitude_degrees,
        })
    }

    pub(crate) fn globe_location(self) -> Vec3 {
        let longitude = self.longitude_degrees.to_radians();
        let latitude = self.latitude_degrees.to_radians();
        let horizontal_radius = latitude.cos();

        Vec3::new(
            (horizontal_radius * longitude.sin()) as f32,
            latitude.sin() as f32,
            (-horizontal_radius * longitude.cos()) as f32,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EphemerisError {
    InvalidJson(String),
    EmptyData,
    InvalidRecord { index: usize, reason: &'static str },
    DuplicateTimestamp { index: usize },
    NonHourlyTimestamp { index: usize },
}

impl Display for EphemerisError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(reason) => {
                write!(formatter, "invalid NASA lunar ephemeris JSON: {reason}")
            }
            Self::EmptyData => formatter.write_str("NASA lunar ephemeris has no hourly records"),
            Self::InvalidRecord { index, reason } => {
                write!(
                    formatter,
                    "invalid NASA lunar ephemeris record {index}: {reason}"
                )
            }
            Self::DuplicateTimestamp { index } => {
                write!(
                    formatter,
                    "duplicate NASA lunar ephemeris timestamp at record {index}"
                )
            }
            Self::NonHourlyTimestamp { index } => {
                write!(
                    formatter,
                    "NASA lunar ephemeris record {index} is not the next hourly sample"
                )
            }
        }
    }
}

impl Error for EphemerisError {}

#[derive(Deserialize)]
struct NasaRecord {
    time: String,
    subsolar: NasaCoordinates,
    subearth: NasaCoordinates,
    posangle: f64,
}

#[derive(Deserialize)]
struct NasaCoordinates {
    lon: f64,
    lat: f64,
}

fn parse_timestamp(source: &str) -> Result<i64, &'static str> {
    let format =
        format_description!("[day padding:zero] [month repr:short] [year] [hour]:[minute] UT");
    let timestamp = PrimitiveDateTime::parse(source, format)
        .map_err(|_| "time must use the NASA `DD Mon YYYY HH:00 UT` format")?;
    if timestamp.minute() != 0 {
        return Err("time must fall on an exact hour");
    }

    Ok(timestamp.assume_utc().unix_timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPOCH: AstronomicalInstant = AstronomicalInstant::from_unix_seconds(1_767_225_600);

    fn two_sample_json(first_longitude: f64, second_longitude: f64) -> Vec<u8> {
        format!(
            r#"[
                {{"time":"01 Jan 2026 00:00 UT","subsolar":{{"lon":{first_longitude},"lat":-2.0}},"subearth":{{"lon":-10.0,"lat":-4.0}},"posangle":350.0}},
                {{"time":"01 Jan 2026 01:00 UT","subsolar":{{"lon":{second_longitude},"lat":2.0}},"subearth":{{"lon":10.0,"lat":4.0}},"posangle":10.0}}
            ]"#
        )
        .into_bytes()
    }

    fn test_sample(
        subsolar_longitude: f64,
        subsolar_latitude: f64,
        subearth_longitude: f64,
        subearth_latitude: f64,
        position_angle: f64,
    ) -> EphemerisSample {
        EphemerisSample {
            subsolar_point: LunarCoordinates::new(subsolar_longitude, subsolar_latitude)
                .expect("coordinates should be valid"),
            subearth_point: LunarCoordinates::new(subearth_longitude, subearth_latitude)
                .expect("coordinates should be valid"),
            position_angle: LunarPositionAngle::new(position_angle)
                .expect("position angle should be valid"),
        }
    }

    /// A sampled sub-Earth point is rotated to the center of the visible disk.
    #[test]
    fn centers_subearth_point_toward_the_camera() {
        let subearth_point =
            LunarCoordinates::new(30.0, 10.0).expect("coordinates should be valid");
        let sample = EphemerisSample {
            subsolar_point: LunarCoordinates::new(0.0, 0.0).expect("coordinates should be valid"),
            subearth_point,
            position_angle: LunarPositionAngle::new(45.0).expect("position angle should be valid"),
        };

        let centered = sample
            .object_to_world()
            .transform_vector3(subearth_point.globe_location());

        assert!(centered.abs_diff_eq(Vec3::NEG_Z, 1.0e-6));
    }

    /// Sub-Earth centering keeps the projected lunar north axis upright.
    #[test]
    fn keeps_lunar_north_upright_while_centering_subearth_point() {
        let sample = EphemerisSample {
            subsolar_point: LunarCoordinates::new(0.0, 0.0).expect("coordinates should be valid"),
            subearth_point: LunarCoordinates::new(-30.0, -10.0)
                .expect("coordinates should be valid"),
            position_angle: LunarPositionAngle::new(0.0).expect("position angle should be valid"),
        };

        let lunar_north = sample.object_to_world().transform_vector3(Vec3::Y);

        assert!(lunar_north.x.abs() < 1.0e-6);
        assert!(lunar_north.y > 0.0);
    }

    /// Positive position angle rolls lunar north counterclockwise from framebuffer up.
    #[test]
    fn positive_position_angle_rolls_lunar_north_counterclockwise() {
        let sample = EphemerisSample {
            subsolar_point: LunarCoordinates::new(0.0, 0.0).expect("coordinates should be valid"),
            subearth_point: LunarCoordinates::new(0.0, 0.0).expect("coordinates should be valid"),
            position_angle: LunarPositionAngle::new(90.0).expect("position angle should be valid"),
        };

        let lunar_north = sample.object_to_world().transform_vector3(Vec3::Y);

        assert!(lunar_north.abs_diff_eq(Vec3::NEG_X, 1.0e-6));
    }

    /// Sun direction and globe pose use the same sampled lunar orientation.
    #[test]
    fn transforms_subsolar_direction_with_lunar_globe_pose() {
        let coordinates = LunarCoordinates::new(30.0, 10.0).expect("coordinates should be valid");
        let sample = EphemerisSample {
            subsolar_point: coordinates,
            subearth_point: coordinates,
            position_angle: LunarPositionAngle::new(45.0).expect("position angle should be valid"),
        };

        let appearance = sample.lunar_appearance();

        assert!(
            appearance
                .sun_direction()
                .as_vec3()
                .abs_diff_eq(Vec3::NEG_Z, 1.0e-6)
        );
    }

    /// An exact hourly instant returns the complete matching NASA record unchanged.
    #[test]
    fn samples_exact_hourly_record() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.add(SECONDS_PER_HOUR as f64);
        let expected = test_sample(20.0, 2.0, 10.0, 4.0, 10.0);

        let sample = ephemeris
            .nearest_sample_at(instant)
            .expect("hour should be covered");

        assert_eq!(sample, expected);
    }

    /// An instant before the half-hour boundary uses the earlier hourly record unchanged.
    #[test]
    fn samples_earlier_record_before_half_hour() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.add(29.0 * 60.0);
        let expected = test_sample(30.0, -2.0, -10.0, -4.0, 350.0);

        let sample = ephemeris
            .nearest_sample_at(instant)
            .expect("instant should be covered");

        assert_eq!(sample, expected);
    }

    /// An instant after the half-hour boundary uses the later hourly record unchanged.
    #[test]
    fn samples_later_record_after_half_hour() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.add(31.0 * 60.0);
        let expected = test_sample(20.0, 2.0, 10.0, 4.0, 10.0);

        let sample = ephemeris
            .nearest_sample_at(instant)
            .expect("instant should be covered");

        assert_eq!(sample, expected);
    }

    /// An instant exactly between records resolves deterministically to the later record.
    #[test]
    fn resolves_half_hour_tie_to_later_record() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.add(SECONDS_PER_HOUR as f64 / 2.0);
        let expected = test_sample(20.0, 2.0, 10.0, 4.0, 10.0);

        let sample = ephemeris
            .nearest_sample_at(instant)
            .expect("half hour should be covered");

        assert_eq!(sample, expected);
    }

    /// Out-of-range sub-Earth coordinates are rejected during ephemeris construction.
    #[test]
    fn rejects_invalid_subearth_coordinates() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":181.0,"lat":0.0},"posangle":0.0},
            {"time":"01 Jan 2026 01:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(
            result,
            Err(EphemerisError::InvalidRecord { index: 0, reason })
                if reason.contains("lunar longitude")
        ));
    }

    /// Lunar position angle rejects non-finite and out-of-range degrees.
    #[test]
    fn position_angle_rejects_invalid_degrees() {
        let below_range = -1.0;
        let upper_bound = 360.0;
        let non_finite = f64::NAN;

        let below_range_result = LunarPositionAngle::new(below_range);
        let upper_bound_result = LunarPositionAngle::new(upper_bound);
        let non_finite_result = LunarPositionAngle::new(non_finite);

        assert!(below_range_result.is_err());
        assert!(upper_bound_result.is_err());
        assert!(non_finite_result.is_err());
    }

    /// Out-of-range lunar position angles are rejected during ephemeris construction.
    #[test]
    fn rejects_invalid_position_angle() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":360.0},
            {"time":"01 Jan 2026 01:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(
            result,
            Err(EphemerisError::InvalidRecord { index: 0, reason })
                if reason.contains("position angle")
        ));
    }

    /// Malformed JSON is rejected before ephemeris records are constructed.
    #[test]
    fn rejects_malformed_json() {
        let source = br#"not json"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(result, Err(EphemerisError::InvalidJson(_))));
    }

    /// One complete hourly record is valid ephemeris data for nearest-record sampling.
    #[test]
    fn accepts_single_record() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(result.is_ok());
    }

    /// An empty record list is rejected because it cannot be sampled.
    #[test]
    fn rejects_empty_data() {
        let source = br#"[]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(result, Err(EphemerisError::EmptyData)));
    }

    /// Adjacent records with the same timestamp are rejected as duplicates.
    #[test]
    fn rejects_duplicate_timestamps() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0},
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":1.0,"lat":1.0},"subearth":{"lon":1.0,"lat":1.0},"posangle":0.0}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(
            result,
            Err(EphemerisError::DuplicateTimestamp { index: 1 })
        ));
    }

    /// A gap between adjacent hourly records is rejected.
    #[test]
    fn rejects_non_hourly_timestamps() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0},
            {"time":"01 Jan 2026 02:00 UT","subsolar":{"lon":1.0,"lat":1.0},"subearth":{"lon":1.0,"lat":1.0},"posangle":0.0}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(
            result,
            Err(EphemerisError::NonHourlyTimestamp { index: 1 })
        ));
    }
}
