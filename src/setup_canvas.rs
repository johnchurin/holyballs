use wasm_bindgen::prelude::*;
use web_sys::*;
#[wasm_bindgen]
extern "C" {
}
// test case
#[wasm_bindgen]
pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
#[wasm_bindgen]
pub fn start_canvas() -> Result<(), JsValue> {
    let window: Window = web_sys::window().unwrap();
    let document: Document = window.document().unwrap();

    // Option A: Get an existing canvas from HTML (e.g., <canvas id="custom-canvas"></canvas>)
    // let canvas = document.get_element_by_id("custom-canvas").unwrap();
    // let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

    // Option B: Create a brand new canvas element programmatically
    let canvas = document.create_element("canvas")?.dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(800);
    canvas.set_height(600);
    document.body().unwrap().append_child(&canvas)?;
    // Get the 2D rendering context
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // Draw a custom rectangle
    context.set_fill_style(&JsValue::from_str("#FF0000"));
    context.fill_rect(10.0, 10.0, 100.0, 100.0);

    Ok(())
}
