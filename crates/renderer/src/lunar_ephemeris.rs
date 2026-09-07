use crate::SunDirection;
use glam::Vec3;
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

    pub(crate) const fn seconds_after(self, seconds: f64) -> Self {
        Self(self.0 + seconds)
    }
}

#[derive(Debug, Clone)]
pub struct LunarEphemeris {
    first_timestamp: i64,
    samples: Vec<SubsolarPoint>,
}

impl LunarEphemeris {
    pub fn from_nasa_json(source: &[u8]) -> Result<Self, EphemerisError> {
        let records: Vec<NasaRecord> = serde_json::from_slice(source)
            .map_err(|error| EphemerisError::InvalidJson(error.to_string()))?;
        if records.len() < 2 {
            return Err(EphemerisError::IncompleteData);
        }

        let mut timestamps = Vec::with_capacity(records.len());
        let mut samples = Vec::with_capacity(records.len());
        for (index, record) in records.into_iter().enumerate() {
            let timestamp = parse_timestamp(&record.time)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            let point = SubsolarPoint::new(record.subsolar.lon, record.subsolar.lat)
                .map_err(|reason| EphemerisError::InvalidRecord { index, reason })?;
            timestamps.push(timestamp);
            samples.push(point);
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

    pub(crate) fn subsolar_point_at(&self, instant: AstronomicalInstant) -> Option<SubsolarPoint> {
        let target_hours = (instant.0 - self.first_timestamp as f64) / SECONDS_PER_HOUR as f64;
        if target_hours < 0.0 {
            return None;
        }

        let rounded_hours = target_hours.round();
        let target_hours = if (target_hours - rounded_hours).abs() < 1.0e-9 {
            rounded_hours
        } else {
            target_hours
        };
        let lower_index = target_hours.floor() as usize;
        let fraction = target_hours - lower_index as f64;
        let lower = self.samples.get(lower_index).copied()?;
        if fraction == 0.0 {
            return Some(lower);
        }
        let upper = self.samples.get(lower_index + 1).copied()?;

        Some(lower.interpolate(upper, fraction))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SubsolarPoint {
    longitude_degrees: f64,
    latitude_degrees: f64,
}

impl SubsolarPoint {
    pub(crate) fn new(longitude_degrees: f64, latitude_degrees: f64) -> Result<Self, &'static str> {
        if !longitude_degrees.is_finite() || !(-180.0..=180.0).contains(&longitude_degrees) {
            return Err("subsolar longitude must be finite and between -180 and 180 degrees");
        }
        if !latitude_degrees.is_finite() || !(-90.0..=90.0).contains(&latitude_degrees) {
            return Err("subsolar latitude must be finite and between -90 and 90 degrees");
        }

        Ok(Self {
            longitude_degrees,
            latitude_degrees,
        })
    }

    fn interpolate(self, other: Self, fraction: f64) -> Self {
        let longitude_delta =
            (other.longitude_degrees - self.longitude_degrees + 180.0).rem_euclid(360.0) - 180.0;
        let longitude_degrees =
            (self.longitude_degrees + longitude_delta * fraction + 180.0).rem_euclid(360.0) - 180.0;

        Self {
            longitude_degrees,
            latitude_degrees: self.latitude_degrees
                + (other.latitude_degrees - self.latitude_degrees) * fraction,
        }
    }

    pub(crate) fn sun_direction(self) -> SunDirection {
        let longitude = self.longitude_degrees.to_radians();
        let latitude = self.latitude_degrees.to_radians();
        let horizontal_radius = latitude.cos();
        let direction = Vec3::new(
            (horizontal_radius * longitude.sin()) as f32,
            latitude.sin() as f32,
            (-horizontal_radius * longitude.cos()) as f32,
        );

        SunDirection::new(direction)
            .expect("a finite subsolar longitude and latitude produce a valid Sun direction")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EphemerisError {
    InvalidJson(String),
    IncompleteData,
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
            Self::IncompleteData => {
                formatter.write_str("NASA lunar ephemeris needs at least two hourly records")
            }
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
    subsolar: NasaSubsolarPoint,
}

#[derive(Deserialize)]
struct NasaSubsolarPoint {
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
                {{"time":"01 Jan 2026 00:00 UT","subsolar":{{"lon":{first_longitude},"lat":-2.0}}}},
                {{"time":"01 Jan 2026 01:00 UT","subsolar":{{"lon":{second_longitude},"lat":2.0}}}}
            ]"#
        )
        .into_bytes()
    }

    /// NASA's exact hourly subsolar coordinates are returned without interpolation.
    #[test]
    fn samples_exact_hourly_subsolar_points() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.seconds_after(SECONDS_PER_HOUR as f64);

        let point = ephemeris
            .subsolar_point_at(instant)
            .expect("hour should be covered");

        assert!((point.longitude_degrees - 20.0).abs() < 1.0e-9);
        assert!((point.latitude_degrees - 2.0).abs() < 1.0e-9);
    }

    /// Latitude and longitude are interpolated halfway between adjacent hourly samples.
    #[test]
    fn interpolates_between_hourly_subsolar_points() {
        let source = two_sample_json(30.0, 20.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.seconds_after(SECONDS_PER_HOUR as f64 / 2.0);

        let point = ephemeris
            .subsolar_point_at(instant)
            .expect("half hour should be covered");

        assert!((point.longitude_degrees - 25.0).abs() < 1.0e-9);
        assert!(point.latitude_degrees.abs() < 1.0e-9);
    }

    /// Longitude interpolation follows the short path across the antimeridian.
    #[test]
    fn interpolates_longitude_across_its_wrap_boundary() {
        let source = two_sample_json(179.0, -179.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let instant = EPOCH.seconds_after(SECONDS_PER_HOUR as f64 / 2.0);

        let point = ephemeris
            .subsolar_point_at(instant)
            .expect("half hour should be covered");

        assert!((point.longitude_degrees.abs() - 180.0).abs() < 1.0e-9);
    }

    /// Malformed JSON is rejected before ephemeris records are constructed.
    #[test]
    fn rejects_malformed_json() {
        let source = br#"not json"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(result, Err(EphemerisError::InvalidJson(_))));
    }

    /// Fewer than two hourly records are rejected as incomplete ephemeris data.
    #[test]
    fn rejects_incomplete_data() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0}}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(result, Err(EphemerisError::IncompleteData)));
    }

    /// Adjacent records with the same timestamp are rejected as duplicates.
    #[test]
    fn rejects_duplicate_timestamps() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0}},
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":1.0,"lat":1.0}}
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
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0}},
            {"time":"01 Jan 2026 02:00 UT","subsolar":{"lon":1.0,"lat":1.0}}
        ]"#;

        let result = LunarEphemeris::from_nasa_json(source);

        assert!(matches!(
            result,
            Err(EphemerisError::NonHourlyTimestamp { index: 1 })
        ));
    }
}
