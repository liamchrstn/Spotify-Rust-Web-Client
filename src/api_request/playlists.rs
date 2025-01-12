use reqwest::Client;
use crate::ui::APP_STATE;
use crate::utils::log_error;

pub async fn fetch_playlists(token: String) {
    let client = Client::new();
    let url = "https://api.spotify.com/v1/me/playlists?limit=50";

    if let Ok(resp) = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let items_value = json["items"].clone();
                let items_vec = items_value.as_array().cloned().unwrap_or_default();
                let mut playlists_data = vec![];
                for item in &items_vec {
                    let images_value = item["images"].clone();
                    let images_vec = images_value.as_array().cloned().unwrap_or_default();
                    let name = item["name"].as_str().unwrap_or("").to_string();
                    let owner = item["owner"]["display_name"].as_str().unwrap_or("").to_string();
                    let image_url = images_vec
                        .get(0)
                        .and_then(|img| img["url"].as_str())
                        .unwrap_or("")
                        .to_string();
                    let id = item["id"].as_str().unwrap_or("").to_string();
                    playlists_data.push((name, owner, image_url, id));
                }
                let mut state = APP_STATE.lock().unwrap();
                state.playlists = playlists_data;
                state.show_playlists = true;
                state.playlists_window_open = true;
            }
        } else {
            log_error(&format!("Failed to fetch playlists: {}", resp.status()));
        }
    } else {
        log_error("Request error while fetching playlists.");
    }
}
