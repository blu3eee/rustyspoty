use serde::{ Deserialize, Serialize };

use super::{
    artist::SimplifiedArtist,
    data_change_fix::as_u32,
    page::Page,
    track::SimplifiedTrack,
    ExternalUrls,
    SpotifyCopyright,
    SpotifyImage,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: i32,
    pub available_markets: Option<Vec<String>>,
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
    pub tracks: Page<SimplifiedTrack>,
    pub copyrights: Vec<SpotifyCopyright>,
    pub genres: Vec<String>,
    #[serde(deserialize_with = "as_u32")]
    pub popularity: u32,
    pub label: Option<String>,
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
pub struct Albums {
    pub albums: Vec<Album>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewAlbums {
    pub albums: Page<SimplifiedAlbum>,
}
