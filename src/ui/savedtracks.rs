use super::app_state::{ViewMode, APP_STATE};
use egui::Context;
use crate::ui::tracks_ui::{show_list_view, show_grid_view};

pub fn show_saved_tracks_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if (!state.show_tracks) {
        return;
    }

    let tracks = state.saved_tracks.clone();
    let is_loading = state.is_loading;
    let total_tracks = state.total_tracks;
    let view_mode = state.view_mode;
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
            if state.is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    if let Some(total) = state.total_tracks {
                        ui.label(format!(
                            "Loading tracks... ({} of {} loaded)",
                            state.saved_tracks.len(),
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
                        state.view_mode = ViewMode::List;  // Only changes saved tracks view mode
                        window_size = (400.0, 600.0);
                    }
                    ui.add_space(8.0);
                    if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).clicked() {
                        window_size = (800.0, 600.0);
                        state.view_mode = ViewMode::Grid;  // Only changes saved tracks view mode
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
                                                crate::api_request::saved_tracks::load_more_tracks(token, false).await;
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
