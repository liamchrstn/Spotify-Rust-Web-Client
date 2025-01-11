use eframe::egui;
use crate::mediaplayer::scrubber::ScrubBar;
use crate::mediaplayer::scrubber::TimeManager;
use crate::ui::app_state::APP_STATE;
use crate::api_request::imagerender::get_or_load_image;
use crate::api_request::Track_Status::{get_current_playback, skip_to_next, skip_to_previous, toggle_shuffle, get_devices, transfer_playback};
use crate::api_request::token::get_token;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys;
use web_sys::console;
use egui_extras::{StripBuilder, Size};

#[wasm_bindgen]
pub fn is_player_window_open() -> bool {
    let state = APP_STATE.lock().unwrap();
    state.player_window_open
}

pub fn show_mediaplayer_window(ctx: &egui::Context) {
    let mut state = APP_STATE.lock().unwrap();
    let window_open = state.player_window_open;

    // Only proceed if the window is open
    if (!window_open) {
        return;
    }

    // Get duration from JavaScript
    let duration = js_sys::eval("window.totalDuration || 100000.0")
        .unwrap_or(100000.0.into())
        .as_f64()
        .unwrap_or(100000.0);

    let mut time_manager = TimeManager::new(duration, 1.0);

    // Update current time from window state (updated by JavaScript)
    if let Ok(current_time) = js_sys::eval("window.currentPlaybackTime || 0.0") {
        if let Some(time) = current_time.as_f64() {
            time_manager.current_time = time;
        }
    }

    // Update playing state from window state (set by get_current_playback)
    if let Ok(is_playing) = js_sys::eval("window.isPlaying || false") {
        time_manager.playing = is_playing.as_bool().unwrap_or(false);
    }

    let mut window_open = state.player_window_open;
    let music_player_pos = state.music_player_window_pos;

    drop(state); // Release the lock

    // Media player window
    let window_response = egui::Window::new("Music Player")
        .resizable(true)
        .default_size([300.0, 400.0])  // Reduced from 500.0
        .min_size([250.0, 350.0])      // Reduced from 400.0
        .open(&mut window_open)
        .current_pos([
            music_player_pos.0, 
            music_player_pos.1
        ])
        .collapsible(true)
        .show(ctx, |ui| {
            // Calculate the album art size once, before the StripBuilder
            let square_size = ui.available_width().min(200.0);

            StripBuilder::new(ui)
                .size(Size::relative(0.5)) //Album art
                .size(Size::exact(30.0))  //Scrubber
                .size(Size::exact(50.0))  //Controls
                .size(Size::exact(60.0))  //Track info
                .vertical(|mut strip| {
                    // Album art section
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            let art_size = egui::vec2(square_size, square_size);
                            let rect = egui::Rect::from_center_size(
                                ui.available_rect_before_wrap().center(),
                                art_size
                            );

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

                            if let Some(url) = album_art_url {
                                if let Some(image) = get_or_load_image(ctx, &url) {
                                    ui.put(rect, image.fit_to_exact_size(art_size));
                                }
                            } else {
                                ui.painter().rect_filled(rect, 10.0, egui::Color32::DARK_GRAY);
                            }
                        });
                    });

                    // Scrubber section
                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            let mut scrub_bar = ScrubBar::new(time_manager.end_time);
                            scrub_bar.add(
                                ui, 
                                &mut time_manager.current_time, 
                                egui::vec2(square_size, 20.0)
                            );
                        });
                    });



                    // Controls section
                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                ui.add_space((ui.available_width() - 200.0) / 2.0); // Adjusted spacing

                                // Shuffle button
                                let shuffle_state = if let Ok(state) = js_sys::eval("window.shuffleState") {
                                    state.as_bool().unwrap_or(false)
                                } else {
                                    false
                                };
                                
                                if ui.add_sized(
                                    [40.0, 40.0],
                                    egui::Button::new("🔀")
                                        .frame(false)
                                        .fill(if shuffle_state {
                                            ui.style().visuals.widgets.active.bg_fill
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        })
                                ).on_hover_text(if shuffle_state { "Shuffle On" } else { "Shuffle Off" })
                                .clicked() {
                                    if let Some(token) = get_token() {
                                        spawn_local(async move {
                                            toggle_shuffle(token).await;
                                        });
                                    }
                                }

                                // Previous track button
                                if ui.add_sized(
                                    [40.0, 40.0],
                                    egui::Button::new("⏮").frame(false)
                                ).on_hover_text("Previous track")
                                .clicked() {
                                    if let Some(token) = get_token() {
                                        spawn_local(async move {
                                            skip_to_previous(token).await;
                                        });
                                    }
                                }

                                // Play/Pause button
                                let is_playing = if let Ok(is_playing) = js_sys::eval("window.isPlaying") {
                                    is_playing.as_bool().unwrap_or(false)
                                } else {
                                    false
                                };

                                let is_ready = if let Ok(is_ready) = js_sys::eval("window.isReady") {
                                    is_ready
                                } else {
                                    JsValue::from(false)
                                };

                                let button = ui.add_sized(
                                    [40.0, 40.0],
                                    egui::Button::new(
                                        if is_playing {
                                            egui::RichText::new("⏸")
                                        } else {
                                            egui::RichText::new("▶")
                                        }
                                    )
                                )
                                .on_hover_text(if is_ready.as_bool() != Some(true) {
                                    "Player not ready"
                                } else if is_playing {
                                    "Pause"
                                } else {
                                    "Play"
                                });

                                if button.clicked() {
                                    console::log_1(&"Play button clicked in Rust UI".into());
                                    let _ = js_sys::eval("console.log('Calling playPause'); window.playPause && window.playPause()");
                                }

                                // Next track button
                                if ui.add_sized(
                                    [40.0, 40.0],
                                    egui::Button::new("⏭").frame(false)
                                ).on_hover_text("Next track")
                                .clicked() {
                                    if let Some(token) = get_token() {
                                        spawn_local(async move {
                                            skip_to_next(token).await;
                                        });
                                    }
                                }

                                // Replace device button & popup with a context menu:
                                let menu_response = ui.menu_button("💻", |ui| {
                                    ui.set_min_width(150.0);

                                    // Only fetch devices when menu is first opened
                                    if let Ok(first_open) = js_sys::eval("
                                        if (!window.deviceMenuFirstOpen) {
                                            window.deviceMenuFirstOpen = true;
                                            true
                                        } else {
                                            false
                                        }
                                    ") {
                                        if first_open.as_bool().unwrap_or(false) {
                                            spawn_local(async {
                                                get_devices().await;
                                            });
                                        }
                                    }

                                    // Use cached devices from previous fetch
                                    let devices = js_sys::eval("window.availableDevices || []").unwrap();
                                    if let Some(devices_array) = devices.dyn_ref::<js_sys::Array>() {
                                        for i in 0..devices_array.length() {
                                            if let Ok(device) = js_sys::Reflect::get(&devices_array.get(i), &"name".into()) {
                                                if let Some(name) = device.as_string() {
                                                    if ui.button(&name).clicked() {
                                                        if let Ok(id) = js_sys::Reflect::get(&devices_array.get(i), &"id".into()) {
                                                            let device_id = id.as_string().unwrap_or_default();
                                                            spawn_local(async move {
                                                                transfer_playback(device_id).await;
                                                            });
                                                        }
                                                        ui.close_menu();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });

                                // Reset first_open state when menu closes
                                if (!ui.ctx().is_pointer_over_area()) {
                                    let _ = js_sys::eval("window.deviceMenuFirstOpen = false");
                                }

                                ui.add_space((ui.available_width() - 200.0) / 2.0); // Adjusted spacing
                            });
                        });
                    });

                    // Track info section
                    strip.strip(|builder| {
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

                        builder
                            .size(Size::exact(30.0))  // Title
                            .size(Size::exact(20.0))  // Artist
                            .vertical(|mut strip| {
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        if let Some((title, _)) = &track_info {
                                            ui.label(egui::RichText::new(title).heading());
                                        } else {
                                            ui.label(egui::RichText::new("No track playing").heading());
                                        }
                                    });
                                });
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        if let Some((_, artist)) = &track_info {
                                            ui.label(egui::RichText::new(artist).small());
                                        } else {
                                            ui.label(egui::RichText::new("Select a track to play").small());
                                        }
                                    });
                                });
                            });
                    });
                });
        });
    
    // Re-lock to update window state if it changed
    let mut state = APP_STATE.lock().unwrap();
    state.player_window_open = window_open;

    if let Some(resp) = window_response {
        let rect = resp.response.rect;
        // Always update position since this window isn't being reset
        state.music_player_window_pos = (rect.min.x, rect.min.y);
    }

    // Update current time more frequently if playing
    if (time_manager.playing) {
        if let Ok(current_time) = js_sys::eval("window.currentPlaybackTime || 0.0") {
            if let Some(time) = current_time.as_f64() {
                time_manager.current_time = time;
            } else {
                // If no time available, increment locally for smoother UI
                time_manager.current_time += 0.2; // Assume 200ms update rate
            }
        }
        // Request more frequent repaints
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
    
    //continuous repaint while playing
    if (time_manager.playing) {
        ctx.request_repaint();
    }
}
