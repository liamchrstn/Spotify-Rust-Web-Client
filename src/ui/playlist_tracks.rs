use egui::Context;
use crate::ui::{APP_STATE, tracks_ui::{show_list_view, show_grid_view, ListViewMode}};
use crate::ui::app_state::ViewMode;

// Fetch playlist tracks from Spotify (replace endpoint params as needed).
pub async fn fetch_playlist_tracks(playlist_id: String, token: String) {
    // ...call Spotify /playlists/{playlist_id}/tracks endpoint...
    // ...store results in APP_STATE.lock().unwrap().playlist_tracks...
}

pub fn show_playlist_tracks_windows(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    let playlist_windows = state.playlist_windows.clone();
    drop(state);

    for (playlist_id, playlist_name, tracks, view_mode, window_open, window_pos) in playlist_windows {
        let mut window_open = window_open;
        let window = egui::Window::new(&playlist_name)
            .open(&mut window_open)
            .current_pos(window_pos)
            .default_size((600.0, 400.0))
            .show(ctx, |ui| {
                match view_mode {
                    ViewMode::List => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            show_list_view(ui, &tracks.iter().collect::<Vec<_>>(), ListViewMode::Tracks);
                        });
                    }
                    ViewMode::Grid => {
                        show_grid_view(
                            ui,
                            &tracks.iter().collect::<Vec<_>>(),
                            None,
                            tracks.len(),
                            tracks.len() as i32,
                            ListViewMode::Tracks
                        );
                    }
                }
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
