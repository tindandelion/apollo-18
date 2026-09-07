use crate::lunar_ephemeris::AstronomicalInstant;
use crate::{LunarAppearance, LunarEphemeris, SceneTime, SunDirection};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

const ANIMATION_PERIOD_SECONDS: f64 = 10.0;
const MEAN_SYNODIC_MONTH_DAYS: f64 = 29.530_588_853;
const HOURS_PER_DAY: f64 = 24.0;
const SECONDS_PER_HOUR: f64 = 3_600.0;

pub const CANONICAL_ANIMATION_EPOCH: AstronomicalInstant =
    AstronomicalInstant::from_unix_seconds(1_767_225_600);

#[derive(Debug, Clone)]
pub struct LunarPhaseAnimation {
    ephemeris: LunarEphemeris,
    animation_epoch: AstronomicalInstant,
}

impl LunarPhaseAnimation {
    pub fn new(
        ephemeris: LunarEphemeris,
        animation_epoch: AstronomicalInstant,
    ) -> Result<Self, AnimationCoverageError> {
        let end = animation_epoch.add(MEAN_SYNODIC_MONTH_DAYS * HOURS_PER_DAY * SECONDS_PER_HOUR);
        if !ephemeris.covers(animation_epoch, end) {
            return Err(AnimationCoverageError);
        }

        Ok(Self {
            ephemeris,
            animation_epoch,
        })
    }

    pub fn lunar_appearance(&self, scene_time: SceneTime) -> LunarAppearance {
        let cycle_fraction = scene_time.cycle_fraction(ANIMATION_PERIOD_SECONDS);
        let astronomy_seconds =
            cycle_fraction * MEAN_SYNODIC_MONTH_DAYS * HOURS_PER_DAY * SECONDS_PER_HOUR;
        let instant = self.animation_epoch.add(astronomy_seconds);
        let sample = self
            .ephemeris
            .sample_at(instant)
            .expect("validated ephemeris coverage includes every animation instant");

        let object_to_world = sample.object_to_world();
        let sun_direction = SunDirection::new(sample.sun_direction())
            .expect("validated ephemeris coordinates produce a valid Sun direction");

        LunarAppearance::with_object_to_world(object_to_world, sun_direction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationCoverageError;

impl Display for AnimationCoverageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("NASA lunar ephemeris does not cover the complete animation cycle")
    }
}

impl Error for AnimationCoverageError {}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene_time_for_astronomy_hours(hours: f64) -> SceneTime {
        SceneTime::from_seconds(
            hours / (MEAN_SYNODIC_MONTH_DAYS * HOURS_PER_DAY) * ANIMATION_PERIOD_SECONDS,
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
                r#"{{"time":"{day:02} Jan 2026 {hour_of_day:02}:00 UT","subsolar":{{"lon":{longitude},"lat":{latitude}}},"subearth":{{"lon":0.0,"lat":0.0}}}}"#
            ));
        }
        format!("[{}]", records.join(",")).into_bytes()
    }

    fn canonical_animation() -> LunarPhaseAnimation {
        let source = include_bytes!("../../../assets/nasa/mooninfo_2026.json");
        let ephemeris =
            LunarEphemeris::from_nasa_json(source).expect("NASA source should be valid");
        LunarPhaseAnimation::new(ephemeris, CANONICAL_ANIMATION_EPOCH)
            .expect("NASA source should cover the canonical animation")
    }

    /// One mapped astronomy hour samples NASA's matching hourly subsolar point.
    #[test]
    fn maps_scene_time_to_astronomical_time() {
        let animation = canonical_animation();
        let scene_time = scene_time_for_astronomy_hours(1.0);
        let expected = glam::Vec3::new(0.547_631_14, 0.072_245_15, -0.833_595_1);

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

    /// A subsolar point becomes a world-space Sun direction in lunar globe coordinates.
    #[test]
    fn derives_sun_direction_from_subsolar_point() {
        let source = covered_json(0.0, 0.0);
        let ephemeris = LunarEphemeris::from_nasa_json(&source).expect("source should be valid");
        let animation = LunarPhaseAnimation::new(ephemeris, CANONICAL_ANIMATION_EPOCH)
            .expect("source should cover the animation");
        let scene_time = scene_time_for_astronomy_hours(0.5);

        let appearance = animation.lunar_appearance(scene_time);
        let direction = appearance.sun_direction().as_vec3();

        assert!(direction.abs_diff_eq(glam::Vec3::NEG_Z, 1.0e-6));
    }

    /// Animation construction rejects ephemeris data that cannot cover the complete cycle.
    #[test]
    fn rejects_incomplete_animation_coverage() {
        let source = br#"[
            {"time":"01 Jan 2026 00:00 UT","subsolar":{"lon":0.0,"lat":0.0},"subearth":{"lon":0.0,"lat":0.0}},
            {"time":"01 Jan 2026 01:00 UT","subsolar":{"lon":1.0,"lat":1.0},"subearth":{"lon":0.0,"lat":0.0}}
        ]"#;
        let ephemeris = LunarEphemeris::from_nasa_json(source).expect("source should be valid");

        let result = LunarPhaseAnimation::new(ephemeris, CANONICAL_ANIMATION_EPOCH);

        assert!(matches!(result, Err(AnimationCoverageError)));
    }
}
