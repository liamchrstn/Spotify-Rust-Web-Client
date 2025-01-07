use eframe::egui;
use crate::mediaplayer::scrubber::ScrubBar;
use crate::mediaplayer::scrubber::TimeManager;
use crate::ui::app_state::APP_STATE;
use crate::api_request::imagerender::get_or_load_image; // Add this import
use wasm_bindgen::prelude::*;
use js_sys;
use web_sys::console;

pub fn show_mediaplayer_window(ctx: &egui::Context) {
    let mut time_manager = TimeManager::new(100_000.0, 1.0);
    let mut state = APP_STATE.lock().unwrap();

    // Media player window
    egui::Window::new("Music Player")
        .resizable(true)
        .open(&mut state.player_window_open)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let padding = 20.0;
                let square_size = egui::vec2(100.0, 100.0);
                let total_size = ui.available_size();
                ui.set_min_size(total_size);

                // Get album art URL from player state
                let album_art_url = js_sys::eval("window.currentPlayerState")
                    .ok()
                    .and_then(|val| {
                        if val.is_object() {
                            let state = js_sys::Object::from(val);
                            if let Ok(track_window) = js_sys::Reflect::get(&state, &"track_window".into()) {
                                if let Ok(track) = js_sys::Reflect::get(&track_window, &"current_track".into()) {
                                    if let Ok(album) = js_sys::Reflect::get(&track, &"album".into()) {
                                        if let Ok(images) = js_sys::Reflect::get(&album, &"images".into()) {
                                            let images_array = js_sys::Array::from(&images);
                                            if images_array.length() > 0 {
                                                if let Ok(image) = js_sys::Reflect::get(&images_array.get(0), &"url".into()) {
                                                    return image.as_string();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None
                    });

                // Create rect outside of conditional blocks so it's available throughout
                let rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2((ui.available_width() - square_size.x) * 0.5, padding),
                    square_size
                );

                // Display album art or placeholder
                if let Some(url) = album_art_url {
                    if let Some(image) = get_or_load_image(ctx, &url) {
                        ui.put(rect, image.fit_to_exact_size(square_size));
                    }
                } else {
                    // Fallback to placeholder if no album art
                    ui.painter().rect_filled(rect, 10.0, egui::Color32::DARK_GRAY);
                }
                ui.add_space(square_size.y + padding);
                
                time_manager.update();
                
                // Media controls
                let mut scrub_bar = ScrubBar::new(time_manager.end_time);
                let scrubber_height = 30.0;
                scrub_bar.add(ui, &mut time_manager.current_time, egui::vec2(square_size.x, scrubber_height));

                // Ensure the play container is below the scrubber
                ui.add_space(10.0); // Add some space between the scrubber and the play container
                let center_x = rect.center().x;
                let button_size = egui::vec2(40.0, 40.0); // Define button size here
                let spacing = 10.0;
                let button_container_width = button_size.x * 3.0 + spacing * 2.0; // Width of the button container
                ui.allocate_ui_at_rect(
                    egui::Rect::from_center_size(
                        egui::pos2(center_x, rect.max.y + padding + scrubber_height), // Adjusted y position
                        egui::vec2(button_container_width, button_size.y)
                    ),
                    |ui| {
                        // Get current player state from JavaScript
                        let player_state = js_sys::eval("window.player && window.player.getCurrentState()")
                            .ok()
                            .and_then(|val| val.as_bool());
                        
                        let mut is_playing = player_state.unwrap_or(false);
                        
                        let mut button = ui.add(egui::Button::new(if is_playing {
                            egui::RichText::new("⏸").size(button_size.x)
                        } else {
                            egui::RichText::new("▶").size(button_size.x)
                        }));
                        
                        // Disable button if player isn't ready
                        if let Ok(is_ready) = js_sys::eval("window.isReady") {
                            if is_ready.as_bool() != Some(true) {
                                button = button.on_hover_text("Player not ready");
                                button = button.interact(egui::Sense::hover());
                            }
                        }
                        
                        if button.clicked() {
                            console::log_1(&"Play button clicked in Rust UI".into());
                            // Toggle play/pause through JavaScript
                            let result = js_sys::eval("console.log('Calling playPause'); window.playPause && window.playPause()");
                            if let Err(err) = result {
                                console::error_1(&format!("Error calling playPause: {:?}", err).into());
                            }
                            is_playing = !is_playing;
                        }
                    }
                );

                // Get current track info from stored state
                let track_info = js_sys::eval("window.currentPlayerState")
                    .ok()
                    .and_then(|val| {
                        if val.is_object() {
                            let state = js_sys::Object::from(val);
                            if let Ok(track_window) = js_sys::Reflect::get(&state, &"track_window".into()) {
                                if let Ok(track) = js_sys::Reflect::get(&track_window, &"current_track".into()) {
                                    if let (Ok(name_value), Ok(artists_value)) = (
                                        js_sys::Reflect::get(&track, &"name".into()),
                                        js_sys::Reflect::get(&track, &"artists".into())
                                    ) {
                                        let title = name_value.as_string();
                                        let artist = {
                                            let artists_array = js_sys::Array::from(&artists_value);
                                            if artists_array.length() > 0 {
                                                if let Ok(artist_obj) = js_sys::Reflect::get(&artists_array.get(0), &"name".into()) {
                                                    artist_obj.as_string()
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        };
                                        
                                        if let (Some(title), Some(artist)) = (title, artist) {
                                            return Some((title, artist));
                                        }
                                    }
                                }
                            }
                        }
                        None
                    });

                // Display track info or placeholder
                if let Some((title, artist)) = track_info {
                    ui.label(egui::RichText::new(title).heading());
                    ui.label(egui::RichText::new(artist).small());
                } else {
                    ui.label(egui::RichText::new("No track playing").heading());
                    ui.label(egui::RichText::new("Select a track to play").small());
                }
            });
        });
    
    //continuous repaint while playing
    if time_manager.playing {
        ctx.request_repaint();
    }
}
