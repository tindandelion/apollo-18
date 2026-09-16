mod bundled_assets;
mod error;
mod globe_web_page;

use apollo18_renderer::{
    Framebuffer, SceneTime,
    lunar_globe::{EphemerisSpanAnimation, LunarColorMap, LunarElevationMap, render_lunar_globe},
};
use bundled_assets::BundledAssets;
use error::Error;
use globe_web_page::{FrameRequest, GlobeWebPage};
use std::fmt;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;

const EPHEMERIS_SPAN_PERIOD: Duration = Duration::from_secs(240);
const FRAME_RELATIVE_RADIUS: f32 = 0.5;
const MAX_BACKING_DIMENSION: u32 = 1024;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let page = Rc::new(GlobeWebPage::discover()?);
    let assets = BundledAssets::load()?;
    let lunar_phase_animation =
        EphemerisSpanAnimation::new(assets.ephemeris, EPHEMERIS_SPAN_PERIOD).map_err(|error| {
            Error::with_context("could not initialize the ephemeris-span animation", error)
        })?;

    let mut animation = CanvasAnimation::new(
        assets.color_map,
        assets.elevation_map,
        lunar_phase_animation,
    );
    if let Err(error) = page.start_presenting(move |request| animation.render(request)) {
        page.show_initialization_failure(&error)?;
    }

    Ok(())
}

struct CanvasAnimation {
    color_map: LunarColorMap,
    elevation_map: LunarElevationMap,
    lunar_phase_animation: EphemerisSpanAnimation,
    started_at_milliseconds: Option<f64>,
}

impl CanvasAnimation {
    fn new(
        color_map: LunarColorMap,
        elevation_map: LunarElevationMap,
        lunar_phase_animation: EphemerisSpanAnimation,
    ) -> Self {
        Self {
            color_map,
            elevation_map,
            lunar_phase_animation,
            started_at_milliseconds: None,
        }
    }

    fn render(&mut self, request: FrameRequest) -> Result<Framebuffer, Error> {
        let resolution = select_backing_resolution(
            request.canvas_css_width(),
            request.canvas_css_height(),
            request.device_pixel_ratio(),
        )
        .map_err(|error| {
            Error::with_context("could not select a canvas backing resolution", error)
        })?;

        let scene_time = scene_time_from_timestamp(
            &mut self.started_at_milliseconds,
            request.timestamp_milliseconds(),
        )
        .map_err(|error| Error::with_context("could not derive scene time", error))?;
        let appearance = self.lunar_phase_animation.lunar_appearance(scene_time);
        let frame = render_lunar_globe(
            resolution.width,
            resolution.height,
            FRAME_RELATIVE_RADIUS,
            appearance,
            &self.color_map,
            &self.elevation_map,
        )
        .map_err(|error| Error::with_context("could not render the lunar globe", error))?;

        Ok(frame)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BackingResolution {
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResolutionError {
    InvalidCssWidth(f64),
    InvalidCssHeight(f64),
    InvalidDevicePixelRatio(f64),
    UnrepresentableDesiredResolution,
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCssWidth(width) => write!(formatter, "invalid canvas CSS width: {width}"),
            Self::InvalidCssHeight(height) => {
                write!(formatter, "invalid canvas CSS height: {height}")
            }
            Self::InvalidDevicePixelRatio(ratio) => {
                write!(formatter, "invalid device pixel ratio: {ratio}")
            }
            Self::UnrepresentableDesiredResolution => {
                formatter.write_str("desired canvas backing resolution is not finite")
            }
        }
    }
}

fn select_backing_resolution(
    css_width: f64,
    css_height: f64,
    device_pixel_ratio: f64,
) -> Result<BackingResolution, ResolutionError> {
    if !css_width.is_finite() || css_width < 0.0 {
        return Err(ResolutionError::InvalidCssWidth(css_width));
    }
    if !css_height.is_finite() || css_height < 0.0 {
        return Err(ResolutionError::InvalidCssHeight(css_height));
    }
    if !device_pixel_ratio.is_finite() || device_pixel_ratio <= 0.0 {
        return Err(ResolutionError::InvalidDevicePixelRatio(device_pixel_ratio));
    }
    let desired_width = css_width * device_pixel_ratio;
    let desired_height = css_height * device_pixel_ratio;
    if !desired_width.is_finite() || !desired_height.is_finite() {
        return Err(ResolutionError::UnrepresentableDesiredResolution);
    }

    let maximum_dimension = f64::from(MAX_BACKING_DIMENSION);
    let scale = (maximum_dimension / desired_width.max(desired_height)).min(1.0);
    let width = (desired_width * scale)
        .round()
        .clamp(1.0, maximum_dimension) as u32;
    let height = (desired_height * scale)
        .round()
        .clamp(1.0, maximum_dimension) as u32;

    Ok(BackingResolution { width, height })
}

fn scene_time_from_timestamp(
    started_at_milliseconds: &mut Option<f64>,
    timestamp_milliseconds: f64,
) -> Result<SceneTime, apollo18_renderer::InvalidSceneTime> {
    let started_at_milliseconds = *started_at_milliseconds.get_or_insert(timestamp_milliseconds);
    SceneTime::from_elapsed_millis(started_at_milliseconds, timestamp_milliseconds)
}

#[cfg(test)]
mod tests {
    use super::{
        BackingResolution, ResolutionError, scene_time_from_timestamp, select_backing_resolution,
    };

