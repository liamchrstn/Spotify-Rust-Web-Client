mod app_state;
mod spotify_apis;
mod token;
mod ui;
mod utils;
mod models;

use eframe::wasm_bindgen::{self, prelude::*};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use wasm_bindgen_futures::spawn_local;
use eframe::WebRunner;
use console_error_panic_hook;
use std::panic;

// External function
#[wasm_bindgen]
extern "C" {
    fn loginWithSpotify();
}

// Entry point for the wasm application
#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let web_options = eframe::WebOptions::default();
    WebRunner::new()
        .start(
            "canvas",
            web_options,
            Box::new(|_cc| Box::new(ui::SpotifyApp::default())),
        )
        .await
}