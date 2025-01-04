use crate::app_state::{ViewMode, APP_STATE};
use crate::spotify_apis::fetch_saved_tracks;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use crate::loginWithSpotify;
use crate::token::ACCESS_TOKEN;
use egui_theme_switch::global_theme_switch;
use egui_extras::{TableBuilder, Column};
use egui::{Rgba, Stroke as PathStroke, Ui};

fn draw_vlines<R>(ui: &mut Ui, height: f32, draw_left: bool, next: impl FnOnce(&mut Ui) -> R) {
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let rect = ui.available_rect_before_wrap();
    next(ui);
    // Only draw left line if requested (for middle columns)
    if draw_left {
        ui.painter().vline(
            rect.left(),
            rect.top()..=rect.bottom(),
            stroke
        );
    }
}

#[derive(Default)]
pub struct SpotifyApp {
}

impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = APP_STATE.lock().unwrap();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Spotify App");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    global_theme_switch(ui);  // Remove extra semicolon
                });
            });
        });

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
                    // Clone all needed values before the window closure
                    let tracks = state.saved_tracks.clone();
                    let window_size = state.tracks_window_size;
                    let is_loading = state.is_loading;
                    let total_tracks = state.total_tracks;
                    let mut view_mode = state.view_mode;
                    let mut tracks_window_open = state.tracks_window_open;
                    
                    egui::Window::new("Saved Tracks")
                        .open(&mut tracks_window_open)
                        .default_size(window_size)
                        .min_width(300.0)          // Add minimum width
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

                            ui.horizontal(|ui| {
                                ui.label("View:");
                                if ui.radio_value(&mut view_mode, ViewMode::List, "List").clicked() {
                                    state.tracks_window_size = (400.0, 600.0);
                                    state.view_mode = ViewMode::List;
                                }
                                if ui.radio_value(&mut view_mode, ViewMode::Grid, "Grid").clicked() {
                                    state.tracks_window_size = (800.0, 600.0);  // Reduced from 1000.0 to 800.0 for default grid width
                                    state.view_mode = ViewMode::Grid;
                                }
                            });
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                match view_mode {
                                    ViewMode::List => {
                                        for (track, artists) in &tracks {
                                            ui.vertical(|ui| {
                                                ui.add(egui::Label::new(
                                                    egui::RichText::new(track)
                                                        .size(16.0)
                                                        .strong()
                                                        .color(ui.visuals().strong_text_color())
                                                ).wrap());
                                                
                                                ui.add(egui::Label::new(
                                                    egui::RichText::new(artists)
                                                        .size(14.0)
                                                        .color(ui.visuals().weak_text_color())
                                                ).wrap());
                                            });
                                            ui.add_space(4.0);
                                            ui.separator();
                                            ui.add_space(4.0);
                                        }
                                    }
                                    ViewMode::Grid => {
                                        let available_width = ui.available_width();
                                        let column_width = (available_width / 3.0).max(100.0);
                                        
                                        egui::ScrollArea::horizontal().show(ui, |ui| {
                                            TableBuilder::new(ui)
                                                .striped(true)
                                                .resizable(false)
                                                .cell_layout(egui::Layout::top_down_justified(egui::Align::Center))
                                                .column(Column::exact(column_width))
                                                .column(Column::exact(column_width))
                                                .column(Column::exact(column_width))
                                                .vscroll(true)
                                                .body(|mut body| {
                                                    let rows = (tracks.len() + 2) / 3;
                                                    for row_idx in 0..rows {
                                                        body.row(100.0, |mut row| {
                                                            for col in 0..3 {
                                                                let idx = row_idx * 3 + col;
                                                                if let Some((track, artists)) = tracks.get(idx) {
                                                                    row.col(|ui| {
                                                                        draw_vlines(ui, 100.0, col > 0, |ui| {
                                                                            ui.horizontal(|ui| {
                                                                                ui.add_space(12.0); // Left padding from divider
                                                                                ui.vertical(|ui| {
                                                                                    // Track name - using strong text color
                                                                                    ui.add(
                                                                                        egui::Label::new(
                                                                                            egui::RichText::new(track)
                                                                                                .size(16.0)
                                                                                                .strong()
                                                                                                .color(ui.visuals().strong_text_color())
                                                                                        ).wrap()
                                                                                    );
                                                                                    // Artist name - using weak text color
                                                                                    ui.add(
                                                                                        egui::Label::new(
                                                                                            egui::RichText::new(artists)
                                                                                                .size(14.0)
                                                                                                .color(ui.visuals().weak_text_color())
                                                                                        ).wrap()
                                                                                    );
                                                                                });
                                                                                ui.add_space(12.0); // Right padding before next divider
                                                                            });
                                                                        });
                                                                    });
                                                                } else {
                                                                    row.col(|ui| {
                                                                        draw_vlines(ui, 100.0, col > 0, |_| {});
                                                                    });
                                                                }
                                                            }
                                                        });
                                                    }
                                                });
                                        });
                                    }
                                }
                                
                                if is_loading {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Loading more tracks...");
                                    });
                                }
                            });
                        });
                        
                    // Update state after window closure
                    state.tracks_window_open = tracks_window_open;
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