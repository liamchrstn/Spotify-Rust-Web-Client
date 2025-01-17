use egui::Context;
use crate::ui::{APP_STATE, tracks_ui::{show_list_view, show_grid_view, ListViewMode}};
use crate::ui::app_state::ViewMode;
use egui::CursorIcon;

pub fn show_playlist_tracks_windows(ctx: &Context) {
    let state = APP_STATE.lock().unwrap();
    let playlist_windows = state.playlist_windows.clone();
    let user_id = state.user_id.clone().unwrap_or_default();
    drop(state); // Release lock to avoid conflicts

    for (playlist_id, playlist_name, tracks, mut view_mode, window_open, window_pos) in playlist_windows {
        let mut local_window_open = window_open;
        let mut state = APP_STATE.lock().unwrap();
        let constrain_rect = state.constrain_to_central_panel(ctx);
        drop(state);

        let window = egui::Window::new(&playlist_name)
            .open(&mut local_window_open)
            .current_pos(window_pos)
            .default_size((600.0, 400.0))
            .constrain_to(constrain_rect)
            .show(ctx, |ui| {
                // Add view mode controls
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            let mut state = APP_STATE.lock().unwrap();
                            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id) {
                                window_state.3 = ViewMode::List;
                            }
                            view_mode = ViewMode::List;
                        }
                        ui.add_space(8.0);
                        if ui.toggle_value(&mut (view_mode == ViewMode::Grid), &format!("{} Grid", egui_phosphor::bold::SQUARES_FOUR)).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                            let mut state = APP_STATE.lock().unwrap();
                            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id) {
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

        let mut state = APP_STATE.lock().unwrap();
        if let Some(resp) = window {
            let r = resp.response.rect;
            if let Some(window_state) = state.playlist_windows.iter_mut().find(|w| w.0 == playlist_id) {
                window_state.5 = (r.min.x, r.min.y);
                window_state.4 = local_window_open;
                window_state.3 = view_mode;
            }
        }
    }

    // Clean up closed windows
    let mut state = APP_STATE.lock().unwrap();
    state.playlist_windows.retain(|(_, _, _, _, w_open, _)| *w_open);
}
