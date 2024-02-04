// src/models/mod.rs

use std::error::Error;
use serde::{ Deserialize, Serialize };

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

pub mod recommendations;
pub mod artist;
pub mod album;
pub mod playlist;
pub mod track;

#[derive(Serialize)]
pub struct AuthRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpotifyImage {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub display_name: Option<String>,
    pub external_urls: ExternalUrls,
    pub r#type: String,
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
