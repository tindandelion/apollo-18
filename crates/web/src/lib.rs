use apollo18_renderer::{
    CANONICAL_ANIMATION_EPOCH, LunarColorMap, LunarElevationMap, LunarEphemeris,
    LunarPhaseAnimation, SceneTime, image::decode_float_tiff, image::decode_jpeg,
    render_lunar_globe,
};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

const MAX_BACKING_DIMENSION: u32 = 1152;
const CANVAS_ID: &str = "apollo18-canvas";
const LUNAR_COLOR_MAP_JPEG: &[u8] = include_bytes!("../../../assets/nasa/lroc_color_2k.jpg");
const LUNAR_ELEVATION_MAP_TIFF: &[u8] = include_bytes!("../../../assets/nasa/ldem_4.tif");
const LUNAR_EPHEMERIS_JSON: &[u8] = include_bytes!("../../../assets/nasa/mooninfo_2026.json");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
    let canvas = document
        .get_element_by_id(CANVAS_ID)
        .ok_or_else(|| JsValue::from_str("apollo18 canvas is missing"))?
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas 2D context is unavailable"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    start_animation(window, canvas, context)
}

fn start_animation(
    window: Window,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    let color_map = LunarColorMap::new(
        decode_jpeg(LUNAR_COLOR_MAP_JPEG).map_err(|error| JsValue::from_str(&error.to_string()))?,
    );
    let elevation_map = LunarElevationMap::new(
        decode_float_tiff(LUNAR_ELEVATION_MAP_TIFF)
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
    );
    let ephemeris = LunarEphemeris::from_nasa_json(LUNAR_EPHEMERIS_JSON)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let lunar_phase_animation = LunarPhaseAnimation::new(ephemeris, CANONICAL_ANIMATION_EPOCH)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let animation = Rc::new(RefCell::new(CanvasAnimation::new(
        canvas,
        context,
        color_map,
        elevation_map,
        lunar_phase_animation,
    )));
    let callback_slot = Rc::new(RefCell::new(None));
    let callback_slot_for_frame = Rc::clone(&callback_slot);
    let window_for_frame = window.clone();

    let callback = Closure::<dyn FnMut(f64)>::new(move |timestamp_milliseconds| {
        if let Err(error) = animation.borrow_mut().render(
            timestamp_milliseconds,
            window_for_frame.device_pixel_ratio(),
        ) {
            wasm_bindgen::throw_val(error);
        }

        let callback_slot = callback_slot_for_frame.borrow();
        let callback = callback_slot
            .as_ref()
            .expect("animation callback should remain installed");
        if let Err(error) = request_animation_frame(&window_for_frame, callback) {
            wasm_bindgen::throw_val(error);
        }
    });
    request_animation_frame(&window, &callback)?;
    callback_slot.replace(Some(callback));

    Ok(())
}

fn request_animation_frame(
    window: &Window,
    callback: &Closure<dyn FnMut(f64)>,
) -> Result<(), JsValue> {
    window
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map(|_| ())
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
) -> Result<Option<BackingResolution>, ResolutionError> {
    if !css_width.is_finite() || css_width < 0.0 {
        return Err(ResolutionError::InvalidCssWidth(css_width));
    }
    if !css_height.is_finite() || css_height < 0.0 {
        return Err(ResolutionError::InvalidCssHeight(css_height));
    }
    if !device_pixel_ratio.is_finite() || device_pixel_ratio <= 0.0 {
        return Err(ResolutionError::InvalidDevicePixelRatio(device_pixel_ratio));
    }
    if css_width == 0.0 || css_height == 0.0 {
        return Ok(None);
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

    Ok(Some(BackingResolution { width, height }))
}

struct CanvasAnimation {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    color_map: LunarColorMap,
    elevation_map: LunarElevationMap,
    lunar_phase_animation: LunarPhaseAnimation,
    started_at_milliseconds: Option<f64>,
}

impl CanvasAnimation {
    fn new(
        canvas: HtmlCanvasElement,
        context: CanvasRenderingContext2d,
        color_map: LunarColorMap,
        elevation_map: LunarElevationMap,
        lunar_phase_animation: LunarPhaseAnimation,
    ) -> Self {
        Self {
            canvas,
            context,
            color_map,
            elevation_map,
            lunar_phase_animation,
            started_at_milliseconds: None,
        }
    }

    fn render(
        &mut self,
        timestamp_milliseconds: f64,
        device_pixel_ratio: f64,
    ) -> Result<(), JsValue> {
        let started_at_milliseconds = *self
            .started_at_milliseconds
            .get_or_insert(timestamp_milliseconds);
        let bounds = self.canvas.get_bounding_client_rect();
        let Some(resolution) =
            select_backing_resolution(bounds.width(), bounds.height(), device_pixel_ratio)
                .map_err(|error| JsValue::from_str(&error.to_string()))?
        else {
            return Ok(());
        };

        if self.canvas.width() != resolution.width || self.canvas.height() != resolution.height {
            self.canvas.set_width(resolution.width);
            self.canvas.set_height(resolution.height);
        }

        let scene_time =
            SceneTime::from_elapsed_millis(started_at_milliseconds, timestamp_milliseconds)
                .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let appearance = self.lunar_phase_animation.lunar_appearance(scene_time);
        let frame = render_lunar_globe(
            resolution.width,
            resolution.height,
            appearance,
            &self.color_map,
            &self.elevation_map,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(frame.pixels()),
            frame.width(),
            frame.height(),
        )?;
        self.context.put_image_data(&image, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{BackingResolution, ResolutionError, select_backing_resolution};

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
            Some(BackingResolution {
                width: 641,
                height: 400
            })
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
            Some(BackingResolution {
                width: 1152,
                height: 576
            })
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
            Some(BackingResolution {
                width: 1,
                height: 1
            })
        );
    }

    /// A zero-sized CSS canvas skips rendering until layout gives it an area.
    #[test]
    fn skips_zero_sized_canvas() {
        let css_width = 0.0;
        let css_height = 200.0;
        let device_pixel_ratio = 2.0;

        let resolution =
            select_backing_resolution(css_width, css_height, device_pixel_ratio).unwrap();

        assert_eq!(resolution, None);
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
