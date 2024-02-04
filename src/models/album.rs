use serde::{ Deserialize, Serialize };

use super::{
    track::SimplifiedTrack,
    artist::SimplifiedArtist,
    ExternalUrls,
    SpotifyCopyright,
    SpotifyImage,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: i32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<SpotifyImage>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub r#type: String,
    pub uri: String,
    pub artists: Vec<SimplifiedArtist>,
    pub tracks: AlbumTracks,
    pub copyrights: Vec<SpotifyCopyright>,
    pub genres: Vec<String>,
    pub label: String,
    pub popularity: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimplifiedAlbum {
    pub album_type: String,
    pub total_tracks: i32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<SpotifyImage>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub r#type: String,
    pub uri: String,
    pub artists: Vec<SimplifiedArtist>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlbumTracks {
    pub href: String,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
    pub items: Vec<SimplifiedTrack>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AlbumGroup {
    Album,
    Single,
    Compilation,
    AppearsOn,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlbumsResponse {
    pub albums: Vec<Album>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewAlbumsResponse {
    pub albums: NewAlbums,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewAlbums {
    pub href: String,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
    pub items: Vec<SimplifiedAlbum>,
}
