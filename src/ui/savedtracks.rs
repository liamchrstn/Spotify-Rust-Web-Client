use super::app_state::{ViewMode, APP_STATE};  // Changed from crate::app_state
use egui::{Context, Ui};
use egui_extras::{TableBuilder, Column};
use crate::api_request::imagerender::get_or_load_image;

fn draw_vlines<R>(ui: &mut Ui, _height: f32, draw_left: bool, next: impl FnOnce(&mut Ui) -> R) {
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let rect = ui.available_rect_before_wrap();
    next(ui);
    if draw_left {
        ui.painter().vline(
            rect.left(),
            rect.top()..=rect.bottom(),
            stroke
        );
    }
}

pub fn show_saved_tracks_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if (!state.show_tracks) {
        return;
    }

    let tracks = state.saved_tracks.clone();
    let is_loading = state.is_loading;
    let total_tracks = state.total_tracks;
    let mut view_mode = state.view_mode;
    let mut window_size = state.tracks_window_size;
    let mut tracks_window_open = state.tracks_window_open;
    
    let window = egui::Window::new("Liked Songs")
        .open(&mut tracks_window_open)
        .current_pos([
            state.liked_songs_window_pos.0, 
            state.liked_songs_window_pos.1
        ])
        .default_size(window_size)
        .min_width(300.0)
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
                // Search on the left
                ui.horizontal(|ui| {
                    ui.label(format!("{} Search:", egui_phosphor::bold::MAGNIFYING_GLASS));
                    // Calculate desired width based on text content, with minimum width
                    let desired_width = (state.search_text.len() as f32 * 8.0).max(100.0);
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut state.search_text)
                            .desired_width(desired_width)
                    );
                    if search_response.changed() {
                        // Search text updated, no need to do anything as we'll filter below
                    }
                });

                // Push view controls to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).clicked() {
                        state.view_mode = ViewMode::List;
                        window_size = (400.0, 600.0);
                    }
                    ui.add_space(8.0);
                    if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).clicked() {
                        window_size = (800.0, 600.0);
                        state.view_mode = ViewMode::Grid;
                    }
                    ui.label("View:");
                });
            });
            ui.add_space(8.0);

            // Filter tracks based on search text
            let filtered_tracks: Vec<_> = tracks.iter()
                .filter(|(track, artists, _, _)| {
                    let search_lower = state.search_text.to_lowercase();
                    track.to_lowercase().contains(&search_lower) || 
                    artists.to_lowercase().contains(&search_lower)
                })
                .collect();

            match view_mode {
                ViewMode::List => {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            show_list_view(ui, &filtered_tracks);

                            // Add Load More button only at the bottom after showing all tracks
                            if let Some(total) = total_tracks {
                                if filtered_tracks.len() >= state.saved_tracks.len() && state.loaded_tracks_count < total {
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(ui.available_width() / 2.0 - 50.0); // Center the button
                                        if ui.button("Load More").clicked() {
                                            let token = web_sys::window()
                                                .and_then(|window| window.local_storage().ok().flatten())
                                                .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                                .unwrap_or_default();
                                            
                                            wasm_bindgen_futures::spawn_local(async move {
                                                crate::api_request::Saved_Tracks::load_more_tracks(token, false).await;
                                            });
                                        }
                                    });
                                }
                            }
                        });
                },
                ViewMode::Grid => show_grid_view(ui, &filtered_tracks, total_tracks, state.saved_tracks.len(), state.loaded_tracks_count),
            }
        });
        
    if let Some(resp) = window {
        let r = resp.response.rect;
        // Always update position since this window isn't being reset
        state.liked_songs_window_pos = (r.min.x, r.min.y);
    }

    state.tracks_window_open = tracks_window_open;
}

fn show_list_view(ui: &mut Ui, tracks: &[&(String, String, String, String)]) {
    for (track, artists, image_url, uri) in tracks {
        let row_response = ui.horizontal(|ui| {
            // Add album art
            if let Some(image) = get_or_load_image(ui.ctx(), image_url) {
                ui.add(image.fit_to_exact_size([40.0, 40.0].into()));
            }
            
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
        }).response;

        // Make the row clickable
        if row_response.interact(egui::Sense::click()).clicked() {
            let uri = uri.clone();
            wasm_bindgen_futures::spawn_local(async move {
                crate::api_request::Track_Status::play_track(uri).await;
            });
        }
        
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
    }
}

fn show_grid_view(ui: &mut Ui, tracks: &[&(String, String, String, String)], total_tracks: Option<i32>, saved_tracks_len: usize, loaded_tracks_count: i32) {
    let available_width = ui.available_width();
    let column_width = (available_width / 3.0).max(100.0) - 10.0; // Add padding
    
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
                                            if let Some((track, artists, image_url, uri)) = tracks.get(idx) {
                                                row.col(|ui| {
                                                    let cell_response = ui.scope(|ui| {
                                    draw_vlines(ui, 100.0, col > 0, |ui| {
                                        ui.horizontal(|ui| {
                                            // Add album art
                                            if let Some(image) = get_or_load_image(ui.ctx(), image_url) {
                                                ui.add(image.fit_to_exact_size([80.0, 80.0].into()));
                                            }
                                            ui.add_space(8.0);
                                            ui.vertical(|ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(track)
                                                            .size(16.0)
                                                            .strong()
                                                            .color(ui.visuals().strong_text_color())
                                                    ).wrap()
                                                );
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(artists)
                                                            .size(14.0)
                                                            .color(ui.visuals().weak_text_color())
                                                    ).wrap()
                                                );
                                                    });
                                                    
                                                    // Make the cell clickable
                                                    if ui.rect_contains_pointer(ui.min_rect()) {
                                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                    }
                                                    if ui.rect_contains_pointer(ui.min_rect()) && ui.input(|i| i.pointer.primary_clicked()) {
                                                        let uri = uri.clone();
                                                        wasm_bindgen_futures::spawn_local(async move {
                                                            crate::api_request::Track_Status::play_track(uri).await;
                                                        });
                                                    }
                                                });
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

                // Add Load More button only after the last row if we have more tracks to load
                if let Some(total) = total_tracks {
                    if tracks.len() >= saved_tracks_len && loaded_tracks_count < total {
                        body.row(50.0, |mut row| {
                            // Use all three columns for the button
                            row.col(|_| {});  // Empty first column
                            row.col(|ui| {
                                // Center the button in the middle column
                                if ui.button("Load More").clicked() {
                                    let token = web_sys::window()
                                        .and_then(|window| window.local_storage().ok().flatten())
                                        .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                        .unwrap_or_default();
                                    
                                    wasm_bindgen_futures::spawn_local(async move {
                                        crate::api_request::Saved_Tracks::load_more_tracks(token, false).await;
                                    });
                                }
                            });
                            row.col(|_| {});  // Empty third column
                        });
                    }
                }
            });
    });
}
