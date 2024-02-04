// src/queries/mod.rs

use std::env;

use regex::Regex;
use reqwest;
use serde::de::DeserializeOwned;

use self::auth::get_spotify_token;

pub mod auth;
pub mod track;
pub mod playlist;
// pub mod token_manager;
// pub mod client;

/// Fetches data from the Spotify API.
///
/// This asynchronous function handles sending a request to the Spotify API and deserializing
/// the response into the specified type. It uses the client credentials flow to authenticate.
///
/// # Arguments
/// * `url` - The full URL to which the request will be sent.
///
/// # Returns
/// A `Result` containing either the deserialized response object or an error if the request fails.
///
/// # Errors
/// Returns `reqwest::Error` if the request fails or if deserialization fails.
pub async fn get_spotify_data<T>(url: &str) -> Result<T, reqwest::Error> where T: DeserializeOwned {
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("Expected a client id");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("Expected a client secret");

    let token = get_spotify_token(&client_id, &client_secret).await?;
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send().await?
        .json::<T>().await?;

    Ok(res)
}

/// Resolves the final URL from a shortened Spotify URL.
///
/// This function performs an HTTP GET request to the shortened URL and returns the final URL after redirection.
///
/// # Arguments
/// * `short_url` - The shortened URL to resolve.
///
/// # Returns
/// A `Result` containing either the final URL as a `String` or an error if the request fails.
///
/// # Errors
/// Returns `reqwest::Error` if the HTTP request fails.
pub async fn get_final_spotify_url(short_url: &str) -> Result<String, reqwest::Error> {
    let resp = reqwest::get(short_url).await?;
    Ok(resp.url().to_string())
}

/// Extracts the Spotify ID and type (playlist or track) from a Spotify URL.
///
/// This function uses a regular expression to parse the URL and extract the resource type and ID.
///
/// # Arguments
/// * `url` - The Spotify URL to parse.
///
/// # Returns
/// An `Option` containing a tuple with the resource type (`String`) and the ID (`String`),
/// or `None` if the URL does not match the expected format.
///
/// # Examples
/// ```
/// let url = "https://open.spotify.com/track/12345";
/// let (kind, id) = extract_spotify_id_from_url(url).unwrap();
/// assert_eq!(kind, "track");
/// assert_eq!(id, "12345");
/// ```
pub fn extract_spotify_id_from_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"spotify\.com/(playlist|track)/([a-zA-Z0-9]+)").unwrap();
    re.captures(url).and_then(|caps| {
        let kind = caps.get(1)?.as_str().to_string();
        let id = caps.get(2)?.as_str().to_string();
        Some((kind, id))
    })
}
