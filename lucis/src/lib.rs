#[macro_use]
extern crate cfg_if;
extern crate web_sys;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

pub mod geometry;
pub mod scene;
mod raytrace;
pub use crate::raytrace::Raytracer;

use nalgebra::{Point3, Transform3, Vector3};

pub type Point = Point3<f32>;
pub type Vector = Vector3<f32>;
pub type Transform = Transform3<f32>;


// Called by our JS entry point to run the example.
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let raytracer: Raytracer = Default::default();
    let mut pixels = raytracer.render(200, 200);

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mutArr: &mut [u8] = pixels.as_mut();
    let clamped: Clamped<&mut [u8]> = Clamped(mutArr);

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(clamped, 200, 200).unwrap();
    context.put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(&image_data, 0.0, 0.0, 0.0, 0.0, 500.0, 500.0);

//    let window = web_sys::window().expect("should have a Window");
//    let document = window.document().expect("should have a Document");
//
//    let p: web_sys::Node = document.create_element("p")?.into();
//    p.set_text_content(Some("Hello! Rust, WebAssembly, and Webpack!"));
//
//    let body = document.body().expect("should have a body");
//    let body: &web_sys::Node = body.as_ref();
//    body.append_child(&p)?;

    Ok(())
}
