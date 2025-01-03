use crate::models::{UserProfile, SavedTracksResponse};
use crate::utils::{log_error, clear_token_and_redirect};
use crate::app_state::{APP_STATE, set_username};
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
    {
        let mut state = APP_STATE.lock().unwrap();
        state.is_loading = true;
    }

    let client = reqwest::Client::new();
    
    // Only fetch first 50 tracks
    let url = "https://api.spotify.com/v1/me/tracks?limit=50&offset=0";
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            match response.status().as_u16() {
                200 => {
                    match response.json::<SavedTracksResponse>().await {
                        Ok(tracks) => {
                            let track_info: Vec<(String, String)> = tracks.items
                                .into_iter()
                                .map(|item| {
                                    let artists = item.track.artists
                                        .iter()
                                        .map(|artist| artist.name.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    (item.track.name, artists)
                                })
                                .collect();
                            
                            let mut state = APP_STATE.lock().unwrap();
                            state.saved_tracks = track_info;
                            state.show_tracks = true;
                            state.is_loading = false;  // Set loading to false when done
                        }
                        Err(err) => {
                            let mut state = APP_STATE.lock().unwrap();
                            state.is_loading = false;  // Set loading to false on error
                            log_error(&format!("Failed to parse saved tracks: {:?}", err));
                        }
                    }
                }
                401 => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.is_loading = false;  // Set loading to false on error
                    clear_token_and_redirect();
                }
                status => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.is_loading = false;  // Set loading to false on error
                    log_error(&format!("Failed to fetch saved tracks: {}", status));
                }
            }
        }
        Err(err) => {
            let mut state = APP_STATE.lock().unwrap();
            state.is_loading = false;  // Set loading to false on error
            log_error(&format!("Request error: {:?}", err));
        }
    }
}

