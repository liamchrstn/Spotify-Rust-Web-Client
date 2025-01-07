# Rust Spotify Application Documentation

## main.rs

The main.rs file serves as the entry point for the WASM application, initializing the panic hook and starting the web application. Here's a detailed technical breakdown:

### Core Functionality
```rust
#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let web_options = eframe::WebOptions::default();
    WebRunner::new()
        .start(
            "canvas",
            web_options,
            Box::new(|_cc| Box::new(SpotifyApp::default())),
        )
        .await
}
```

#### Key Components:
1. **Panic Hook Setup**
   - Initializes console error panic hook for better error reporting in web context
   - Ensures panic messages are properly displayed in browser console

2. **Web Application Initialization**
   - Creates default web options for eframe
   - Initializes WebRunner with canvas element
   - Sets up default SpotifyApp instance

### Technical Notes
- Uses wasm_bindgen for JavaScript interop
- Implements async startup sequence
- Provides error handling through Result type
- Uses eframe for web-based GUI rendering

## lib.rs

The lib.rs file serves as the core library module, setting up the WASM bindings and initializing the application's main components. Here's a detailed technical breakdown:

### Module Organization
```rust
mod api_request;
mod ui;
mod utils;
mod storage;
mod mediaplayer;
```
- Organizes functionality into distinct modules
- Separates concerns for better maintainability
- Provides clear structure for application components

### External Interface
```rust
#[wasm_bindgen]
extern "C" {
    pub fn loginWithSpotify();
}
```
- Defines JavaScript function bindings
- Enables Spotify authentication integration
- Uses wasm_bindgen for FFI

### Application Initialization
```rust
#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    // Canvas element setup
    let canvas_element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    // WebRunner configuration
    let web_options = eframe::WebOptions::default();
    WebRunner::new()
        .start(
            canvas_element,
            web_options,
            Box::new(|cc| {
                // Font configuration
                let mut fonts = egui::FontDefinitions::default();
                egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
                cc.egui_ctx.set_fonts(fonts);
                
                // Image loader setup
                egui_extras::install_image_loaders(&cc.egui_ctx);
                
                Ok(Box::new(ui::SpotifyApp::default()))
            }),
        )
        .await
}
```

#### Key Components:
1. **Canvas Setup**
   - Retrieves canvas element from DOM
   - Ensures proper HTML element casting
   - Provides rendering target for application

2. **UI Configuration**
   - Sets up custom fonts with Phosphor icons
   - Installs image loaders for media support
   - Initializes SpotifyApp with configured context

### Technical Notes
- Uses web_sys for DOM interaction
- Implements async/await for initialization
- Provides comprehensive error handling
- Configures egui for UI rendering
- Sets up custom font and icon support

## storage.rs

The storage.rs file implements local storage functionality for caching track data. Here's a detailed technical breakdown:

### Constants
```rust
const TRACKS_KEY: &str = "spotify_tracks";
const CACHE_DURATION: u64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds
```
- Defines storage key for track data
- Sets cache duration to 24 hours
- Uses millisecond precision for timestamps

### Track Storage Implementation
```rust
pub fn save_tracks(tracks: &Vec<(String, String, String)>, total: i32) -> Result<(), String> {
    if let Some(storage) = get_local_storage() {
        let stored_tracks = StoredTracks {
            tracks: tracks.clone(),
            total,
            timestamp: js_sys::Date::now() as u64,
        };
        // ... serialization and storage
    }
}
```

#### Key Features:
1. **Data Structure**
   - Stores track information as tuples
   - Includes total count for pagination
   - Timestamps data for cache invalidation

2. **Error Handling**
   - Provides detailed error messages
   - Handles serialization failures
   - Manages localStorage unavailability

### Track Retrieval
```rust
pub fn load_tracks() -> Option<StoredTracks> {
    let storage = get_local_storage()?;
    let json = storage.get_item(TRACKS_KEY).ok()??;
    
    let stored_tracks: StoredTruct = serde_json::from_str(&json).ok()?;
    
    // Cache validation
    let now = js_sys::Date::now() as u64;
    if now - stored_tracks.timestamp <= CACHE_DURATION {
        Some(stored_tracks)
    } else {
        let _ = storage.remove_item(TRACKS_KEY);
        None
    }
}
```

