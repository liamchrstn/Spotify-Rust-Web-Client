use egui::Context;
use crate::ui::{APP_STATE, tracks_ui::{show_list_view, show_grid_view, ListViewMode}};
use crate::ui::app_state::ViewMode;

// Fetch playlist tracks from Spotify (replace endpoint params as needed).
pub async fn fetch_playlist_tracks(playlist_id: String, token: String) {
    // ...call Spotify /playlists/{playlist_id}/tracks endpoint...
    // ...store results in APP_STATE.lock().unwrap().playlist_tracks...
}

pub fn show_playlist_tracks_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    if !state.show_playlist_tracks_window {
        return;
    }

    let tracks = state.playlist_tracks.clone();
    let view_mode = state.view_mode;
    let mut window_open = state.playlist_tracks_window_open;
    let window_pos = state.playlist_tracks_window_pos;
    let playlist_name = state.selected_playlist_name.clone().unwrap_or_else(|| "Selected Playlist".to_string());
    
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
        state.playlist_tracks_window_pos = (r.min.x, r.min.y);
    }

    state.playlist_tracks_window_open = window_open;
}
