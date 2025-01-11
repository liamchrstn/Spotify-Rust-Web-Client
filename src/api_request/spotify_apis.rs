use super::models::{UserProfile, SavedTracksResponse, StoredTracks};
use crate::utils::{log_error, clear_token_and_redirect};
use crate::ui::{APP_STATE, set_username};  // Changed from crate::app_state
use crate::storage::{load_tracks, save_tracks};
use reqwest::Client;
use wasm_bindgen::prelude::*;
use web_sys;
use js_sys;

// Handles responses that don't return any content (204 No Content)
pub async fn handle_empty_response<F>(response: reqwest::Result<reqwest::Response>, success_handler: F)
where
    F: FnOnce() -> (),
{
    match response {
        Ok(response) => {
            if response.status() == 401 {
                clear_token_and_redirect();
            } else if response.status().is_success() {
                success_handler();
            } else {
                log_error(&format!("Failed to execute command: {:?}", response.status()));
            }
        }
        Err(err) => log_error(&format!("Request error: {:?}", err)),
    }
}

// Handles the response from an API request, calling the success handler if the request is successful
pub async fn handle_response<T, F>(response: reqwest::Result<reqwest::Response>, success_handler: F)
where
    F: FnOnce(T) -> (),
    T: serde::de::DeserializeOwned,
{
    // Set default window state in case of errors
    let set_default_state = || {
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::set(&window, &"currentPlayerState".into(), &wasm_bindgen::JsValue::NULL);
            let _ = js_sys::Reflect::set(&window, &"currentPlaybackTime".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"totalDuration".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &false.into());
        }
    };

    match response {
        Ok(response) => {
            if response.status() == 401 {
                clear_token_and_redirect();
                set_default_state();
            } else if response.status() == 204 {
                // No content, treat as success but with default state
                set_default_state();
            } else if response.status().is_success() {
                match response.json::<T>().await {
                    Ok(data) => success_handler(data),
                    Err(err) => {
                        log_error(&format!("Failed to parse response: {:?}", err));
                        set_default_state();
                    }
                }
            } else {
                log_error(&format!("Failed to fetch data: {:?}", response.status()));
                set_default_state();
            }
        }
        Err(err) => {
            log_error(&format!("Request error: {:?}", err));
            set_default_state();
        }
    }
}
