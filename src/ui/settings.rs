use egui::Context;
use super::app_state::APP_STATE;
use egui_theme_switch::global_theme_switch;
use wasm_bindgen::JsValue;
use web_sys::{window, Storage};
use crate::api_request::token::SDK_STATUS; // Add this import

pub fn show_settings_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if (!state.settings_window_open) {
        return;
    }

    static mut SETTINGS_INITIALIZED: bool = false;
    static mut PLAYER_NAME: String = String::new();
    static mut ORIGINAL_NAME: String = String::new();

    unsafe {
        if (!SETTINGS_INITIALIZED) {
            PLAYER_NAME = state.player_name.clone();
            ORIGINAL_NAME = state.player_name.clone();
            SETTINGS_INITIALIZED = true;
        }
    }

    let mut settings_open = state.settings_window_open;
    let mut reset_triggered = false;  // Move this flag outside the closure

    let show_response = egui::Window::new("Settings")
        .open(&mut settings_open)
        .current_pos([
            state.settings_window_pos.0, 
            state.settings_window_pos.1
        ])
        .default_width(280.0)
        .movable(!state.settings_window_locked)
        .show(ctx, |ui| {
            ui.heading("Appearance");
            ui.horizontal(|ui| {
                ui.label("Theme:");
                global_theme_switch(ui);
            });
            
            ui.add_space(16.0);
            ui.heading("Position Lock");
            if ui.button(if state.settings_window_locked { 
                egui::RichText::new("🔒").size(24.0) 
            } else { 
                egui::RichText::new("🔓").size(24.0)
            }).clicked() {
                state.settings_window_locked = !state.settings_window_locked;
            }


            ui.add_space(16.0);
            ui.heading("Window Management");
            if ui.button("Close All Windows").clicked() {
                state.tracks_window_open = false;
                state.player_window_open = false;
                state.settings_window_open = false;
            }

            // Add Reset Window Positions button
            if ui.button("Reset Window Positions").clicked() {
                state.reset_areas();
                reset_triggered = true;  // Set the flag when reset is triggered
                ctx.request_repaint();
            }
            
            ui.add_space(16.0);
            ui.heading("Tracks Loading");
            ui.horizontal(|ui| {
                ui.label("Tracks per load:")
                    .on_hover_text("Number of tracks to load at a time");
                let mut tracks_per_load = state.tracks_per_load;
                ui.add(egui::Slider::new(&mut tracks_per_load, 10..=1000).step_by(10.0)
                    .custom_formatter(|n, _| {
                        if n >= 1000.0 { "Unlimited".to_string() }
                        else { format!("{}", n as i32) }
                    }))
                    .on_hover_text("Choose how many tracks to load at once. Values above 50 will make multiple requests to load tracks faster. 'Unlimited' will load all tracks.");
                if tracks_per_load != state.tracks_per_load {
                    state.tracks_per_load = tracks_per_load;
                }
            });

            ui.add_space(16.0);
            ui.heading("Web Player Settings");
            ui.horizontal(|ui| {
                unsafe {
                    ui.label("Player Name:")
                    .on_hover_text("Rename the Spotify Player device. This is visible across all Spotify Connect devices.");
                    let name_response = ui.text_edit_singleline(&mut PLAYER_NAME);
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
            ui.label("SDK Status")
            .on_hover_text("The current status of the Spotify Web Playback SDK.");
            if let Some(status) = &*SDK_STATUS.lock().unwrap() {
                ui.label(status); // Display SDK status
            }
            ui.add_space(8.0);

            // Add section showing open window positions
            ui.heading("Open Window Positions");
            ui.label(format!("Settings Window: {:?}", state.settings_window_pos));
            if state.tracks_window_open {
                ui.label(format!("Tracks Window: {:?}", state.liked_songs_window_pos));
            }
            if state.player_window_open {
                ui.label(format!("Player Window: {:?}", state.music_player_window_pos));
            }
        });

    // Update the window's position after response
    if let Some(resp) = show_response {
        let rect = resp.response.rect;
        // Only update position if we're not actively resetting
        if !reset_triggered {
            state.settings_window_pos = (rect.min.x, rect.min.y);
        }
    }

    state.settings_window_open = settings_open;

    if (!settings_open) {
        unsafe {
            SETTINGS_INITIALIZED = false;
        }
    }
}
