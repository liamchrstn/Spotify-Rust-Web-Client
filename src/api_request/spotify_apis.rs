use super::models::{UserProfile, SavedTracksResponse, StoredTracks};
use crate::utils::{log_error, clear_token_and_redirect};
use crate::ui::{APP_STATE, set_username};  // Changed from crate::app_state
use crate::storage::{load_tracks, save_tracks};
use reqwest::Client;

pub use handle_response as other_handle_response;

// Handles the response from an API request, calling the success handler if the request is successful
pub async fn handle_response<T, F>(response: reqwest::Result<reqwest::Response>, success_handler: F)
where
    F: FnOnce(T) -> (),
    T: serde::de::DeserializeOwned,
{
    match response {
        Ok(response) => {
            if response.status() == 401 {
                clear_token_and_redirect();
            } else if response.status().is_success() {
                match response.json::<T>().await {
                    Ok(data) => success_handler(data),
                    Err(err) => log_error(&format!("Failed to parse response: {:?}", err)),
                }
            } else {
                log_error(&format!("Failed to fetch data: {:?}", response.status()));
            }
        }
        Err(err) => log_error(&format!("Request error: {:?}", err)),
    }
}
