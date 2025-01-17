#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let options = eframe::WebOptions::default();
    
    eframe::start_web(
        "the_canvas_id",
        options,
        Box::new(|_cc| Box::new(SpotifyApp::new())) // Use new() instead of default()
    )
    .map_err(|err| err.to_string().into())
}