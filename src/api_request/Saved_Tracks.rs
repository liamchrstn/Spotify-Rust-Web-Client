use super::models::SavedTracksResponse;
use crate::utils::log_error;
use crate::ui::APP_STATE;
use crate::storage::{load_tracks, save_tracks};
use reqwest::Client;

// Fetches the user's saved tracks from Spotify and updates the app state
pub async fn fetch_saved_tracks(token: String) {
    // Set loading state
    let mut state = APP_STATE.lock().unwrap();
    state.is_loading = true;
    state.show_tracks = true;
    state.loaded_tracks_count = 0;
    drop(state);

    // Add small delay to ensure loading state is visible
    gloo_timers::future::TimeoutFuture::new(100).await;

    // Try to load from storage first
    if let Some(stored_tracks) = load_tracks() {
        let mut state = APP_STATE.lock().unwrap();
        let initial_load = state.tracks_per_load;
        state.total_tracks = Some(stored_tracks.total);
        
        // Keep all tracks in storage but only load initial batch into state
        state.saved_tracks = stored_tracks.tracks[..initial_load as usize].to_vec();
        state.loaded_tracks_count = state.saved_tracks.len() as i32;
        state.is_loading = false;
        
        // Save all tracks back to storage in their original order
        if let Err(e) = save_tracks(&stored_tracks.tracks, stored_tracks.total) {
            log_error(&format!("Failed to save tracks to storage: {}", e));
        }
        return;
    }

    load_more_tracks(token, true).await;
}

async fn fetch_tracks_batch(client: &Client, token: &str, offset: usize, limit: i32) -> Option<SavedTracksResponse> {
    let url = format!(
        "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
        limit.min(50), // Ensure we don't exceed API limit
        offset
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(resp) => {
            match resp.json::<SavedTracksResponse>().await {
                Ok(tracks) => Some(tracks),
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}

pub async fn load_more_tracks(token: String, is_initial: bool) {
    // Try loading from storage first
    if let Some(stored_tracks) = load_tracks() {
        let mut state = APP_STATE.lock().unwrap();
        let offset = state.loaded_tracks_count as usize;
        let desired_limit = if state.tracks_per_load >= 1000 {
            if let Some(total) = state.total_tracks {
                total - state.loaded_tracks_count
            } else {
                1000
            }
        } else {
            state.tracks_per_load
        };

        // Load next batch from storage using array slicing to maintain order
        let end_idx = (offset + desired_limit as usize).min(stored_tracks.tracks.len());
        let next_batch = stored_tracks.tracks[offset..end_idx].to_vec();
        
        if !next_batch.is_empty() {
            let batch_len = next_batch.len();
            state.saved_tracks.extend(next_batch);
            
            // Save all tracks back to storage in their original order
            if let Err(e) = save_tracks(&stored_tracks.tracks, stored_tracks.total) {
                log_error(&format!("Failed to save tracks to storage: {}", e));
            }
            state.loaded_tracks_count += batch_len as i32;
            state.is_loading = false;
            return;
        }
        drop(state);
    }

    // If storage is empty or we've loaded all stored tracks, fetch from API
    let client = Client::new();
    let mut state = APP_STATE.lock().unwrap();
    let offset = state.loaded_tracks_count as usize;
    let desired_limit = if state.tracks_per_load >= 1000 {
        if let Some(total) = state.total_tracks {
            total - state.loaded_tracks_count
        } else {
            1000
        }
    } else {
        state.tracks_per_load
    };
    drop(state);

    // Add small delay between requests if not initial load
    if !is_initial {
        gloo_timers::future::TimeoutFuture::new(100).await;
    }

    // Calculate how many batches we need
    let num_batches = (desired_limit as f32 / 50.0).ceil() as i32;
    let mut remaining = desired_limit;

    for i in 0..num_batches {
        let current_offset = offset + (i as usize * 50);
        let current_limit = remaining.min(50);
        remaining -= current_limit;

        if let Some(tracks) = fetch_tracks_batch(&client, &token, current_offset, current_limit).await {
            let items_len = tracks.items.len();
            // Process tracks in order (newest first)
            let track_info: Vec<(String, String, String, String)> = tracks.items
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

                    (item.track.name, artists, image_url, item.track.uri)
                })
                .collect();
            
            let total = tracks.total;
            let mut state = APP_STATE.lock().unwrap();
            state.total_tracks = Some(total);
            state.saved_tracks.extend(track_info.into_iter());
            state.loaded_tracks_count += items_len as i32;
            
            if state.loaded_tracks_count >= total {
                if let Err(e) = save_tracks(&state.saved_tracks, total) {
                    log_error(&format!("Failed to save tracks to storage: {}", e));
                }
                state.is_loading = false;
                break;
            } else if is_initial || i == num_batches - 1 {
                state.is_loading = false;
            }
            drop(state);

            // Small delay between batches to avoid rate limiting
            if i < num_batches - 1 {
                gloo_timers::future::TimeoutFuture::new(50).await;
            }
        } else {
            // If a batch fails, stop loading
            APP_STATE.lock().unwrap().is_loading = false;
            break;
        }
    }
}
