use serde::{ Deserialize, Serialize };

use super::{ album::SimplifiedAlbum, track::Track, ExternalUrls, Followers, SpotifyImage };

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub images: Vec<SpotifyImage>,
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub genres: Vec<String>,
    pub r#type: String,
    pub uri: String,
    pub popularity: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArtistsResponse {
    pub artists: Vec<Artist>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimplifiedArtist {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArtistAlbumsResponse {
    pub href: String,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
    pub items: Vec<SimplifiedAlbum>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArtistTopTracksResponse {
    pub tracks: Vec<Track>,
}
