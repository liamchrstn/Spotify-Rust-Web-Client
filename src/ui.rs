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
                    let total_tracks = state.total_tracks;
                    
                    egui::Window::new("Saved Tracks")
                        .open(&mut state.tracks_window_open)
                        .default_size(window_size)
                        .resizable(true)
                        .show(ctx, |ui| {
                            if is_loading {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    if let Some(total) = total_tracks {
                                        ui.label(format!(
                                            "Loading tracks... ({} of {} loaded)", 
                                            tracks.len(), 
                                            total
                                        ));
                                    } else {
                                        ui.label("Loading tracks...");
                                    }
                                });
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);
                            }

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (track, artists) in &tracks {
                                    ui.vertical(|ui| {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(track)
                                                .size(16.0)
                                                .strong()
                                        ));
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(artists)
                                                .size(14.0)
                                                .color(egui::Color32::LIGHT_GRAY)
                                        ));
                                    });
                                    ui.add_space(4.0);
                                    ui.separator();
                                    ui.add_space(4.0);
                                }
                                
                                if is_loading {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Loading more tracks...");
                                    });
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
        
        // Request repaint while loading to ensure smooth updates
        if state.is_loading {
            ctx.request_repaint();
        }
    }
}