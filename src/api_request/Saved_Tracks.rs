use super::models::SavedTracksResponse;
use crate::utils::{log_error, clear_token_and_redirect};
use crate::ui::APP_STATE;
use crate::storage::{load_tracks, save_tracks};
use reqwest::Client;
use crate::api_request::spotify_apis::handle_response;

// Fetches the user's saved tracks from Spotify and updates the app state
pub async fn fetch_saved_tracks(token: String) {
    // Set loading state
    let mut state = APP_STATE.lock().unwrap();
    state.is_loading = true;
    state.show_tracks = true;
    drop(state);

    // Add small delay to ensure loading state is visible
    gloo_timers::future::TimeoutFuture::new(100).await;

    // Try to load from storage first
    if let Some(stored_tracks) = load_tracks() {
        let mut state = APP_STATE.lock().unwrap();
        state.saved_tracks = stored_tracks.tracks;
        state.total_tracks = Some(stored_tracks.total);
        state.is_loading = false;
        return;
    }

    let client = Client::new();
    let mut offset = 0;
    let limit = 50;

    loop {
        // Add small delay between requests
        gloo_timers::future::TimeoutFuture::new(100).await;

        let url = format!(
            "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
            limit, offset
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        handle_response(response, |tracks: SavedTracksResponse| {
            let track_info: Vec<(String, String, String)> = tracks.items
                .into_iter()
                .map(|item| {
                    let artists = item.track.artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    let image_url = item.track.album.images
                        .iter()
                        .min_by_key(|img| img.width.unwrap_or(i32::MAX))
                        .map(|img| img.url.clone())
                        .unwrap_or_default();

                    (item.track.name, artists, image_url)
                })
                .collect();
            
            let total = tracks.total;
            {
                let mut state = APP_STATE.lock().unwrap();
                state.total_tracks = Some(total);
                state.saved_tracks.extend(track_info);
                
                if offset + limit >= total as usize {
                    if let Err(e) = save_tracks(&state.saved_tracks, total) {
                        log_error(&format!("Failed to save tracks to storage: {}", e));
                    }
                    state.is_loading = false;
                }
            }
            offset += limit;
        }).await;

        if APP_STATE.lock().unwrap().is_loading == false {
            break;
        }
    }
}