#### Cache Management:
- Implements time-based cache invalidation
- Automatically cleans up expired data
- Provides seamless cache miss handling

### Technical Notes
- Uses serde for JSON serialization
- Implements Option-based error handling
- Provides automatic cache cleanup
- Uses web_sys for localStorage access
- Implements robust error recovery

## utils.rs

The utils.rs file provides utility functions for error logging and authentication management. Here's a detailed technical breakdown:

### Error Logging
```rust
pub fn log_error(message: &str) {
    console::error_1(&message.into());
}
```
- Provides console error logging
- Converts Rust strings to JavaScript
- Enables cross-platform error reporting

### Authentication Management
```rust
pub fn clear_token_and_redirect() {
    if let Some(window) = web_sys::window() {
        if let Ok(local_storage) = window.local_storage() {
            if let Some(storage) = local_storage {
                let _ = storage.remove_item("spotify_token");
            }
        }
        window.location().set_href("/").unwrap();
    }
}
```

#### Key Features:
1. **Token Cleanup**
   - Removes authentication token
   - Handles storage access failures
   - Provides clean logout functionality

2. **Navigation**
   - Redirects to home page
   - Ensures clean authentication state
   - Handles window access failures

### Technical Notes
- Uses web_sys for browser integration
- Implements graceful error handling
- Provides clean authentication reset
- Ensures proper resource cleanup

## api_request Module

The api_request module handles all Spotify API interactions and data models. Here's a detailed technical breakdown:

### Module Structure
```rust
pub mod models;
pub mod spotify_apis;
pub mod token;
pub mod imagerender;
```
- Organizes API functionality into focused submodules
- Separates data models from API calls
- Handles token management independently
- Provides image rendering capabilities

### Data Models (models.rs)
```rust
#[derive(Deserialize)]
pub struct SavedTrack {
    pub track: Track,
}

#[derive(Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Album,
}
```

#### Key Structures:
1. **Track Information**
   - Represents Spotify track data
   - Includes nested artist and album information
   - Uses serde for JSON deserialization

2. **Storage Models**
```rust
#[derive(Serialize, Deserialize)]
pub struct StoredTracks {
    pub tracks: Vec<(String, String, String)>, // (track name, artist name, image url)
    pub total: i32,
    pub timestamp: u64,
}
```
   - Optimized for local storage
   - Includes metadata for cache management
   - Stores essential track information

### API Implementation (spotify_apis.rs)

#### User Profile Fetching
```rust
pub async fn fetch_user_profile(token: String) {
    let client = Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
    // ... response handling
}
```
- Fetches user profile information
- Handles authentication errors
- Updates application state with user data

#### Saved Tracks Fetching
```rust
pub async fn fetch_saved_tracks(token: String) {
    // ... initialization
    loop {
        // Pagination handling
        let url = format!(
            "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
            limit, offset
        );
        // ... request and response handling
    }
}
```

#### Key Features:
1. **Pagination Support**
   - Handles large track collections
   - Implements rate limiting
   - Provides progress tracking

2. **Caching System**
   - Checks local storage first
   - Implements cache invalidation
   - Saves complete track list

3. **Error Handling**
   - Manages authentication failures
   - Handles network errors
   - Provides detailed error logging

4. **State Management**
   - Updates loading status
   - Manages track window visibility
   - Handles concurrent access

### Technical Notes
- Uses reqwest for HTTP requests
- Implements async/await patterns
- Provides comprehensive error handling
- Uses mutex for state management
- Implements efficient data caching
- Handles token expiration
- Manages API rate limits
- Optimizes image URL selection

### Token Management (token.rs)

The token.rs file implements secure token storage and management for Spotify authentication. Here's a detailed technical breakdown:

### Global State
```rust
pub static ACCESS_TOKEN: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
pub static SDK_STATUS: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
```
- Uses thread-safe global state
- Implements lazy initialization
- Provides mutex-protected access

### Token Management Functions

