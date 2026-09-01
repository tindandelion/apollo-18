use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneTime(f64);

impl SceneTime {
    pub fn from_seconds(seconds: f64) -> Result<Self, InvalidSceneTime> {
        if seconds.is_finite() && seconds >= 0.0 {
            Ok(Self(seconds))
        } else {
            Err(InvalidSceneTime)
        }
    }

    pub fn for_frame(frame_index: u32, frames_per_second: NonZeroU32) -> Self {
        Self(f64::from(frame_index) / f64::from(frames_per_second.get()))
    }

    pub fn from_elapsed_millis(
        start_millis: f64,
        current_millis: f64,
    ) -> Result<Self, InvalidSceneTime> {
        Self::from_seconds((current_millis - start_millis) / 1000.0)
    }

    pub fn as_seconds(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSceneTime;

impl Display for InvalidSceneTime {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("scene time must be non-negative and finite")
    }
}

impl Error for InvalidSceneTime {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_non_negative_finite_seconds() {
        let zero = SceneTime::from_seconds(0.0).expect("zero seconds should be valid");
        let later = SceneTime::from_seconds(2.5).expect("positive seconds should be valid");

        assert_eq!(zero.as_seconds(), 0.0);
        assert_eq!(later.as_seconds(), 2.5);
    }

    #[test]
    fn derives_scene_time_from_frame_index_and_rate() {
        let frames_per_second = NonZeroU32::new(24).expect("frame rate should be nonzero");
        let scene_time = SceneTime::for_frame(60, frames_per_second);

        assert_eq!(scene_time.as_seconds(), 2.5);
    }

    #[test]
    fn derives_scene_time_from_elapsed_milliseconds() {
        let scene_time = SceneTime::from_elapsed_millis(1_250.0, 3_750.0)
            .expect("monotonic millisecond timestamps should be valid");

        assert_eq!(scene_time.as_seconds(), 2.5);
        assert!(SceneTime::from_elapsed_millis(2.0, 1.0).is_err());
        assert!(SceneTime::from_elapsed_millis(f64::NAN, 1.0).is_err());
    }

    #[test]
    fn rejects_negative_and_non_finite_seconds() {
        for seconds in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = SceneTime::from_seconds(seconds).expect_err("scene time should be invalid");

            assert_eq!(
                error.to_string(),
                "scene time must be non-negative and finite"
            );
        }
    }
}
