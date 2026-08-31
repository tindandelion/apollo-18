use apollo18_renderer::render_triangles;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

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

    let frame = render_triangles(CANONICAL_WIDTH, CANONICAL_HEIGHT)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(frame.pixels()),
        frame.width(),
        frame.height(),
    )?;
    context.put_image_data(&image, 0.0, 0.0)?;

    Ok(())
}
