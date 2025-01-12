use egui::Context;
use crate::ui::app_state::{APP_STATE, ViewMode};
use crate::ui::tracks_ui::{show_list_view, show_grid_view};

pub fn show_playlists_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.show_playlists {
        return;
    }

    let playlists = state.playlists.clone();
    let view_mode = state.view_mode;
    let window_size = state.playlists_window_size;
    let mut playlists_window_open = state.playlists_window_open;

    let window = egui::Window::new("Your Playlists")
        .open(&mut playlists_window_open)
        .current_pos([state.playlists_window_pos.0, state.playlists_window_pos.1])
        .default_size(window_size)
        .show(ctx, |ui| {
            let filtered: Vec<_> = playlists.iter().collect();
            match view_mode {
                ViewMode::List => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        show_list_view(ui, &filtered);
                    });
                },
                ViewMode::Grid => {
                    show_grid_view(ui, &filtered, None, playlists.len(), playlists.len() as i32);
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