#### Access Token Setting
```rust
#[wasm_bindgen]
pub fn set_access_token(token: String) {
    let mut stored_token = ACCESS_TOKEN.lock().unwrap();
    *stored_token = Some(token.clone());
    spawn_local(async move {
        fetch_user_profile(token).await;
    });
}
```

#### Key Features:
1. **Thread Safety**
   - Uses mutex for concurrent access
   - Prevents race conditions
   - Ensures data consistency

2. **Profile Integration**
   - Automatically fetches user profile
   - Uses asynchronous execution
   - Maintains token synchronization

#### SDK Status Management
```rust
#[wasm_bindgen]
pub fn set_sdk_status(status: String) {
    let mut sdk_status = SDK_STATUS.lock().unwrap();
    *sdk_status = Some(status);
}
```
- Tracks SDK initialization state
- Provides status visibility
- Enables state-based UI updates

### Technical Notes
- Uses wasm_bindgen for JavaScript interop
- Implements thread-safe state management
- Provides async operation support
- Uses once_cell for lazy initialization
- Ensures proper resource cleanup
- Handles concurrent access safely

### Image Rendering (imagerender.rs)

The imagerender.rs file implements efficient image caching and rendering for the application. Here's a detailed technical breakdown:

### Image Cache Implementation
```rust
static IMAGE_CACHE: Lazy<Mutex<HashMap<String, Image<'static>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
```
- Uses global static cache
- Implements thread-safe access
- Provides lazy initialization
- Maps URLs to loaded images

### Image Loading Function
```rust
pub fn get_or_load_image(ctx: &egui::Context, url: &str) -> Option<Image<'static>> {
    let mut cache = IMAGE_CACHE.lock().unwrap();
    let url = url.to_string();
    
    if let Some(image) = cache.get(&url) {
        return Some(image.clone());
    }

    ctx.include_bytes(url.clone(), vec![]);
    let image = Image::from_uri(url.clone());
    cache.insert(url, image.clone());
    
    Some(image)
}
```

#### Key Features:
1. **Cache Management**
   - Checks cache before loading
   - Stores loaded images for reuse
   - Prevents redundant loading

2. **Asynchronous Loading**
   - Uses egui's async loading system
   - Provides empty bytes placeholder
   - Enables non-blocking operation

3. **Memory Efficiency**
   - Implements image caching
   - Prevents duplicate loading
   - Manages memory usage

### Technical Notes
- Uses egui for image handling
- Implements thread-safe caching
- Provides efficient resource reuse
- Uses HashMap for fast lookups
- Handles concurrent access safely
- Enables asynchronous loading
- Optimizes memory usage

## mediaplayer Module

The mediaplayer module implements playback control and UI components. Here's a detailed technical breakdown:

### Module Structure
```rust
pub mod scrubber;
pub mod mediaplayerwidget;
```
- Separates scrubber functionality
- Encapsulates media player widget
- Provides modular playback control

### Scrubber Implementation (scrubber.rs)

#### ScrubBar Component
```rust
pub struct ScrubBar {
    end_time: f64,
}
```

#### Key Features:
1. **Interactive Timeline**
```rust
pub fn add(&mut self, ui: &mut Ui, current_time: &mut f64, size: Vec2) {
    // Base line coordinates
    let start_y = scrub_painter.clip_rect().center().y;
    let start_x = scrub_painter.clip_rect().min.x + pointer_radius + circle_radius;
    let end_x = scrub_painter.clip_rect().max.x - pointer_radius - circle_radius;
    
    // ... drawing and interaction handling
}
```
- Implements draggable progress bar
- Provides visual feedback
- Handles hover interactions
- Shows time tooltips

2. **Playback Controls**
```rust
pub fn play_button(&self, ui: &mut Ui, playing: &mut bool, button_size: Vec2) {
    ui.horizontal(|ui| {
        if self.skip_button(ui, "⏮", button_size) {
            // Handle previous track
        }
        
        let response = ui.add_sized(
            button_size,
            egui::Button::new(if *playing { "⏸" } else { "▶" })
                .rounding(15.0)
        );
        // ... button handling
    });
}
```
- Provides play/pause toggle
- Implements skip controls
- Uses Unicode symbols
- Handles button states

