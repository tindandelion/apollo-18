use apollo18_renderer::{SceneTime, render_cube};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

const CANONICAL_WIDTH: u32 = 800;
const CANONICAL_HEIGHT: u32 = 800;
const CANVAS_ID: &str = "apollo18-canvas";

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

    canvas.set_width(CANONICAL_WIDTH);
    canvas.set_height(CANONICAL_HEIGHT);

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas 2D context is unavailable"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    start_animation(window, context)
}

fn start_animation(window: Window, context: CanvasRenderingContext2d) -> Result<(), JsValue> {
    let animation = Rc::new(RefCell::new(CubeAnimation::new(context)));
    let callback_slot = Rc::new(RefCell::new(None));
    let callback_slot_for_frame = Rc::clone(&callback_slot);
    let window_for_frame = window.clone();

    let callback = Closure::<dyn FnMut(f64)>::new(move |timestamp_milliseconds| {
        if let Err(error) = animation.borrow_mut().render(timestamp_milliseconds) {
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

struct CubeAnimation {
    context: CanvasRenderingContext2d,
    started_at_milliseconds: Option<f64>,
}

impl CubeAnimation {
    fn new(context: CanvasRenderingContext2d) -> Self {
        Self {
            context,
            started_at_milliseconds: None,
        }
    }

    fn render(&mut self, timestamp_milliseconds: f64) -> Result<(), JsValue> {
        let started_at_milliseconds = *self
            .started_at_milliseconds
            .get_or_insert(timestamp_milliseconds);
        let elapsed_seconds = (timestamp_milliseconds - started_at_milliseconds) / 1000.0;
        let scene_time = SceneTime::from_seconds(elapsed_seconds)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let frame = render_cube(CANONICAL_WIDTH, CANONICAL_HEIGHT, scene_time)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(frame.pixels()),
            frame.width(),
            frame.height(),
        )?;
        self.context.put_image_data(&image, 0.0, 0.0)
    }
}
