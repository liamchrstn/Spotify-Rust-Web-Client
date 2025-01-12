use reqwest::Client;
use crate::ui::APP_STATE;
use crate::utils::log_error;

pub async fn fetch_playlist_tracks(playlist_id: String, token: String) {
    let client = Client::new();
    let url = format!(
        "https://api.spotify.com/v1/playlists/{}",
        playlist_id
    );

    let mut state = APP_STATE.lock().unwrap();
    state.playlist_tracks.clear();
    state.is_loading = true;
    drop(state);

    if let Ok(resp) = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let playlist_name = json["name"].as_str().unwrap_or("Selected Playlist").to_string();
                let items = json["tracks"]["items"].as_array().cloned().unwrap_or_default();
                let tracks_data = items.into_iter()
                    .filter_map(|item| {
                        let track = item["track"].as_object()?;
                        let name = track["name"].as_str()?.to_string();
                        
                        let artists = track["artists"].as_array()?
                            .iter()
                            .filter_map(|artist| artist["name"].as_str())
                            .collect::<Vec<_>>()
                            .join(", ");

                        let album = track["album"].as_object()?;
                        let image_url = album["images"].as_array()?
                            .first()?["url"].as_str()?
                            .to_string();
                            
                        let uri = track["uri"].as_str()?.to_string();

                        Some((name, artists, image_url, uri))
                    })
                    .collect::<Vec<_>>();

                let mut state = APP_STATE.lock().unwrap();
                state.playlist_tracks = tracks_data;
                state.selected_playlist_name = Some(playlist_name);
                state.show_playlist_tracks_window = true;
                state.playlist_tracks_window_open = true;
                state.is_loading = false;
            }
        } else {
            log_error(&format!("Failed to fetch playlist tracks: {}", resp.status()));
        }
    } else {
        log_error("Request error while fetching playlist tracks");
    }
}
