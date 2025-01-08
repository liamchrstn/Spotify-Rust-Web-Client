use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
}

pub struct AppState {
    pub username: Option<String>,
    pub saved_tracks: Vec<(String, String, String)>, // (track name, artist name, image url)
    pub show_tracks: bool,
    pub tracks_window_open: bool,
    pub player_window_open: bool, 
    pub tracks_window_size: (f32, f32),
    pub total_tracks: Option<i32>,
    pub is_loading: bool,  // Add loading state
    pub view_mode: ViewMode,
    pub search_text: String,
    pub settings_window_open: bool,
    pub player_name: String,  // Add this field
}

impl Default for AppState {
    fn default() -> Self {
        // Try to get the stored player name from localStorage
        let player_name = web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item("player_name").ok().flatten())
            .unwrap_or_else(|| "Web Playback SDK Quick Start Player".to_string());

        AppState { 
            username: None,
            saved_tracks: Vec::new(),
            show_tracks: false,
            tracks_window_open: false,
            player_window_open: false,  // Added missing field
            tracks_window_size: (800.0, 600.0), // Increased window size
            total_tracks: None,
            is_loading: false,  // Initialize loading state
            view_mode: ViewMode::Grid,  // Changed from List to Grid
            search_text: String::new(),
            settings_window_open: false,
            player_name,
        }
    }
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}
