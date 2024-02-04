// src/queries/track.rs

use crate::models::track::Track;

use super::get_spotify_data;

pub async fn get_track_data(id: &str) -> Result<Track, reqwest::Error> {
    let url = format!("https://api.spotify.com/v1/tracks/{id}"); // Replace with the actual API endpoint
    get_spotify_data(&url).await
}
