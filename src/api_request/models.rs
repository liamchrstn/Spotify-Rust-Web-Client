use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub struct Album {
    pub images: Vec<Image>,
}

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SavedTracksResponse {
    pub items: Vec<SavedTrack>,
    pub total: i32,
}

#[derive(Deserialize)]
pub struct UserProfile {
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredTracks {
    pub tracks: Vec<(String, String, String)>, // (track name, artist name, image url)
    pub total: i32,
    pub timestamp: u64,
}
