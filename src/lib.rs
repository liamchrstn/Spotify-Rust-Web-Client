use eframe::wasm_bindgen::{self, prelude::*};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use eframe::WebRunner;
use web_sys::window;
use console_error_panic_hook;
use std::panic;

static ACCESS_TOKEN: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
pub fn set_access_token(token: String) {
    let mut stored_token = ACCESS_TOKEN.lock().unwrap();
    *stored_token = Some(token.clone());
    spawn_local(async move {
        fetch_user_profile(token).await;
    });
}

#[wasm_bindgen]
pub async fn fetch_saved_songs() {
    if let Some(window) = web_sys::window() {
        if let Ok(local_storage) = window.local_storage() {
            if let Some(storage) = local_storage {
                if let Ok(Some(token)) = storage.get_item("spotify_token") {
                    let token_clone = token.clone();
                    spawn_local(async move {
                        fetch_saved_tracks(token_clone).await;
                    });
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct SavedTrack {
    track: Track,
}

#[derive(Deserialize)]
struct Track {
    name: String,
    artists: Vec<Artist>,
}

#[derive(Deserialize)]
struct Artist {
    name: String,
}

#[derive(Deserialize)]
struct SavedTracksResponse {
    items: Vec<SavedTrack>,
}

struct AppState {
    username: Option<String>,
    saved_tracks: Vec<(String, String)>, // (track name, artist name)
    show_tracks: bool,
    tracks_window_open: bool,
    tracks_window_size: (f32, f32),
    is_loading: bool,  // Add loading state
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

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

#[wasm_bindgen]
extern "C" {
    fn loginWithSpotify();
}

fn set_username(name: String) {
    let mut state = APP_STATE.lock().unwrap();
    state.username = Some(name);
}

#[derive(Deserialize)]
struct UserProfile {
    display_name: Option<String>,
}

async fn fetch_user_profile(token: String) {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status() == 401 {
                clear_token_and_redirect();
            } else if response.status().is_success() {
                match response.json::<UserProfile>().await {
                    Ok(user) => {
                        if let Some(name) = user.display_name {
                            set_username(name);
                        }
                    }
                    Err(err) => {
                        log_error(&format!("Failed to parse user profile: {:?}", err));
                    }
                }
            } else {
                log_error(&format!("Failed to fetch user profile: {:?}", response.status()));
            }
        }
        Err(err) => {
            log_error(&format!("Request error: {:?}", err));
        }
    }
}

async fn fetch_saved_tracks(token: String) {
    {
        let mut state = APP_STATE.lock().unwrap();
        state.is_loading = true;
    }

    let client = reqwest::Client::new();
    
    // Only fetch first 50 tracks
    let url = "https://api.spotify.com/v1/me/tracks?limit=50&offset=0";
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            match response.status().as_u16() {
                200 => {
                    match response.json::<SavedTracksResponse>().await {
                        Ok(tracks) => {
                            let track_info: Vec<(String, String)> = tracks.items
                                .into_iter()
                                .map(|item| {
                                    let artists = item.track.artists
                                        .iter()
                                        .map(|artist| artist.name.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    (item.track.name, artists)
                                })
                                .collect();
                            
                            let mut state = APP_STATE.lock().unwrap();
                            state.saved_tracks = track_info;
                            state.show_tracks = true;
                            state.is_loading = false;  // Set loading to false when done
                        }
                        Err(err) => {
                            let mut state = APP_STATE.lock().unwrap();
                            state.is_loading = false;  // Set loading to false on error
                            log_error(&format!("Failed to parse saved tracks: {:?}", err));
                        }
                    }
                }
                401 => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.is_loading = false;  // Set loading to false on error
                    clear_token_and_redirect();
                }
                status => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.is_loading = false;  // Set loading to false on error
                    log_error(&format!("Failed to fetch saved tracks: {}", status));
                }
            }
        }
        Err(err) => {
            let mut state = APP_STATE.lock().unwrap();
            state.is_loading = false;  // Set loading to false on error
            log_error(&format!("Request error: {:?}", err));
        }
    }
}

fn clear_token_and_redirect() {
    if let Some(window) = web_sys::window() {
        if let Ok(local_storage) = window.local_storage() {
            if let Some(storage) = local_storage {
                let _ = storage.remove_item("spotify_token");
            }
        }
        window.location().set_href("/").unwrap();
    }
}

fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

#[derive(Default)]
struct SpotifyApp;

impl eframe::App for SpotifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = APP_STATE.lock().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(name) = &state.username {
                ui.heading(format!("Welcome, {}", name));
                
                if ui.button("Show Saved Tracks").clicked() {
                    state.show_tracks = true;
                    state.tracks_window_open = true;
                    spawn_local(async {
                        fetch_saved_songs().await;
                    });
                }

                if state.show_tracks {
                    // Clone the data we need before the window closure
                    let tracks = state.saved_tracks.clone();
                    let window_size = state.tracks_window_size;
                    let is_loading = state.is_loading;
                    
                    egui::Window::new("Saved Tracks")
                        .open(&mut state.tracks_window_open)
                        .default_size(window_size)
                        .resizable(true)
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                if is_loading {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Loading tracks...");
                                    });
                                } else {
                                    for (track, artists) in &tracks {
                                        ui.horizontal(|ui| {
                                            let text = format!("{} - {}", track, artists);
                                            let available_width = ui.available_width();
                                            let text_width = ui.fonts(|fonts| fonts.layout_no_wrap(text.clone(), egui::FontId::default(), egui::Color32::WHITE).size().x);
                                            let truncated_text = if text_width > available_width {
                                                format!("{}...", &text[..(available_width as usize / 10)]) // Approximation
                                            } else {
                                                text
                                            };
                                            ui.add(egui::Label::new(truncated_text).wrap(false));
                                        });
                                        ui.separator();
                                    }
                                }
                            });
                        });
                }

                if ui.button("Logout").clicked() {
                    if let Some(window) = window() {
                        if let Ok(local_storage) = window.local_storage() {
                            if let Some(storage) = local_storage {
                                let _ = storage.remove_item("spotify_token");
                            }
                        }
                    }
                    state.username = None;
                    state.saved_tracks.clear();
                    state.show_tracks = false;
                }
            } else {
                if ui.button("Login with Spotify").clicked() {
                    loginWithSpotify();
                }
            }
        });
    }
}

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