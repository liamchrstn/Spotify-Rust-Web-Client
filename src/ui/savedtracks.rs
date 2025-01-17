use super::app_state::{ViewMode, APP_STATE};
use egui::Context;
use crate::ui::tracks_ui::{show_list_view, show_grid_view, ListViewMode};
use egui::CursorIcon;

pub fn show_saved_tracks_window(ctx: &Context) {
    let state = APP_STATE.lock().unwrap();
    if !state.show_tracks {
        return;
    }

    let tracks = state.saved_tracks.clone();
    let total_tracks = state.total_tracks;
    let view_mode = state.view_mode;
    let mut tracks_window_open = state.tracks_window_open;
    let user_id = state.user_id.clone().unwrap_or_default();
    let mut window_size = state.tracks_window_size; // Make window_size mutable
    let current_pos = state.liked_songs_window_pos;
    let is_loading = state.is_loading;
    let constrain_rect = state.constrain_to_central_panel(ctx);
    let mut search_text = state.search_text.clone();
    drop(state);

    let window = egui::Window::new("Liked Songs")
        .open(&mut tracks_window_open)
        .current_pos([current_pos.0, current_pos.1])
        .default_size(window_size)
        .min_width(300.0)
        .resizable(true)
        .constrain_to(constrain_rect)
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
                    let desired_width = (search_text.len() as f32 * 8.0).max(100.0);
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut search_text)
                            .desired_width(desired_width)
                    );
                    if search_response.changed() {
                        // Search text updated, no need to do anything as we'll filter below
                    }
                });

                // Push view controls to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        // Update view_mode locally
                        window_size = (400.0, 600.0);
                        // You might want to store view_mode back to state later
                    }
                    ui.add_space(8.0);
                    if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        window_size = (800.0, 600.0);
                        // You might want to store view_mode back to state later
                    }
                    ui.label("View:");
                });
            });
            ui.add_space(8.0);

            // Filter tracks based on search text
            let filtered_tracks: Vec<(usize, &(String, String, String, String))> = tracks
                .iter()
                .enumerate()
                .filter(|(_, (track, artists, _, _))| {
                    let search_lower = search_text.to_lowercase();
                    track.to_lowercase().contains(&search_lower)
                        || artists.to_lowercase().contains(&search_lower)
                })
                .collect();

            match view_mode {
                ViewMode::List => {
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            show_list_view(ui, &filtered_tracks, ListViewMode::Tracks, None, &user_id);

                            // Add Load More button only at the bottom after showing all tracks
                            if let Some(total) = total_tracks {
                                if filtered_tracks.len() >= tracks.len() {
                                    let loaded_tracks_count = APP_STATE.lock().unwrap().loaded_tracks_count;
                                    if loaded_tracks_count < total {
                                        ui.add_space(16.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(ui.available_width() / 2.0 - 50.0); // Center the button
                                            if ui.button("Load More").on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                                let token = web_sys::window()
                                                    .and_then(|window| window.local_storage().ok().flatten())
                                                    .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                                    .unwrap_or_default();
                                                
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    crate::api_request::saved_tracks::load_more_tracks(token, false).await;
                                                });
                                            }
                                        });
                                    }
                                }
                            }
                        });
                },
                ViewMode::Grid => show_grid_view(
                    ui,
                    &filtered_tracks,
                    total_tracks,
                    tracks.len(),
                    APP_STATE.lock().unwrap().loaded_tracks_count,
                    ListViewMode::Tracks,
                    None,
                    &user_id
                ),
            }
        });
        
    let mut state = APP_STATE.lock().unwrap();
    state.tracks_window_open = tracks_window_open;
    state.search_text = search_text;         // Update with modified search_text
    state.tracks_window_size = window_size; // Update with modified window_size
    if let Some(resp) = window {
        let r = resp.response.rect;
        state.liked_songs_window_pos = (r.min.x, r.min.y);
    }
}
