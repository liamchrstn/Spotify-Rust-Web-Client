use egui::Context;
use crate::ui::app_state::{APP_STATE, ViewMode};
use crate::ui::tracks_ui::{show_list_view, show_grid_view, ListViewMode};

pub fn show_playlists_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if (!state.show_playlists) {
        return;
    }

    let playlists = state.playlists.clone();
    let view_mode = state.playlist_view_mode;  // Use playlist-specific view mode
    let mut window_size = state.playlists_window_size;
    let mut playlists_window_open = state.playlists_window_open;

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

            let filtered: Vec<_> = playlists.iter().collect();
            match view_mode {
                ViewMode::List => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        show_list_view(ui, &filtered, ListViewMode::Playlists);
                    });
                },
                ViewMode::Grid => {
                    show_grid_view(ui, &filtered, None, playlists.len(), playlists.len() as i32, ListViewMode::Playlists);
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