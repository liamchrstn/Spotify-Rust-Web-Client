use crate::app_state::APP_STATE;
use crate::spotify_apis::fetch_saved_tracks;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use crate::loginWithSpotify;
use crate::token::ACCESS_TOKEN;

#[derive(Default)]
pub struct SpotifyApp;

impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = APP_STATE.lock().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(name) = &state.username {
                ui.heading(format!("Welcome, {}", name));
                
                if ui.button("Show Saved Tracks").clicked() {
                    state.show_tracks = true;
                    state.tracks_window_open = true;
                    let token = ACCESS_TOKEN.lock().unwrap().clone().unwrap(); // Fetch token from ACCESS_TOKEN
                    spawn_local(async {
                        fetch_saved_tracks(token).await;
                    });
                }

                if state.show_tracks {
                    // Clone the data we need before the window closure
                    let tracks = state.saved_tracks.clone();
                    let window_size = state.tracks_window_size;
                    let is_loading = state.is_loading;
                    
                    egui::Window::new("Saved Tracks")
                        .open(&mut state.tracks_window_open)
                        .default_size(window_size)
                        .resizable(true)
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if is_loading {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Loading tracks...");
                                    });
                                } else {
                                    for (track, artists) in &tracks {
                                        ui.horizontal(|ui| {
                                            let text = format!("{} - {}", track, artists);
                                            let available_width = ui.available_width();
                                            let text_width = ui.fonts(|fonts| fonts.layout_no_wrap(text.clone(), egui::FontId::default(), egui::Color32::WHITE).size().x);
                                            let truncated_text = if text_width > available_width {
                                                format!("{}...", &text[..(available_width as usize / 10)]) // Approximation
                                            } else {
                                                text
                                            };
                                            ui.add(egui::Label::new(truncated_text).wrap(false));
                                        });
                                        ui.separator();
                                    }
                                }
                            });
                        });
                }

                if ui.button("Logout").clicked() {
                    if let Some(window) = window() {
                        if let Ok(local_storage) = window.local_storage() {
                            if let Some(storage) = local_storage {
                                let _ = storage.remove_item("spotify_token");
                            }
                        }
                    }
                    state.username = None;
                    state.saved_tracks.clear();
                    state.show_tracks = false;
                }
            } else {
                if ui.button("Login with Spotify").clicked() {
                    loginWithSpotify();
                }
            }
        });
    }
}