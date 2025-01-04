use super::app_state::APP_STATE;  // Changed from crate::app_state
use crate::api_request::{fetch_saved_tracks, ACCESS_TOKEN};  // Changed imports
use crate::loginWithSpotify;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use egui_theme_switch::global_theme_switch;
use super::savedtracks::show_saved_tracks_window;

#[derive(Default)]
pub struct SpotifyApp {}

impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut state = APP_STATE.lock().unwrap();
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(name) = &state.username {
                        ui.heading(format!("Welcome, {}", name));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            global_theme_switch(ui);
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
                if let Some(name) = &state.username {                 
                    if ui.button("View Your Liked Songs").clicked() {
                        state.show_tracks = true;
                        state.tracks_window_open = true;
                        let token = ACCESS_TOKEN.lock().unwrap().clone().unwrap();
                        spawn_local(async {
                            fetch_saved_tracks(token).await;
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
        
        // Check loading state in a separate scope
        let is_loading = {
            let state = APP_STATE.lock().unwrap();
            state.is_loading
        };
        
        if is_loading {
            ctx.request_repaint();
        }
    }
}