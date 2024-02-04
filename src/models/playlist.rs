use serde::{ Deserialize, Serialize };

use super::{ ExternalUrls, Followers, SpotifyImage, User, track::Track };

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tracks: PlaylistTracks,
    pub owner: User,
    pub images: Vec<SpotifyImage>,
    pub followers: Followers,
    pub external_urls: ExternalUrls,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlaylistTracks {
    pub items: Vec<PlaylistTrackItem>,
    pub total: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlaylistTrackItem {
    pub track: Track,
}