### Time Management
```rust
pub struct TimeManager {
    pub current_time: f64,
    pub end_time: f64,
    pub playing: bool,
    start_timestamp: f64,
    last_update: f64,
}
```

#### Key Features:
1. **Time Tracking**
   - Manages playback position
   - Handles time updates
   - Provides looping functionality

2. **Performance Integration**
```rust
pub fn update(&mut self) {
    if self.playing {
        if let Some(performance) = window().and_then(|w| w.performance()) {
            let now = performance.now();
            let delta = now - self.last_update;
            // ... time update logic
        }
    }
}
```
- Uses browser performance API
- Implements precise timing
- Handles pause/resume states

### Visual Components
1. **Progress Bar**
   - Draws interactive timeline
   - Shows current position
   - Provides hover feedback
   - Displays time tooltips

2. **Time Display**
```rust
pub fn time_stamp_to_string(time: f64) -> String {
    let n = time as i32;
    let mins = n / (1000 * 60);
    let secs = (n / 1000) % 60;
    format!("{mins:02}:{secs:02}")
}
```
- Formats time display
- Shows minutes and seconds
- Uses zero-padding

### Technical Notes
- Uses egui for UI rendering
- Implements smooth animations
- Provides responsive design
- Handles user interactions
- Uses web_sys for timing
- Implements efficient drawing
- Manages state transitions

### Media Player Widget (mediaplayerwidget.rs)

The mediaplayerwidget.rs file implements the main media player interface. Here's a detailed technical breakdown:

### Window Implementation
```rust
pub fn show_mediaplayer_window(ctx: &egui::Context) {
    let mut time_manager = TimeManager::new(100_000.0, 1.0);
    let mut state = APP_STATE.lock().unwrap();

    egui::Window::new("Music Player")
        .resizable(true)
        .open(&mut state.player_window_open)
        .collapsible(true)
        .show(ctx, |ui| {
            // ... window content
        });
}
```

#### Key Features:
1. **Layout Management**
```rust
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    let padding = 20.0;
    let square_size = egui::vec2(100.0, 100.0);
    let total_size = ui.available_size();
    ui.set_min_size(total_size);
    // ... component layout
});
```
- Implements centered layout
- Manages component spacing
- Provides responsive sizing
- Handles window resizing

2. **Album Art Display**
```rust
let rect = egui::Rect::from_min_size(
    ui.min_rect().min + egui::vec2((ui.available_width() - square_size.x) * 0.5, padding),
    square_size
);
ui.painter().rect_filled(rect, 10.0, egui::Color32::BLUE);
```
- Displays album artwork
- Centers content automatically
- Implements rounded corners
- Maintains aspect ratio

3. **Playback Controls**
```rust
let mut button = ui.add(egui::Button::new(if is_playing {
    egui::RichText::new("⏸").size(button_size.x)
} else {
    egui::RichText::new("▶").size(button_size.x)
}));
```
- Provides play/pause toggle
- Shows player state
- Handles button interactions
- Implements hover effects

### JavaScript Integration
```rust
// Get current player state
let player_state = js_sys::eval("window.player && window.player.getCurrentState()")
    .ok()
    .and_then(|val| val.as_bool());

// Toggle playback
if button.clicked() {
    let result = js_sys::eval("window.playPause && window.playPause()");
    // ... error handling
}
```

#### Key Features:
1. **State Management**
   - Syncs with JavaScript player
   - Handles state transitions
   - Provides error recovery

2. **Track Information**
```rust
let track_info = js_sys::eval("window.currentPlayerState")
    .ok()
    .and_then(|val| {
        // ... track info extraction
    });
```
- Extracts track metadata
- Handles missing data
- Updates display dynamically

### UI Components
1. **Track Display**
```rust
if let Some((title, artist)) = track_info {
    ui.label(egui::RichText::new(title).heading());
    ui.label(egui::RichText::new(artist).small());
} else {
    ui.label(egui::RichText::new("No track playing").heading());
    ui.label(egui::RichText::new("Select a track to play").small());
}
```
- Shows track title
- Displays artist name
- Provides fallback states
- Uses styled text

2. **Scrubber Integration**
```rust
let mut scrub_bar = ScrubBar::new(time_manager.end_time);
scrub_bar.add(ui, &mut time_manager.current_time, egui::vec2(square_size.x, scrubber_height));
```
- Implements progress bar
- Manages playback position
- Provides seek functionality