    /// The first render-ready callback establishes scene time zero.
    #[test]
    fn first_timestamp_guarantees_zero_scene_time() {
        let mut started_at_milliseconds = None;

        let scene_time = scene_time_from_timestamp(&mut started_at_milliseconds, 42_000.0)
            .expect("timestamp should be valid");

        assert_eq!(
            scene_time,
            apollo18_renderer::SceneTime::from_seconds(0.0).expect("scene time should be valid")
        );
    }

    /// Later scene time comes from elapsed monotonic time rather than frame count.
    #[test]
    fn timestamps_preserve_elapsed_time_across_stalls() {
        let mut started_at_milliseconds = None;
        let _ = scene_time_from_timestamp(&mut started_at_milliseconds, 1_000.0)
            .expect("timestamp should be valid");

        let scene_time = scene_time_from_timestamp(&mut started_at_milliseconds, 121_000.0)
            .expect("timestamp should be valid");

        assert_eq!(
            scene_time,
            apollo18_renderer::SceneTime::from_seconds(120.0).expect("scene time should be valid")
        );
    }

    /// An uncapped display request is rounded to the nearest backing pixels.
    #[test]
    fn selects_uncapped_display_derived_resolution() {
        let css_width = 320.25;
        let css_height = 200.2;
        let device_pixel_ratio = 2.0;

        let resolution =
            select_backing_resolution(css_width, css_height, device_pixel_ratio).unwrap();

        assert_eq!(
            resolution,
            BackingResolution {
                width: 641,
                height: 400
            }
        );
    }

    /// A request beyond the source-matched bound is scaled uniformly to the cap.
    #[test]
    fn caps_resolution_while_preserving_aspect_ratio() {
        let css_width = 800.0;
        let css_height = 400.0;
        let device_pixel_ratio = 2.0;

        let resolution =
            select_backing_resolution(css_width, css_height, device_pixel_ratio).unwrap();

        assert_eq!(
            resolution,
            BackingResolution {
                width: 1024,
                height: 512
            }
        );
    }

    /// Tiny positive display requests remain renderable after integer rounding.
    #[test]
    fn clamps_positive_measurements_to_one_backing_pixel() {
        let css_width = 0.1;
        let css_height = 0.2;
        let device_pixel_ratio = 1.0;

        let resolution =
            select_backing_resolution(css_width, css_height, device_pixel_ratio).unwrap();

        assert_eq!(
            resolution,
            BackingResolution {
                width: 1,
                height: 1
            }
        );
    }

    /// Invalid browser measurements are reported instead of becoming allocations.
    #[test]
    fn rejects_invalid_measurements() {
        let negative_width = select_backing_resolution(-1.0, 200.0, 2.0);
        let non_finite_height = select_backing_resolution(200.0, f64::NAN, 2.0);
        let zero_device_pixel_ratio = select_backing_resolution(200.0, 200.0, 0.0);
        let non_finite_device_pixel_ratio = select_backing_resolution(200.0, 200.0, f64::INFINITY);
        let overflowing_request = select_backing_resolution(f64::MAX, f64::MAX, 2.0);

        assert_eq!(negative_width, Err(ResolutionError::InvalidCssWidth(-1.0)));
        assert!(matches!(
            non_finite_height,
            Err(ResolutionError::InvalidCssHeight(height)) if height.is_nan()
        ));
        assert_eq!(
            zero_device_pixel_ratio,
            Err(ResolutionError::InvalidDevicePixelRatio(0.0))
        );
        assert_eq!(
            non_finite_device_pixel_ratio,
            Err(ResolutionError::InvalidDevicePixelRatio(f64::INFINITY))
        );
        assert_eq!(
            overflowing_request,
            Err(ResolutionError::UnrepresentableDesiredResolution)
        );
    }
}
