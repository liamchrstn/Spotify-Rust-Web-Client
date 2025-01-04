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
    pub tracks_window_size: (f32, f32),
    pub total_tracks: Option<i32>,
    pub is_loading: bool,  // Add loading state
    pub view_mode: ViewMode,
}

impl Default for AppState {
    fn default() -> Self {
        AppState { 
            username: None,
            saved_tracks: Vec::new(),
            show_tracks: false,
            tracks_window_open: false,
            tracks_window_size: (400.0, 600.0), // Increased window size
            total_tracks: None,
            is_loading: false,  // Initialize loading state
            view_mode: ViewMode::List,
        }
    }
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}
