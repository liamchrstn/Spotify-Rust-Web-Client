mod api_request;
mod ui;
mod utils;
mod storage;
mod mediaplayer;

#[wasm_bindgen]
extern "C" {
    pub fn loginWithSpotify();
}

use eframe::wasm_bindgen::{self, prelude::*};
use web_sys::HtmlCanvasElement;
use eframe::WebRunner;
use console_error_panic_hook;
use std::panic;


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
            Box::new(|cc| {
                // Configure fonts with Unicode support
                let mut fonts = egui::FontDefinitions::default();
                
                // Add support for Asian characters
                fonts.font_data.insert(
                    "noto_sans".to_owned(),
                    egui::FontData::from_static(include_bytes!("../assets/NotoSansCJKjp-Regular.otf")).into()
                );
                
                // Add Noto Sans as primary font for all text styles
                fonts.families.get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "noto_sans".to_owned());
                
                fonts.families.get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "noto_sans".to_owned());

                egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
                cc.egui_ctx.set_fonts(fonts);
                
                // Install image loaders
                egui_extras::install_image_loaders(&cc.egui_ctx);
                
                Ok(Box::new(ui::SpotifyApp::default()))
            }),
        )
        .await
}
