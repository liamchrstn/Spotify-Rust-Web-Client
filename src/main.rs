#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let web_options = eframe::WebOptions::default();
    WebRunner::new()
        .start(
            "canvas",
            web_options,
            Box::new(|_cc| Box::new(SpotifyApp::default())),
        )
        .await
}