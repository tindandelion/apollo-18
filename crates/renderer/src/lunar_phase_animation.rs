use crate::lunar_ephemeris::AstronomicalInstant;
use crate::{LunarAppearance, LunarEphemeris, SceneTime};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const SYNODIC_MONTH_PERIOD_SECONDS: f64 = 10.0;
const EPHEMERIS_SPAN_PERIOD_SECONDS: f64 = 120.0;
const MEAN_SYNODIC_MONTH_DAYS: f64 = 29.530_588_853;
const HOURS_PER_DAY: f64 = 24.0;
const SECONDS_PER_HOUR: f64 = 3_600.0;
const MEAN_SYNODIC_MONTH_SECONDS: f64 = MEAN_SYNODIC_MONTH_DAYS * HOURS_PER_DAY * SECONDS_PER_HOUR;

#[derive(Debug, Clone)]
pub struct SynodicMonthAnimation {
    ephemeris: LunarEphemeris,
    animation_epoch: AstronomicalInstant,
}

impl SynodicMonthAnimation {
    pub fn new(ephemeris: LunarEphemeris) -> Result<Self, AnimationCoverageError> {
        let animation_epoch = ephemeris.first_instant();
        let required_end = animation_epoch.add(MEAN_SYNODIC_MONTH_SECONDS);
        if !ephemeris.covers(animation_epoch, required_end) {
            return Err(AnimationCoverageError);
        }

        Ok(Self {
            ephemeris,
            animation_epoch,
        })
    }

    pub fn lunar_appearance(&self, scene_time: SceneTime) -> LunarAppearance {
        let cycle_fraction = scene_time.cycle_fraction(SYNODIC_MONTH_PERIOD_SECONDS);
        let instant = self
            .animation_epoch
            .add(cycle_fraction * MEAN_SYNODIC_MONTH_SECONDS);
        self.ephemeris
            .lunar_appearance_at(instant)
            .expect("validated ephemeris coverage includes every animation instant")
    }
}

#[derive(Debug, Clone)]
pub struct EphemerisSpanAnimation {
    ephemeris: LunarEphemeris,
    animation_epoch: AstronomicalInstant,
    span_seconds: f64,
}

impl EphemerisSpanAnimation {
    pub fn new(ephemeris: LunarEphemeris) -> Self {
        let animation_epoch = ephemeris.first_instant();
        let span_seconds = ephemeris.last_instant().seconds_since(animation_epoch);

        Self {
            ephemeris,
            animation_epoch,
            span_seconds,
        }
    }

    pub fn lunar_appearance(&self, scene_time: SceneTime) -> LunarAppearance {
        let cycle_fraction = scene_time.cycle_fraction(EPHEMERIS_SPAN_PERIOD_SECONDS);
        let instant = self.animation_epoch.add(cycle_fraction * self.span_seconds);
        self.ephemeris
            .lunar_appearance_at(instant)
            .expect("ephemeris-span animation remains inside validated timestamps")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationCoverageError;

impl Display for AnimationCoverageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "NASA lunar ephemeris must cover one complete mean synodic month from its first timestamp",
        )
    }
}

