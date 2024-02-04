// src/queries/playlist.rs

use crate::models::playlist::Playlist;

use super::get_spotify_data;

pub async fn get_playlist_data(playlist_id: &str) -> Result<Playlist, reqwest::Error> {
    let url = format!("https://api.spotify.com/v1/playlists/{playlist_id}"); // Replace with the actual API endpoint
    get_spotify_data(&url).await
}
