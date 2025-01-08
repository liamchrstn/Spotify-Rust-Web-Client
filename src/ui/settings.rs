use egui::Context;
use super::app_state::APP_STATE;
use egui_theme_switch::global_theme_switch;
use wasm_bindgen::JsValue;
use web_sys::{window, Storage};

pub fn show_settings_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.settings_window_open {
        return;
    }

    static mut SETTINGS_INITIALIZED: bool = false;
    static mut PLAYER_NAME: String = String::new();
    static mut ORIGINAL_NAME: String = String::new();

    unsafe {
        if !SETTINGS_INITIALIZED {
            PLAYER_NAME = state.player_name.clone();
            ORIGINAL_NAME = state.player_name.clone();
            SETTINGS_INITIALIZED = true;
        }
    }

    let mut settings_open = state.settings_window_open;

    egui::Window::new("Settings")
        .open(&mut settings_open)
        .show(ctx, |ui| {
            ui.heading("Appearance");
            ui.horizontal(|ui| {
                ui.label("Theme:");
                global_theme_switch(ui);
            });
            
            ui.add_space(16.0);
            ui.heading("Player Settings");
            ui.horizontal(|ui| {
                unsafe {
                    ui.label("Player Name:");
                    let name_response = ui.text_edit_singleline(&mut PLAYER_NAME)
                        .on_hover_text("Rename the Spotify Player device. This is visible across all Spotify Connect devices.");
                    let name_changed = PLAYER_NAME != ORIGINAL_NAME;
                    
                    let apply_button = ui.add_enabled(
                        name_changed,
                        egui::Button::new("Apply")
                    );

                    if apply_button.clicked() {
                        if let Some(window) = window() {
                            if let Ok(local_storage) = window.local_storage() {
                                if let Some(storage) = local_storage {
                                    let _ = storage.set_item("player_name", &PLAYER_NAME);
                                    // Call JavaScript to reinitialize the player
                                    let _ = js_sys::eval("window.reinitializePlayer && window.reinitializePlayer()");
                                    ORIGINAL_NAME = PLAYER_NAME.clone();
                                    state.player_name = PLAYER_NAME.clone();
                                }
                            }
                        }
                    }
                }
            });
            ui.add_space(8.0);
        });

    state.settings_window_open = settings_open;

    if !settings_open {
        unsafe {
            SETTINGS_INITIALIZED = false;
        }
    }
}
