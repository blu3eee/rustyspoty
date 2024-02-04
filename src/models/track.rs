use serde::{ Deserialize, Serialize };

use super::{ album::SimplifiedAlbum, artist::SimplifiedArtist, ExternalUrls };

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Track {
    pub album: SimplifiedAlbum,
    pub id: String,
    pub name: String,
    pub artists: Vec<SimplifiedArtist>,
    pub duration_ms: u64,
    pub preview_url: Option<String>,
    pub external_urls: ExternalUrls,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimplifiedTrack {
    pub artists: Vec<SimplifiedArtist>,
    pub disc_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    pub preview_url: Option<String>,
    pub track_number: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TracksResponse {
    pub tracks: Vec<Track>,
}
