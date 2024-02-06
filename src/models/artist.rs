use serde::{ Deserialize, Serialize };

use super::{ data_change_fix::as_u32, ExternalUrls, Followers, SpotifyImage };

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
    #[serde(deserialize_with = "as_u32")]
    pub popularity: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artists {
    pub artists: Vec<Artist>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimplifiedArtist {
    pub id: String,
    pub name: String,
    pub external_urls: ExternalUrls,
    pub href: Option<String>,
}
