use super::app_state::APP_STATE;  // Changed from crate::app_state
use crate::api_request::{saved_tracks::fetch_saved_tracks, token::{ACCESS_TOKEN}}; // Removed SDK_STATUS import
use crate::loginWithSpotify;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use super::savedtracks::show_saved_tracks_window;
use wasm_bindgen::JsCast;  // Add JsCast trait for dyn_ref
use crate::api_request::playlists::fetch_playlists;
use crate::ui::playlist_tracks::show_playlist_tracks_windows;

#[derive(Default)]
pub struct SpotifyApp {
    pub show_player: bool, // new field
}

impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut state = APP_STATE.lock().unwrap();
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(name) = &state.username {
                        ui.heading(format!("Welcome, {}", name));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⛭").clicked() {
                                state.settings_window_open = true;
                            }
                            if ui.button("Logout").clicked() {
                                if let Some(window) = window() {
                                    if let Ok(local_storage) = window.local_storage() {
                                        if let Some(storage) = local_storage {
                                            let _ = storage.remove_item("spotify_token");
                                            state.username = None;
                                            state.saved_tracks.clear();
                                            state.show_tracks = false;
                                        }
                                    }
                                }
                            }
                        });
                    }
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(_) = &state.username {                 
                    if ui.button("View Your Liked Songs").clicked() {
                        state.show_tracks = true;
                        state.tracks_window_open = true;
                        let token = ACCESS_TOKEN.lock().unwrap().clone().unwrap();
                        spawn_local(async {
                            fetch_saved_tracks(token).await;
                        });
                    }

                    if ui.button("View Your Playlists").clicked() {
                        let token = crate::api_request::token::ACCESS_TOKEN
                            .lock().unwrap().clone().unwrap_or_default();
                        wasm_bindgen_futures::spawn_local(async move {
                            fetch_playlists(token).await;
                        });
                    }
                    
                    if ui.button("Create Collage").clicked() {
                        // Open collage creation window
                        state.collage_window_open = true;
                    }

                    if ui.button("Show Player").clicked() { // new button
                        self.show_player = true;
                        state.player_window_open = true;
                        
                        // Check for active devices on player show
                        use crate::api_request::track_status::{has_active_devices, get_devices};
                        spawn_local(async {
                            // First check for active devices
                            has_active_devices().await;
                            
                            // Wait a bit for the check to complete
                            gloo_timers::future::TimeoutFuture::new(500).await;
                            
                            // Get window state to check result
                            if let Some(window) = web_sys::window() {
                                if let Ok(has_active) = js_sys::Reflect::get(&window, &"hasActiveDevices".into()) {
                                    if !has_active.as_bool().unwrap_or(true) {
                                        // No active devices, get available devices
                                        get_devices().await;
                                        
                                        // Wait for devices to be fetched
                                        gloo_timers::future::TimeoutFuture::new(500).await;
                                        
                                        // Get first available device and activate it
                                        if let Ok(devices) = js_sys::Reflect::get(&window, &"availableDevices".into()) {
                                            if let Some(devices_array) = devices.dyn_ref::<js_sys::Array>() {
                                                if devices_array.length() > 0 {
                                                    if let Ok(device) = js_sys::Reflect::get(&devices_array.get(0), &"id".into()) {
                                                        if let Some(device_id) = device.as_string() {
                                                            use crate::api_request::track_status::activate_device;
                                                            activate_device(device_id).await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0); // Add some space from the top
                        if ui.add_sized([200.0, 50.0], egui::Button::new("Connect with Spotify")).clicked() {
                            loginWithSpotify();
                        }
                    });
                }
            });
        }

        // Show saved tracks window in a separate scope
        show_saved_tracks_window(ctx);
        super::settings::show_settings_window(ctx);
        super::collage::show_collage_window(ctx);
        super::playlists_window::show_playlists_window(ctx);
        show_playlist_tracks_windows(ctx); // Call the new function
        
        // Check loading state in a separate scope
        let is_loading = {
            let state = APP_STATE.lock().unwrap();
            state.is_loading
        };
        
        if is_loading {
            ctx.request_repaint();
        }

        if self.show_player {
            // call the media player widget here
            super::super::mediaplayer::mediaplayerwidget::show_mediaplayer_window(ctx);
        }
    }
}
