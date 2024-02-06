// src/models/mod.rs

use serde::{ Deserialize, Serialize };

use self::data_change_fix::as_some_u32;

pub mod page;
// remove this when spotify fix their API response
pub mod data_change_fix;
pub mod recommendations;
pub mod artist;
pub mod album;
pub mod playlist;
pub mod track;
pub mod user;
pub mod auth;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpotifyImage {
    pub url: String,
    #[serde(deserialize_with = "as_some_u32")]
    pub height: Option<u32>,
    #[serde(deserialize_with = "as_some_u32")]
    pub width: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Followers {
    // pub href: Option<String>,
    pub total: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpotifyCopyright {
    pub text: String,
    pub r#type: String,
}
