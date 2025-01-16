use super::models::UserProfile;
use crate::ui::{set_username, set_user_id};
use reqwest::Client;
use crate::api_request::spotify_apis::handle_response;

// Fetches the user's profile from Spotify and sets the username and user ID in the app state
pub async fn fetch_user_profile(token: String) {
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_response(response, |user: UserProfile| {
        if let Some(name) = user.display_name {
            set_username(name);
        }
        if let Some(id) = user.id {
            set_user_id(id);
        }
    }).await;
}
