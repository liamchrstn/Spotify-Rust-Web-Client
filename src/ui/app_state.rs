use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
}

pub struct AppState {
    pub username: Option<String>,
    pub saved_tracks: Vec<(String, String, String, String)>, // (track name, artist name, image url, uri)
    pub tracks_per_load: i32, // Number of tracks to load at a time
    pub loaded_tracks_count: i32, // Number of tracks currently loaded
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
    pub settings_window_locked: bool,
    pub settings_window_pos: (f32, f32), // Default position for Settings window
    pub liked_songs_window_pos: (f32, f32), // Default position for Liked Songs window
    pub music_player_window_pos: (f32, f32), // Default position for Music Player window
    pub collage_window_open: bool,
    pub collage_window_pos: (f32, f32), // Default position for Collage window
    pub loading_message: String, // Status message for loading operations
}

impl Default for AppState {
    fn default() -> Self {
        // Get localStorage instance
        let local_storage = web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten());

        // Load settings from localStorage with defaults
        let player_name = local_storage
            .as_ref()
            .and_then(|storage| storage.get_item("player_name").ok().flatten())
            .unwrap_or_else(|| "Web Playback SDK Quick Start Player".to_string());

        let tracks_per_load = local_storage
            .as_ref()
            .and_then(|storage| storage.get_item("tracks_per_load").ok().flatten())
            .and_then(|val| val.parse().ok())
            .unwrap_or(50);

        let settings_window_locked = local_storage
            .as_ref()
            .and_then(|storage| storage.get_item("settings_window_locked").ok().flatten())
            .and_then(|val| val.parse().ok())
            .unwrap_or(true);

        let view_mode = local_storage
            .as_ref()
            .and_then(|storage| storage.get_item("view_mode").ok().flatten())
            .map(|val| if val == "List" { ViewMode::List } else { ViewMode::Grid })
            .unwrap_or(ViewMode::Grid);

        AppState { 
            username: None,
            saved_tracks: Vec::new(),
            tracks_per_load,
            loaded_tracks_count: 0,
            show_tracks: false,
            tracks_window_open: false,
            player_window_open: false,
            tracks_window_size: (800.0, 600.0),
            total_tracks: None,
            is_loading: false,
            view_mode,
            search_text: String::new(),
            settings_window_open: false,
            player_name,
            settings_window_locked,
            settings_window_pos: (1490.0, 30.0),    // Hardcoded defaults
            liked_songs_window_pos: (238.0, 30.0),
            music_player_window_pos: (1069.0, 30.0),
            collage_window_open: false,
            collage_window_pos: (650.0, 30.0),
            loading_message: String::new(),
        }
    }
}

impl AppState {
    pub fn reset_areas(&mut self) {
        // Reset window-related states to default
        self.tracks_window_size = (800.0, 600.0); // Default size
        self.view_mode = ViewMode::Grid; // Default view mode
        self.search_text.clear(); // Clear search text
        self.settings_window_pos = (1490.0, 30.0);
        self.liked_songs_window_pos = (238.0, 30.0);
        self.music_player_window_pos = (1069.0, 30.0);
        self.collage_window_pos = (650.0, 30.0);
        // Add any additional reset logic as needed
    }
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}
