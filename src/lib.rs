mod app_state;
mod spotify_apis;
mod token;
mod ui;
mod utils;
mod models;

use eframe::wasm_bindgen::{self, prelude::*};
use web_sys::HtmlCanvasElement;
use eframe::WebRunner;
use console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
extern "C" {
    fn loginWithSpotify();
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let canvas_element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let web_options = eframe::WebOptions::default();
    WebRunner::new()
        .start(
            canvas_element,
            web_options,
            Box::new(|_cc| Ok(Box::new(ui::SpotifyApp::default()))),
        )
        .await
}