use wasm_bindgen::JsValue;
use web_sys::window;
use crate::api_request::models::StoredTracks;

const TRACKS_KEY: &str = "spotify_tracks";
const CACHE_DURATION: u64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

pub fn save_tracks(tracks: &Vec<(String, String, String, String)>, total: i32) -> Result<(), String> {
    if let Some(storage) = get_local_storage() {
        let stored_tracks = StoredTracks {
            tracks: tracks.clone(),
            total,
            timestamp: js_sys::Date::now() as u64,
        };

        let json = serde_json::to_string(&stored_tracks)
            .map_err(|e| format!("Failed to serialize tracks: {}", e))?;

        storage.set_item(TRACKS_KEY, &json)
            .map_err(|e| format!("Failed to save to localStorage: {:?}", e))?;

        Ok(())
    } else {
        Err("LocalStorage not available".to_string())
    }
}

pub fn load_tracks() -> Option<StoredTracks> {
    let storage = get_local_storage()?;
    let json = storage.get_item(TRACKS_KEY).ok()??;
    
    let stored_tracks: StoredTracks = serde_json::from_str(&json).ok()?;
    
    // Check if cache is still valid (within 24 hours)
    let now = js_sys::Date::now() as u64;
    if now - stored_tracks.timestamp <= CACHE_DURATION {
        Some(stored_tracks)
    } else {
        // Clear expired cache
        let _ = storage.remove_item(TRACKS_KEY);
        None
    }
}

fn get_local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}