### Technical Notes
- Uses egui for UI rendering
- Implements JavaScript interop
- Provides state synchronization
- Handles window management
- Uses responsive design
- Implements error handling
- Manages continuous repainting

## ui Module

The ui module implements the application's user interface and state management. Here's a detailed technical breakdown:

### Module Structure
```rust
pub mod app_state;
mod savedtracks;
mod ui;
```
- Manages application state
- Handles saved tracks display
- Implements main UI components

### Application State (app_state.rs)

#### State Structure
```rust
pub struct AppState {
    pub username: Option<String>,
    pub saved_tracks: Vec<(String, String, String)>, // (track name, artist name, image url)
    pub show_tracks: bool,
    pub tracks_window_open: bool,
    pub player_window_open: bool, 
    pub tracks_window_size: (f32, f32),
    pub total_tracks: Option<i32>,
    pub is_loading: bool,
    pub view_mode: ViewMode,
    pub search_text: String,
}
```

#### Key Features:
1. **View Mode Management**
```rust
#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
}
```
- Supports multiple view types
- Enables layout switching
- Maintains view preferences

2. **Global State Access**
```rust
pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));
```
- Provides thread-safe state
- Uses lazy initialization
- Implements default values

3. **Window Management**
   - Tracks window states
   - Manages window sizes
   - Controls visibility

4. **Track Information**
   - Stores track metadata
   - Manages loading states
   - Handles search functionality

### State Management Functions

#### Username Setting
```rust
pub fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}
```
- Updates user information
- Provides thread-safe access
- Handles state mutations

### Default Implementation
```rust
impl Default for AppState {
    fn default() -> Self {
        AppState { 
            username: None,
            saved_tracks: Vec::new(),
            show_tracks: false,
            tracks_window_open: false,
            player_window_open: false,
            tracks_window_size: (800.0, 600.0),
            total_tracks: None,
            is_loading: false,
            view_mode: ViewMode::Grid,
            search_text: String::new(),
        }
    }
}
```

#### Key Features:
1. **Initial State**
   - Sets default window sizes
   - Initializes empty collections
   - Configures default view mode

2. **Window Configuration**
   - Sets default dimensions
   - Initializes visibility flags
   - Configures window states

### Technical Notes
- Uses once_cell for lazy loading
- Implements mutex for thread safety
- Provides type-safe enums
- Manages window states
- Handles loading indicators
- Supports search functionality
- Implements view mode switching

### Saved Tracks Display (savedtracks.rs)

The savedtracks.rs file implements the saved tracks display window with list and grid views. Here's a detailed technical breakdown:

### Window Implementation
```rust
pub fn show_saved_tracks_window(ctx: &Context) {
    let mut state = APP_STATE.lock().unwrap();
    
    egui::Window::new("Liked Songs")
        .open(&mut tracks_window_open)
        .default_size(window_size)
        .min_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            // ... window content
        });
}
```

#### Key Features:
1. **Search Functionality**
```rust
ui.horizontal(|ui| {
    ui.label(format!("{} Search:", egui_phosphor::bold::MAGNIFYING_GLASS));
    let desired_width = (state.search_text.len() as f32 * 8.0).max(100.0);
    let search_response = ui.add(
        egui::TextEdit::singleline(&mut state.search_text)
            .desired_width(desired_width)
    );
});
```
- Implements real-time search
- Provides dynamic width
- Uses icon integration
- Filters track list

2. **View Mode Toggle**
```rust
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    if ui.toggle_value(&mut (view_mode == ViewMode::List), &format!("{} List", egui_phosphor::bold::LIST)).clicked() {
        state.view_mode = ViewMode::List;
        window_size = (400.0, 600.0);
    }
    // ... grid view toggle
});
```
- Supports list/grid views
- Adjusts window size
- Uses icon indicators
- Maintains state