impl Error for AnimationCoverageError {}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene_time_for_astronomy_hours(hours: f64) -> SceneTime {
        SceneTime::from_seconds(
            hours / (MEAN_SYNODIC_MONTH_DAYS * HOURS_PER_DAY) * SYNODIC_MONTH_PERIOD_SECONDS,
        )
        .expect("scene time should be valid")
    }

    fn covered_json(first_longitude: f64, second_longitude: f64) -> Vec<u8> {
        let mut records = Vec::new();
        for hour in 0..=709 {
            let day = hour / 24 + 1;
            let hour_of_day = hour % 24;
            let (longitude, latitude) = if hour == 0 {
                (first_longitude, -2.0)
            } else {
                (second_longitude, 2.0)
            };
            records.push(format!(
                r#"{{"time":"{day:02} Jan 2026 {hour_of_day:02}:00 UT","subsolar":{{"lon":{longitude},"lat":{latitude}}},"subearth":{{"lon":0.0,"lat":0.0}},"posangle":0.0}}"#
            ));
        }
        format!("[{}]", records.join(",")).into_bytes()
    }

    fn canonical_animation() -> SynodicMonthAnimation {
        let source = include_bytes!("../../../assets/nasa/mooninfo_2026.json");
        let ephemeris =
            LunarEphemeris::from_nasa_json(source).expect("NASA source should be valid");
        SynodicMonthAnimation::new(ephemeris)
            .expect("NASA source should cover the canonical animation")
    }

    /// One mapped astronomy hour samples NASA's matching hourly subsolar point.
    #[test]
    fn maps_scene_time_to_astronomical_time() {
        let animation = canonical_animation();
        let scene_time = scene_time_for_astronomy_hours(1.0);
        let expected = glam::Vec3::new(0.551_907_2, -0.022_753_18, -0.833_595_1);

        let appearance = animation.lunar_appearance(scene_time);

        assert!(
            appearance
                .sun_direction()
                .as_vec3()
                .abs_diff_eq(expected, 1.0e-6)
        );
    }

    /// Ten scene seconds advance one mean synodic month and then reset to the animation epoch.
    #[test]
    fn resets_after_one_mean_synodic_month() {
        let animation = canonical_animation();
        let start = SceneTime::from_seconds(0.0).expect("scene time should be valid");
        let reset = SceneTime::from_seconds(10.0).expect("scene time should be valid");

        let start_appearance = animation.lunar_appearance(start);
        let reset_appearance = animation.lunar_appearance(reset);

        assert_eq!(reset_appearance, start_appearance);
    }

    /// Appearance sampling is deterministic when scene times are requested out of order.
    #[test]
    fn samples_deterministically_out_of_order() {
        let animation = canonical_animation();
        let earlier = SceneTime::from_seconds(1.25).expect("scene time should be valid");
        let later = SceneTime::from_seconds(7.5).expect("scene time should be valid");

        let first_earlier = animation.lunar_appearance(earlier);
        let later_appearance = animation.lunar_appearance(later);
        let second_earlier = animation.lunar_appearance(earlier);

        assert_eq!(second_earlier, first_earlier);
        assert_ne!(later_appearance, first_earlier);
    }

    /// A half-hour animation instant derives Sun direction from the later nearest record.
    #[test]
    fn half_hour_uses_later_subsolar_point() {
        let source = covered_json(0.0, 0.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let animation =
            SynodicMonthAnimation::new(ephemeris).expect("source should cover the animation");
        let scene_time = scene_time_for_astronomy_hours(0.5);
        let latitude = 2.0_f32.to_radians();
        let expected = glam::Vec3::new(0.0, latitude.sin(), -latitude.cos());

        let appearance = animation.lunar_appearance(scene_time);
        let direction = appearance.sun_direction().as_vec3();

        assert!(direction.abs_diff_eq(expected, 1.0e-6));
    }

    /// Coverage must reach the first hourly record after one complete mean synodic month.
    #[test]
    fn enforces_synodic_month_coverage_boundary() {
        let covered = covered_json(0.0, 1.0);
        let mut insufficient_records = serde_json::from_slice::<Vec<serde_json::Value>>(&covered)
            .expect("fixture should decode");
        insufficient_records.pop();
        let insufficient =
            serde_json::to_vec(&insufficient_records).expect("fixture should encode");
        let covered_ephemeris =
            LunarEphemeris::from_nasa_json(&covered).expect("source should be valid");
        let insufficient_ephemeris =
            LunarEphemeris::from_nasa_json(&insufficient).expect("source should be valid");

        let covered_result = SynodicMonthAnimation::new(covered_ephemeris);
        let insufficient_result = SynodicMonthAnimation::new(insufficient_ephemeris);

        assert!(covered_result.is_ok());
        assert!(matches!(insufficient_result, Err(AnimationCoverageError)));
    }

    /// Animation construction rejects ephemeris data that cannot cover the complete cycle.
    #[test]
    fn rejects_incomplete_animation_coverage() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0},
            {"time":"01 Jan 2026 01:00 UT","subsolar":{"lon":1.0,"lat":1.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0}
        ]"#;
        let ephemeris = LunarEphemeris::from_nasa_json(source).expect("source should be valid");

        let result = SynodicMonthAnimation::new(ephemeris);

        assert!(matches!(result, Err(AnimationCoverageError)));
    }

    /// The native animation derives its epoch from the first validated record.
    #[test]
    fn synodic_month_starts_at_first_validated_record() {
        let source = covered_json(0.0, 90.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let expected = ephemeris
            .lunar_appearance_at(ephemeris.first_instant())
            .expect("first record should be available");
        let animation =
            SynodicMonthAnimation::new(ephemeris).expect("source should cover the animation");
        let start = SceneTime::from_seconds(0.0).expect("scene time should be valid");

        let appearance = animation.lunar_appearance(start);

        assert_eq!(appearance, expected);
    }

    /// The web animation maps one cycle across any complete validated ephemeris span.
    #[test]
    fn ephemeris_span_maps_complete_source_to_two_minutes() {
        let source = three_sample_json();
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let expected_start = ephemeris
            .lunar_appearance_at(ephemeris.first_instant())
            .expect("first record should be available");
        let expected_middle = ephemeris
            .lunar_appearance_at(ephemeris.first_instant().add(SECONDS_PER_HOUR))
            .expect("middle record should be available");
        let expected_end = ephemeris
            .lunar_appearance_at(ephemeris.last_instant())
            .expect("last record should be available");
        let animation = EphemerisSpanAnimation::new(ephemeris);

        let start = animation.lunar_appearance(scene_time(0.0));
        let middle = animation.lunar_appearance(scene_time(60.0));
        let near_end = animation.lunar_appearance(scene_time(119.999));

        assert_eq!(start, expected_start);
        assert_eq!(middle, expected_middle);
        assert_eq!(near_end, expected_end);
    }

    /// Nearest sampling gives endpoint records half the web display interval of interior records.
    #[test]
    fn ephemeris_span_gives_endpoints_half_intervals() {
        let ephemeris =
            LunarEphemeris::from_nasa_json(&three_sample_json()).expect("source should be valid");
        let first = ephemeris
            .lunar_appearance_at(ephemeris.first_instant())
            .expect("first record should be available");
        let middle = ephemeris
            .lunar_appearance_at(ephemeris.first_instant().add(SECONDS_PER_HOUR))
            .expect("middle record should be available");
        let last = ephemeris
            .lunar_appearance_at(ephemeris.last_instant())
            .expect("last record should be available");
        let animation = EphemerisSpanAnimation::new(ephemeris);

        let before_first_tie = animation.lunar_appearance(scene_time(29.999));
        let first_tie = animation.lunar_appearance(scene_time(30.0));
        let before_last_tie = animation.lunar_appearance(scene_time(89.999));
        let last_tie = animation.lunar_appearance(scene_time(90.0));

        assert_eq!(before_first_tie, first);
        assert_eq!(first_tie, middle);
        assert_eq!(before_last_tie, middle);
        assert_eq!(last_tie, last);
    }

    /// The web cycle resets directly to the first record at two minutes.
    #[test]
    fn ephemeris_span_resets_at_two_minutes() {
        let ephemeris =
            LunarEphemeris::from_nasa_json(&three_sample_json()).expect("source should be valid");
        let animation = EphemerisSpanAnimation::new(ephemeris);

        let start = animation.lunar_appearance(scene_time(0.0));
        let reset = animation.lunar_appearance(scene_time(120.0));

        assert_eq!(reset, start);
    }

    fn three_sample_json() -> Vec<u8> {
        br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0},"posangle":0.0},
            {"time":"01 Jan 2026 01:00 UT","subsolar":{"lon":45.0,"lat":0.0},"subearth":{"lon":5.0,"lat":0.0},"posangle":5.0},
            {"time":"01 Jan 2026 02:00 UT","subsolar":{"lon":90.0,"lat":0.0},"subearth":{"lon":10.0,"lat":0.0},"posangle":10.0}
        ]"#.to_vec()
    }

    fn scene_time(seconds: f64) -> SceneTime {
        SceneTime::from_seconds(seconds).expect("scene time should be valid")
    }
}
