use egui::Context;
use crate::ui::app_state::{APP_STATE, ViewMode};
use crate::ui::tracks_ui::{show_grid_view, ListViewMode, render_square_with_image};

pub fn show_playlists_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.show_playlists {
        return;
    }

    let playlists = state.playlists.clone();
    let view_mode = state.playlist_view_mode;  // Use playlist-specific view mode
    let mut window_size = state.playlists_window_size;
    let mut playlists_window_open = state.playlists_window_open;
    let user_id = state.user_id.clone().unwrap_or_default(); // Convert to String

    let window = egui::Window::new("Your Playlists")
        .open(&mut playlists_window_open)
        .current_pos([state.playlists_window_pos.0, state.playlists_window_pos.1])
        .default_size(window_size)
        .show(ctx, |ui| {
            // Add view mode controls
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).clicked() {
                        state.playlist_view_mode = ViewMode::List;
                        window_size = (400.0, 600.0);
                    }
                    ui.add_space(8.0);
                    if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).clicked() {
                        window_size = (800.0, 600.0);
                        state.playlist_view_mode = ViewMode::Grid;
                    }
                    ui.label("View:");
                });
            });
            ui.add_space(8.0);

            let filtered: Vec<(String, String, String, String)> = playlists
                .iter()
                .map(|(name, owner, image_url, id, total_tracks)| 
                    (name.clone(), owner.clone(), image_url.clone(), id.clone())
                )
                .collect();
            

            match view_mode {
                ViewMode::List => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (name, owner, image_url, id, total_tracks) in playlists {
                            let row_response = ui.horizontal(|ui| {
                                render_square_with_image(ui, 40.0, &image_url);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&name)
                                                .size(16.0)
                                                .strong()
                                                .color(ui.visuals().strong_text_color())
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(" • {} tracks", total_tracks))
                                                .size(14.0)
                                                .color(ui.visuals().weak_text_color())
                                        );
                                    });
                                    ui.label(
                                        egui::RichText::new(&owner)
                                            .size(14.0)
                                            .color(ui.visuals().weak_text_color())
                                    );
                                });
                            }).response;

                            // Make the row clickable
                            if row_response.interact(egui::Sense::click()).clicked() {
                                let id = id.clone();
                                let token = web_sys::window()
                                    .and_then(|window| window.local_storage().ok().flatten())
                                    .and_then(|storage| storage.get_item("spotify_token").ok().flatten())
                                    .unwrap_or_default();
                                
                                wasm_bindgen_futures::spawn_local(async move {
                                    crate::api_request::playlist_tracks::fetch_playlist_tracks(id, token).await;
                                });
                            }
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);
                        }
                    });
                },
                ViewMode::Grid => {
                    let enumerated: Vec<_> = filtered.iter().enumerate().collect();
                    show_grid_view(
                        ui,
                        &enumerated,
                        None,
                        playlists.len(),
                        playlists.len() as i32,
                        ListViewMode::Playlists,
                        None,
                        &user_id
                    );
                },
            }
        }
    );

    if let Some(resp) = window {
        let r = resp.response.rect;
        state.playlists_window_pos = (r.min.x, r.min.y);
    }

    state.playlists_window_open = playlists_window_open;
}