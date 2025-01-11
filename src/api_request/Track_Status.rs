use super::models::{CurrentPlaybackResponse, PlayerStateResponse, DevicesResponse, Device};
use reqwest::Client;
use crate::api_request::spotify_apis::{handle_response, handle_empty_response};
use crate::api_request::token::get_token;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// Fetches the current playback state from Spotify and updates the window state
#[wasm_bindgen]
pub async fn skip_to_next(token: String) {
    let client = Client::new();
    let response = client
        .post("https://api.spotify.com/v1/me/player/next")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_empty_response(response, || {
        // Success, no response body needed
    }).await;
}

#[wasm_bindgen]
pub async fn skip_to_previous(token: String) {
    let client = Client::new();
    let response = client
        .post("https://api.spotify.com/v1/me/player/previous")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_empty_response(response, || {
        // Success, no response body needed
    }).await;
}

#[wasm_bindgen]
pub async fn toggle_shuffle(token: String) {
    let client = Client::new();
    
    // First get current state
    let response = client
        .get("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    let client_clone = client.clone();
    let token_clone = token.clone();
    handle_response(response, |state: PlayerStateResponse| {
        // Toggle the state
        let new_state = !state.shuffle_state;
        
        // Update shuffle state
        spawn_local(async move {
            let toggle_response = client_clone
                .put(&format!("https://api.spotify.com/v1/me/player/shuffle?state={}", new_state))
                .header("Authorization", format!("Bearer {}", token_clone))
                .send()
                .await;

            handle_empty_response(toggle_response, || {
                // Update window state on success
                let window = web_sys::window().expect("no global window exists");
                let _ = js_sys::Reflect::set(&window, &"shuffleState".into(), &new_state.into());
            }).await;
        });
    }).await;
}

#[wasm_bindgen]
pub async fn get_devices() {
    let token = if let Some(token) = get_token() {
        token
    } else {
        return;
    };
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_response(response, |devices: DevicesResponse| {
        let array = js_sys::Array::new();
        for device in devices.devices {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&obj, &"id".into(), &device.id.into());
            let _ = js_sys::Reflect::set(&obj, &"name".into(), &device.name.into());
            let _ = js_sys::Reflect::set(&obj, &"is_active".into(), &device.is_active.into());
            array.push(&obj);
        }
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"availableDevices".into(), &array);
    }).await;
}

#[wasm_bindgen]
pub async fn transfer_playback(device_id: String) {
    let token = if let Some(token) = get_token() {
        token
    } else {
        return;
    };
    let client = Client::new();
    let response = client
        .put("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "device_ids": [device_id],
            "play": true
        }))
        .send()
        .await;

    handle_empty_response(response, || {
        // Success, no response body needed
    }).await;
}

#[wasm_bindgen]
pub async fn activate_device(device_id: String) {
    let token = if let Some(token) = get_token() {
        token
    } else {
        return;
    };
    let client = Client::new();
    
    // First activate the device
    let response = client
        .put("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "device_ids": [device_id],
            "play": false
        }))
        .send()
        .await;

    handle_empty_response(response, || {}).await;

    // Wait for device activation
    web_sys::window()
        .expect("no global window exists")
        .set_timeout_with_callback_and_timeout_and_arguments_0(&js_sys::Function::new_no_args(""), 1000)
        .expect("failed to set timeout");

    // Verify device is active
    let verify_response = client
        .get("https://api.spotify.com/v1/me/player")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_response(verify_response, |_: PlayerStateResponse| {
        // Device activation successful
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"deviceActivated".into(), &true.into());
    }).await;
}

#[wasm_bindgen]
pub async fn has_active_devices() {
    get_devices().await;
    let window = web_sys::window().expect("no global window exists");
    if let Ok(devices) = js_sys::Reflect::get(&window, &"availableDevices".into()) {
        if let Some(devices_array) = devices.dyn_ref::<js_sys::Array>() {
            for i in 0..devices_array.length() {
                if let Ok(device) = js_sys::Reflect::get(&devices_array.get(i), &"is_active".into()) {
                    if device.as_bool().unwrap_or(false) {
                        let _ = js_sys::Reflect::set(&window, &"hasActiveDevices".into(), &true.into());
                        return;
                    }
                }
            }
        }
    }
    let _ = js_sys::Reflect::set(&window, &"hasActiveDevices".into(), &false.into());
}

#[wasm_bindgen]
pub async fn get_current_playback() {
    let token = if let Some(token) = get_token() {
        token
    } else {
        return;
    };
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_response(response, |playback: CurrentPlaybackResponse| {
        let window = web_sys::window().expect("no global window exists");
        
        // Update window state with playback information
        if let Some(track) = playback.item {
            // Create state objects
            let state = js_sys::Object::new();

            // Create track_window object
            let track_window = js_sys::Object::new();
            let current_track = js_sys::Object::new();
            
            // Set track name
            let _ = js_sys::Reflect::set(&current_track, &"name".into(), &track.name.into());
            
            // Set album and images
            let album = js_sys::Object::new();
            let images = js_sys::Array::new();
            for image in track.album.images {
                let img_obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&img_obj, &"url".into(), &image.url.into());
                images.push(&img_obj);
            }
            let _ = js_sys::Reflect::set(&album, &"images".into(), &images);
            let _ = js_sys::Reflect::set(&current_track, &"album".into(), &album);
            
            // Set artists
            let artists = js_sys::Array::new();
            for artist in track.artists {
                let artist_obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&artist_obj, &"name".into(), &artist.name.into());
                artists.push(&artist_obj);
            }
            let _ = js_sys::Reflect::set(&current_track, &"artists".into(), &artists);
            
            // Set track window
            let _ = js_sys::Reflect::set(&track_window, &"current_track".into(), &current_track);
            let _ = js_sys::Reflect::set(&state, &"track_window".into(), &track_window);
            
            // Set playback state
            let _ = js_sys::Reflect::set(&state, &"paused".into(), &(!playback.is_playing).into());
            let _ = js_sys::Reflect::set(&state, &"position".into(), &playback.progress_ms.into());
            let _ = js_sys::Reflect::set(&state, &"duration".into(), &track.duration_ms.into());
            
            // Update window state with track info
            let _ = js_sys::Reflect::set(&window, &"currentPlayerState".into(), &state);
            let _ = js_sys::Reflect::set(&window, &"currentPlaybackTime".into(), &(playback.progress_ms as f64).into());
            let _ = js_sys::Reflect::set(&window, &"totalDuration".into(), &(track.duration_ms as f64).into());
            let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &playback.is_playing.into());
        } else {
            // No track in response, set default state
            let _ = js_sys::Reflect::set(&window, &"currentPlayerState".into(), &JsValue::NULL);
            let _ = js_sys::Reflect::set(&window, &"currentPlaybackTime".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"totalDuration".into(), &0.0.into());
            let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &false.into());
        }
    }).await;
}