### List View Implementation
```rust
fn show_list_view(ui: &mut Ui, tracks: &[&(String, String, String)]) {
    for (track, artists, image_url) in tracks {
        ui.horizontal(|ui| {
            // Album art
            if let Some(image) = get_or_load_image(ui.ctx(), image_url) {
                ui.add(image.fit_to_exact_size([40.0, 40.0].into()));
            }
            
            // Track info
            ui.vertical(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(track)
                        .size(16.0)
                        .strong()
                ));
                // ... artist label
            });
        });
    }
}
```

#### Key Features:
1. **Track Display**
   - Shows album artwork
   - Displays track title
   - Shows artist name
   - Uses consistent spacing

2. **Visual Styling**
   - Implements text formatting
   - Uses color theming
   - Provides visual hierarchy
   - Adds separators

### Grid View Implementation
```rust
fn show_grid_view(ui: &mut Ui, tracks: &[&(String, String, String)]) {
    let available_width = ui.available_width();
    let column_width = (available_width / 3.0).max(100.0) - 10.0;
    
    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .column(Column::exact(column_width))
        // ... table configuration
        .body(|mut body| {
            // ... grid layout
        });
}
```

#### Key Features:
1. **Layout Management**
   - Uses table builder
   - Implements responsive columns
   - Handles vertical lines
   - Manages spacing

2. **Track Display**
   - Shows larger artwork
   - Implements grid layout
   - Provides visual separation
   - Handles empty cells

### Loading State
```rust
if is_loading {
    ui.horizontal(|ui| {
        ui.spinner();
        if let Some(total) = total_tracks {
            ui.label(format!(
                "Loading tracks... ({} of {} loaded)", 
                tracks.len(), 
                total
            ));
        }
    });
}
```
- Shows loading spinner
- Displays progress
- Handles total count
- Provides feedback

### Technical Notes
- Uses egui for UI rendering
- Implements responsive design
- Provides search filtering
- Handles image loading
- Uses consistent styling
- Manages window state
- Implements view switching

### Main UI Implementation (ui.rs)

The ui.rs file implements the core application interface and functionality. Here's a detailed technical breakdown:

### Application Structure
```rust
#[derive(Default)]
pub struct SpotifyApp {
    pub show_player: bool,
    pub sdk_status: String,
}
```
- Manages player visibility
- Tracks SDK status
- Provides default implementation

### UI Implementation
```rust
impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ... UI implementation
    }
}
```

#### Key Features:
1. **Top Panel**
```rust
egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
    ui.horizontal(|ui| {
        if let Some(name) = &state.username {
            ui.heading(format!("Welcome, {}", name));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                global_theme_switch(ui);
                // ... logout button
            });
        }
    });
});
```
- Shows user welcome
- Provides theme switching
- Implements logout
- Uses horizontal layout

2. **Central Panel**
```rust
egui::CentralPanel::default().show(ctx, |ui| {
    if let Some(name) = &state.username {
        if ui.button("View Your Liked Songs").clicked() {
            state.show_tracks = true;
            state.tracks_window_open = true;
            // ... fetch tracks
        }
        
        if ui.button("Show Player").clicked() {
            self.show_player = true;
            state.player_window_open = true;
        }
    } else {
        // ... login button
    }
});
```

#### Key Features:
1. **Authentication Flow**
   - Shows login button
   - Handles authentication
   - Manages user state
   - Provides logout functionality

2. **Feature Access**
   - Controls track viewing
   - Manages player visibility
   - Shows SDK status
   - Handles state updates

### State Management
```rust
{
    let mut state = APP_STATE.lock().unwrap();
    // ... UI implementation using state
}
```

#### Key Features:
1. **Thread Safety**
   - Uses mutex locking
   - Provides scoped access
   - Ensures state consistency
   - Handles concurrent updates

2. **Window Management**
   - Controls window visibility
   - Manages window states
   - Handles component updates
   - Provides loading states

### Loading Management
```rust
let is_loading = {
    let state = APP_STATE.lock().unwrap();
    state.is_loading
};

if is_loading {
    ctx.request_repaint();
}
```
- Tracks loading state
- Requests repaints
- Provides visual feedback
- Manages UI updates

### Technical Notes
- Uses egui for UI rendering
- Implements theme switching
- Provides responsive layout
- Handles authentication flow
- Uses async operations
- Manages window states
- Implements component visibility
