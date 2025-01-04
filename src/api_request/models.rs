use serde::Deserialize;

#[derive(Deserialize)]
pub struct SavedTrack {
    pub track: Track,
}

#[derive(Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<Artist>,
}

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SavedTracksResponse {
    pub items: Vec<SavedTrack>,
    pub total: i32,
    pub offset: i32,
}

#[derive(Deserialize)]
pub struct UserProfile {
    pub display_name: Option<String>,
}