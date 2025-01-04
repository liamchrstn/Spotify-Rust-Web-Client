use super::models::{UserProfile, SavedTracksResponse};  // Changed from crate::models
use crate::utils::{log_error, clear_token_and_redirect};
use crate::ui::{APP_STATE, set_username};  // Changed from crate::app_state
use crate::storage::{load_tracks, save_tracks};
use reqwest::Client;

pub async fn fetch_user_profile(token: String) {
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status() == 401 {
                clear_token_and_redirect();
            } else if response.status().is_success() {
                match response.json::<UserProfile>().await {
                    Ok(user) => {
                        if let Some(name) = user.display_name {
                            set_username(name);
                        }
                    }
                    Err(err) => {
                        log_error(&format!("Failed to parse user profile: {:?}", err));
                    }
                }
            } else {
                log_error(&format!("Failed to fetch user profile: {:?}", response.status()));
            }
        }
        Err(err) => {
            log_error(&format!("Request error: {:?}", err));
        }
    }
}

pub async fn fetch_saved_tracks(token: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.is_loading = true;
    state.show_tracks = true;  // Show window immediately
    drop(state); // Release lock before async operations

    // Try to load from storage first
    if let Some(stored_tracks) = load_tracks() {
        let mut state = APP_STATE.lock().unwrap();
        state.saved_tracks = stored_tracks.tracks;
        state.total_tracks = Some(stored_tracks.total);
        state.is_loading = false;
        return;
    }

    let client = reqwest::Client::new();
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

        match response {
            Ok(response) => {
                match response.status().as_u16() {
                    200 => {
                        match response.json::<SavedTracksResponse>().await {
                            Ok(tracks) => {
                                let track_info: Vec<(String, String, String)> = tracks.items
                                    .into_iter()
                                    .map(|item| {
                                        let artists = item.track.artists
                                            .iter()
                                            .map(|artist| artist.name.clone())
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        
                                        // Get smallest image for list view
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
                                        // Save to storage when all tracks are fetched
                                        if let Err(e) = save_tracks(&state.saved_tracks, total) {
                                            log_error(&format!("Failed to save tracks to storage: {}", e));
                                        }
                                        state.is_loading = false;
                                        break;
                                    }
                                }
                                offset += limit;
                            }
                            Err(err) => {
                                let mut state = APP_STATE.lock().unwrap();
                                state.is_loading = false;
                                log_error(&format!("Failed to parse saved tracks: {:?}", err));
                                break;
                            }
                        }
                    }
                    401 => {
                        let mut state = APP_STATE.lock().unwrap();
                        state.is_loading = false;
                        clear_token_and_redirect();
                        break;
                    }
                    status => {
                        let mut state = APP_STATE.lock().unwrap();
                        state.is_loading = false;
                        log_error(&format!("Failed to fetch saved tracks: {}", status));
                        break;
                    }
                }
            }
            Err(err) => {
                let mut state = APP_STATE.lock().unwrap();
                state.is_loading = false;
                log_error(&format!("Request error: {:?}", err));
                break;
            }
        }
    }
}
