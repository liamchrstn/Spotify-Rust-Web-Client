use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GradientDirection {
    Diagonal,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StartingCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct AppState {
    pub collage_image: Option<Vec<u8>>, // Store the generated collage image data
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
    pub playlist_view_mode: ViewMode,  // Add this new field
    pub search_text: String,
    pub settings_window_open: bool,
    pub player_name: String,
    pub settings_window_locked: bool,
    pub settings_window_pos: (f32, f32), // Default position for Settings window
    pub liked_songs_window_pos: (f32, f32), // Default position for Liked Songs window
    pub music_player_window_pos: (f32, f32), // Default position for Music Player window
    pub collage_window_open: bool,
    pub collage_window_pos: (f32, f32), // Default position for Collage window
    pub loading_message: String, // Status message for loading operations
    pub progress: f32, // Progress for the progress bar
    pub collage_loading: bool, // Loading state for collage generation
    pub collage_width: u32,
    pub collage_height: u32,
    pub hue_shift: f32, // Add hue shift field
    pub gradient_direction: GradientDirection, // Add gradient direction field
    pub starting_corner: StartingCorner, // Add starting corner field
    pub playlists: Vec<(String, String, String, String, i32)>, // (playlist name, owner, image url, id, total tracks)
    pub show_playlists: bool,
    pub playlists_window_open: bool,
    pub playlists_window_size: (f32, f32),
    pub playlists_window_pos: (f32, f32),
    pub show_playlist_tracks_window: bool,
    pub playlist_tracks_window_open: bool,
    pub playlist_windows: Vec<(String, String, Vec<(String, String, String, String)>, ViewMode, bool, (f32, f32))>, // Add this new field
    pub user_id: Option<String>, // Add this new field
    pub settings_initialized: bool, // New field to track initialization
    pub original_name: String,      // New field to store the original player name
    pub sidebar_open: bool, // needed so 'sidebar_open' is recognized
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
            .unwrap_or_else(|| "Rustify Web Player".to_string());

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


        AppState { 
            collage_image: None,
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
            view_mode: ViewMode::List,
            playlist_view_mode: ViewMode::List,  // Add this field initialization
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
            progress: 0.0,
            collage_loading: false,
            collage_width: 1920,
            collage_height: 1080,
            hue_shift: 0.0, // Default hue shift value
            gradient_direction: GradientDirection::Diagonal, // Default gradient direction
            starting_corner: StartingCorner::TopLeft, // Default starting corner
            playlists: Vec::new(),
            show_playlists: false,
            playlists_window_open: false,
            playlists_window_size: (400.0, 500.0),
            playlists_window_pos: (300.0, 100.0),
            show_playlist_tracks_window: false,
            playlist_tracks_window_open: false,
            playlist_windows: Vec::new(), // Initialize the new field
            user_id: None, // Initialize the new field
            settings_initialized: false,                  // Initialize new fields
            original_name: String::new(),                 // Initialize new fields
            sidebar_open: true, // Initialize the new field
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

    pub fn constrain_to_central_panel(&self, ctx: &egui::Context) -> egui::Rect {
        let screen_rect = ctx.screen_rect();
        let sidebar_width = if self.sidebar_open { 180.0 } else { 0.0 }; // Only reserve space if sidebar is open
        let top_bar_height = 30.0;
        egui::Rect::from_min_max(
            egui::pos2(sidebar_width, screen_rect.min.y + top_bar_height),
            screen_rect.max,
        )
    }
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}

pub fn get_user_id_from_state() -> Option<String> {
    let state = APP_STATE.lock().unwrap();
    state.user_id.clone()
}

pub fn set_user_id(id: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.user_id = Some(id);
}
