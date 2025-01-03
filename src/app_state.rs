use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct AppState {
    pub username: Option<String>,
    pub saved_tracks: Vec<(String, String)>, // (track name, artist name)
    pub show_tracks: bool,
    pub tracks_window_open: bool,
    pub tracks_window_size: (f32, f32),
    pub is_loading: bool,  // Add loading state
}

impl Default for AppState {
    fn default() -> Self {
        AppState { 
            username: None,
            saved_tracks: Vec::new(),
            show_tracks: false,
            tracks_window_open: false,
            tracks_window_size: (200.0, 400.0), // Changed from 300.0 to 200.0
            is_loading: false,  // Initialize loading state
        }
    }
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}