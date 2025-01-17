use egui::Context;
use crate::ui::{APP_STATE, tracks_ui::{show_list_view, show_grid_view, ListViewMode}};
use crate::ui::app_state::ViewMode;

pub fn show_playlist_tracks_windows(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    let playlist_windows = state.playlist_windows.clone();
    let user_id = state.user_id.clone().unwrap_or_default(); // Convert to String
    drop(state);

    for (playlist_id, playlist_name, tracks, mut view_mode, window_open, window_pos) in playlist_windows {
        let mut window_open = window_open;
        let playlist_id_clone = playlist_id.clone();
        let window = egui::Window::new(&playlist_name)
            .open(&mut window_open)
            .current_pos(window_pos)
            .default_size((600.0, 400.0))
            .show(ctx, |ui| {
                // Add view mode controls
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).clicked() {
                            let mut state = APP_STATE.lock().unwrap();
                            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id_clone) {
                                window_state.3 = ViewMode::List;
                            }
                            view_mode = ViewMode::List;
                        }
                        ui.add_space(8.0);
                        if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).clicked() {
                            let mut state = APP_STATE.lock().unwrap();
                            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id_clone) {
                                window_state.3 = ViewMode::Grid;
                            }
                            view_mode = ViewMode::Grid;
                        }
                        ui.label("View:");
                    });
                });
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match view_mode {
                        ViewMode::List => {
                            let enumerated: Vec<_> = tracks.iter().enumerate().collect();
                            show_list_view(
                                ui,
                                &enumerated,
                                ListViewMode::Tracks,
                                Some(&playlist_id),
                                &user_id
                            );
                        },
                        ViewMode::Grid => {
                            let enumerated: Vec<_> = tracks.iter().enumerate().collect();
                            show_grid_view(
                                ui,
                                &enumerated,
                                None,
                                tracks.len(),
                                tracks.len() as i32,
                                ListViewMode::Tracks,
                                Some(&playlist_id),
                                &user_id
                            );
                        }
                    }
                });
            });

        if let Some(resp) = window {
            let r = resp.response.rect;
            let mut state = APP_STATE.lock().unwrap();
            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id) {
                window_state.5 = (r.min.x, r.min.y);
                window_state.4 = window_open;
            }
        }
    }
    // Ensure the state is updated correctly to allow reopening
    let mut state = APP_STATE.lock().unwrap();
    state.playlist_windows.retain(|(_, _, _, _, window_open, _)| *window_open);
}
