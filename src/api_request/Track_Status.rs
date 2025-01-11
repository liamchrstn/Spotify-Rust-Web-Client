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
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for playback".into());
            return;
        }
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
        web_sys::console::log_1(&"Playback transfer started".into());
    }).await;
}

async fn get_user_id() -> Option<String> {
    web_sys::console::log_1(&"Getting user ID...".into());
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for user profile".into());
            return None;
        }
    };
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        web_sys::console::log_1(&"Failed to get user profile".into());
        return None;
    }

    #[derive(serde::Deserialize)]
    struct User {
        id: String,
    }

    let user = match response.json::<User>().await {
        Ok(u) => u,
        Err(e) => {
            web_sys::console::log_2(&"Failed to parse user profile:".into(), &e.to_string().into());
            return None;
        }
    };
    
    web_sys::console::log_2(&"Found user ID:".into(), &user.id.clone().into());
    Some(user.id)
}

#[wasm_bindgen]
pub async fn start_playback(device_id: String) {
    web_sys::console::log_2(&"Starting playback for device:".into(), &device_id.clone().into());

    // Check device activation state
    let window = web_sys::window().expect("no global window exists");
    if let Ok(activated) = js_sys::Reflect::get(&window, &"deviceActivated".into()) {
        if !activated.as_bool().unwrap_or(false) {
            web_sys::console::log_1(&"Device not activated, activating first...".into());
            activate_device(device_id.clone()).await;
            return;
        }
    }

    // Get user ID for collection URI
    web_sys::console::log_1(&"Device ready, getting user collection...".into());
    let user_id = match get_user_id().await {
        Some(id) => id,
        None => {
            web_sys::console::log_1(&"Could not get user ID".into());
            return;
        }
    };

    // Get shuffle state
    let window = web_sys::window().expect("no global window exists");
    let shuffle = if let Ok(state) = js_sys::Reflect::get(&window, &"shuffleState".into()) {
        state.as_bool().unwrap_or(false)
    } else {
        false
    };
    web_sys::console::log_2(&"Current shuffle state:".into(), &shuffle.into());

    let context_uri = format!("spotify:user:{}:collection", user_id);
    web_sys::console::log_2(&"Using context URI:".into(), &context_uri.clone().into());

    // Set shuffle state before playing
    if shuffle {
        if let Some(token_clone) = get_token() {
            toggle_shuffle(token_clone).await;
        }
    }
    let token = if let Some(token) = get_token() {
        token
    } else {
        return;
    };
    let client = Client::new();
    let response = client
        .put("https://api.spotify.com/v1/me/player/play")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "device_id": device_id,
            "context_uri": context_uri
        }))
        .send()
        .await;

    handle_empty_response(response, || {
        web_sys::console::log_2(&"Starting playback with context:".into(), &context_uri.into());
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

    handle_empty_response(response, || {
        web_sys::console::log_1(&"Device activation started".into());
    }).await;

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
        web_sys::console::log_1(&"Device activation verified".into());
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"deviceActivated".into(), &true.into());
        
        // Start playback
        let device_id_clone = device_id.clone();
        spawn_local(async move {
            web_sys::console::log_1(&"Starting playback after device activation".into());
            start_playback(device_id_clone).await;
        });
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
pub async fn pause_playback() {
    web_sys::console::log_1(&"Pausing playback via API...".into());
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for pause".into());
            return;
        }
    };

    let client = Client::new();
    let response = client
        .put("https://api.spotify.com/v1/me/player/pause")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_empty_response(response, || {
        web_sys::console::log_1(&"Playback paused via API".into());
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &false.into());
    }).await;
}

#[wasm_bindgen]
pub async fn seek_playback(position_ms: i32) {
    web_sys::console::log_2(&"Seeking via API to position:".into(), &position_ms.into());
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for seek".into());
            return;
        }
    };

    let client = Client::new();
    let response = client
        .put(&format!("https://api.spotify.com/v1/me/player/seek?position_ms={}", position_ms))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_empty_response(response, || {
        web_sys::console::log_2(&"Seek completed via API to:".into(), &position_ms.into());
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"currentPlaybackTime".into(), &(position_ms as f64).into());
    }).await;
}

#[wasm_bindgen]
pub async fn play_track(uri: String) {
    web_sys::console::log_2(&"Playing track:".into(), &uri.clone().into());

    // Verify URI format
    if !uri.starts_with("spotify:track:") {
        web_sys::console::log_1(&"Invalid track URI format".into());
        return;
    }

    // Get token once at the start
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for playing track".into());
            return;
        }
    };

    let window = web_sys::window().expect("no global window exists");
    
    // Check and activate device if needed
    web_sys::console::log_1(&"Checking for active devices...".into());
    has_active_devices().await;

    // Wait a bit for device check to complete
    gloo_timers::future::TimeoutFuture::new(500).await;

    // Get first available device and activate it
    if let Ok(devices) = js_sys::Reflect::get(&window, &"availableDevices".into()) {
        if let Some(devices_array) = devices.dyn_ref::<js_sys::Array>() {
            web_sys::console::log_2(&"Found devices:".into(), &devices_array.length().into());
            if devices_array.length() > 0 {
                if let Ok(device) = js_sys::Reflect::get(&devices_array.get(0), &"id".into()) {
                    if let Some(device_id) = device.as_string() {
                        web_sys::console::log_2(&"Activating device:".into(), &device_id.clone().into());
                        activate_device(device_id).await;
                        
                        // Wait for device activation
                        gloo_timers::future::TimeoutFuture::new(1000).await;
                    }
                }
            } else {
                web_sys::console::log_1(&"No devices found".into());
                return;
            }
        }
    }

    // Create client and attempt to play track
    let client = Client::new();
    web_sys::console::log_1(&"Sending play request...".into());
    let response = client
        .put("https://api.spotify.com/v1/me/player/play")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "uris": [uri]
        }))
        .send()
        .await;

    handle_empty_response(response, || {
        web_sys::console::log_1(&"Track playback started".into());
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &true.into());
    }).await;
}

pub async fn resume_playback() {
    web_sys::console::log_1(&"Resuming playback via API...".into());
    let token = match get_token() {
        Some(token) => token,
        None => {
            web_sys::console::log_1(&"No token available for resume".into());
            return;
        }
    };

    let client = Client::new();
    let response = client
        .put("https://api.spotify.com/v1/me/player/play")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    handle_empty_response(response, || {
        web_sys::console::log_1(&"Playback resumed via API".into());
        let window = web_sys::window().expect("no global window exists");
        let _ = js_sys::Reflect::set(&window, &"isPlaying".into(), &true.into());
    }).await;
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
