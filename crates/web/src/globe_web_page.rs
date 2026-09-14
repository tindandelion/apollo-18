use crate::error::Error;
use apollo18_renderer::Framebuffer;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, ImageData, Window};

const CANVAS_ID: &str = "apollo18-canvas";
const GLOBE_CONTAINER_ID: &str = "apollo18-globe-container";
const LOADING_STATUS_ID: &str = "apollo18-render-loading";
const RENDER_ERROR_ID: &str = "apollo18-render-error";

pub(crate) struct GlobeWebPage {
    window: Window,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    globe_container: Element,
    loading_status: Element,
    render_error: Element,
    initial_loading_completed: Cell<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FrameRequest {
    timestamp_milliseconds: f64,
    canvas_css_width: f64,
    canvas_css_height: f64,
    device_pixel_ratio: f64,
}

impl FrameRequest {
    pub(crate) fn timestamp_milliseconds(self) -> f64 {
        self.timestamp_milliseconds
    }

    pub(crate) fn canvas_css_width(self) -> f64 {
        self.canvas_css_width
    }

    pub(crate) fn canvas_css_height(self) -> f64 {
        self.canvas_css_height
    }

    pub(crate) fn device_pixel_ratio(self) -> f64 {
        self.device_pixel_ratio
    }
}

impl GlobeWebPage {
    pub(crate) fn discover() -> Result<Self, Error> {
        let window = web_sys::window().ok_or_else(|| Error::new("window is unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| Error::new("document is unavailable"))?;
        let canvas = document
            .get_element_by_id(CANVAS_ID)
            .ok_or_else(|| Error::new("apollo18 canvas is missing"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| Error::new("apollo18 canvas is not an HTML canvas element"))?;
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| Error::new("Canvas 2D context is unavailable"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| Error::new("canvas context is not a Canvas 2D context"))?;
        let globe_container = document
            .get_element_by_id(GLOBE_CONTAINER_ID)
            .ok_or_else(|| Error::new("apollo18 globe container is missing"))?;
        let loading_status = document
            .get_element_by_id(LOADING_STATUS_ID)
            .ok_or_else(|| Error::new("apollo18 loading status is missing"))?;
        let render_error = document
            .get_element_by_id(RENDER_ERROR_ID)
            .ok_or_else(|| Error::new("apollo18 render error message is missing"))?;

        Ok(Self {
            window,
            canvas,
            context,
            globe_container,
            loading_status,
            render_error,
            initial_loading_completed: Cell::new(false),
        })
    }

    pub(crate) fn start_presenting<F>(self: &Rc<Self>, mut render: F) -> Result<(), Error>
    where
        F: FnMut(FrameRequest) -> Result<Framebuffer, Error> + 'static,
    {
        let callback_slot = Rc::new(RefCell::new(None));
        let callback_slot_for_frame = Rc::clone(&callback_slot);
        let page = Rc::clone(self);

        let callback = Closure::<dyn FnMut(f64)>::new(move |timestamp_milliseconds| {
            if let Err(error) = page.present_animation_frame(timestamp_milliseconds, &mut render) {
                wasm_bindgen::throw_val(error.into());
            }

            let callback_slot = callback_slot_for_frame.borrow();
            let callback = callback_slot
                .as_ref()
                .expect("animation callback should remain installed");
            if let Err(error) = page.request_animation_frame(callback) {
                wasm_bindgen::throw_val(error.into());
            }
        });
        self.request_animation_frame(&callback)?;
        callback_slot.replace(Some(callback));

        Ok(())
    }

    pub(crate) fn show_initialization_failure(&self, error: &Error) -> Result<(), Error> {
        web_sys::console::error_1(&JsValue::from_str(&format!(
            "Apollo 18 could not initialize lunar rendering: {error}"
        )));
        self.dismiss_loading_status()?;
        self.canvas.set_attribute("hidden", "")?;
        self.render_error.remove_attribute("hidden")?;
        Ok(())
    }

    fn present_animation_frame<F>(
        &self,
        timestamp_milliseconds: f64,
        render: &mut F,
    ) -> Result<(), Error>
    where
        F: FnMut(FrameRequest) -> Result<Framebuffer, Error>,
    {
        let bounds = self.canvas.get_bounding_client_rect();
        if bounds.width() == 0.0 || bounds.height() == 0.0 {
            return Ok(());
        }

        let frame = render(FrameRequest {
            timestamp_milliseconds,
            canvas_css_width: bounds.width(),
            canvas_css_height: bounds.height(),
            device_pixel_ratio: self.window.device_pixel_ratio(),
        })?;

        if self.canvas.width() != frame.width() || self.canvas.height() != frame.height() {
            self.canvas.set_width(frame.width());
            self.canvas.set_height(frame.height());
        }

        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(frame.pixels()),
            frame.width(),
            frame.height(),
        )?;
        self.context.put_image_data(&image, 0.0, 0.0)?;
        self.dismiss_loading_after_first_presentation()
    }

    fn dismiss_loading_after_first_presentation(&self) -> Result<(), Error> {
        if self.initial_loading_completed.get() {
            return Ok(());
        }

        self.dismiss_loading_status()?;
        self.initial_loading_completed.set(true);
        Ok(())
    }

    fn dismiss_loading_status(&self) -> Result<(), Error> {
        self.globe_container.remove_attribute("aria-busy")?;
        self.loading_status.set_attribute("hidden", "")?;
        Ok(())
    }

    fn request_animation_frame(&self, callback: &Closure<dyn FnMut(f64)>) -> Result<(), Error> {
        self.window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .map(|_| ())
            .map_err(Error::from)
    }
}
