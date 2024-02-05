use std::error::Error;

use reqwest::Client as ReqwestClient;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    token_manager::SpotifyTokenManager,
    RustyError,
    SeedValidationError,
    models::{ playlist::*, recommendations::*, track::*, artist::*, album::* },
};

/// A client for interacting with the Spotify Web API.
///
/// This client encapsulates the process of making requests to the Spotify API,
/// including handling authentication through the `SpotifyTokenManager`.
pub struct SpotifyClient {
    /// Manages the Spotify API authentication tokens.
    token_manager: SpotifyTokenManager,
    /// A `reqwest::Client` for making HTTP requests.
    http_client: ReqwestClient,
}

// Define the base URL for the Spotify API as a constant
const SPOTIFY_API_BASE_URL: &str = "https://api.spotify.com/v1";

impl SpotifyClient {
    /// Creates a new instance of `SpotifyClient`.
    ///
    /// Initializes the client with client ID and secret for authentication.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The Spotify client ID.
    /// * `client_secret` - The Spotify client secret.
    pub fn new(client_id: String, client_secret: String) -> Self {
        let token_manager: SpotifyTokenManager = SpotifyTokenManager::new(client_id, client_secret);
        let http_client: ReqwestClient = ReqwestClient::new();
        SpotifyClient {
            token_manager,
            http_client,
        }
    }

    /// Performs a GET request to the specified Spotify API endpoint.
    ///
    /// This method automatically handles authorization with the Spotify API
    /// and deserializes the response into the specified type.
    ///
    /// # Arguments
    ///
    /// * `path` - The specific endpoint path after the base URL.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the deserialized response data or an error.
    async fn get_spotify_data<T>(&mut self, path: &str) -> Result<T, RustyError>
        where T: DeserializeOwned
    {
        let token: String = self.token_manager.get_valid_token().await?;
        let url: String = format!("{SPOTIFY_API_BASE_URL}{path}");
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?
            .json::<T>().await?;

        Ok(response)
    }

    /// Fetches detailed information about a specific album by its Spotify ID.
    ///
    /// # Arguments
    /// * `album_id` - The Spotify ID of the album.
    ///
    /// # Returns
    /// * `Result<Album, RustyError>`: On success, returns an `Album` object containing detailed information about the album. On failure, returns a `RustyError` detailing the issue.
    ///
    /// # Errors
    /// * Returns an error for invalid album ID, network issues, or problems with the Spotify API.
    ///
    /// # Example
    /// ```no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let album_id = "1DFixLWuPkv3KT3TnV35m3";
    /// let album = spotify_client.get_album(album_id).await?;
    /// println!("Album name: {}", album.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_album(&mut self, album_id: &str) -> Result<Album, RustyError> {
        let path = format!("/albums/{album_id}");
        self.get_spotify_data(&path).await
    }

    /// Fetches detailed information for several albums based on their Spotify IDs.
    ///
    /// Useful for retrieving data about multiple albums in one request. This method optimizes data fetching by minimizing the number of API calls needed to retrieve album information.
    ///
    /// # Arguments
    /// * `album_ids`: A slice of Spotify album IDs. Each ID must correspond to an album on Spotify.
    ///
    /// # Returns
    /// * `Result<AlbumsResponse, Box<dyn Error>>`: On success, returns an `AlbumsResponse` containing detailed information about each requested album. On error, returns a boxed error detailing the failure, such as exceeding the maximum number of IDs allowed.
    ///
    /// # Errors
    /// * Returns an error if the provided list of album IDs exceeds 20, as this is the Spotify API's limit for this type of request.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let album_ids = ["1o2NpYGqHiCq7FoiYdyd1x", "4tZwfgrHOc3mvqYlEYSvVi"];
    /// let result = client.get_several_albums(&album_ids).await;
    /// if let Ok(albums_response) = result {
    ///     for album in albums_response.albums {
    ///         println!("Album: {}", album.name);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_several_albums(
        &mut self,
        album_ids: &[String]
    ) -> Result<AlbumsResponse, Box<dyn Error>> {
        if album_ids.len() > 20 {
            return Err(
                Box::new(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Maximum of 20 IDs.")
                )
            );
        }

        let ids_param = album_ids.join(",");
        let path = format!("/albums?ids={}", ids_param);

        Ok(self.get_spotify_data(&path).await.map_err(|e| Box::new(e) as Box<dyn Error>)?)
    }

    /// Retrieves the tracks contained in a specific album on Spotify.
    ///
    /// This function is ideal for applications that need to display track listings for albums, such as music library managers or playlist creators.
    ///
    /// # Arguments
    /// * `album_id`: The unique identifier for the album on Spotify.
    ///
    /// # Returns
    /// * `Result<AlbumTracks, RustyError>`: On success, returns an `AlbumTracks` object containing a list of tracks in the specified album. On failure, returns a `RustyError` detailing the issue encountered during the API call.
    ///
    /// # Errors
    /// * An error will be returned if the album ID is invalid, if there's a problem with the network request, or if the API responds with an error.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let album_id = "4aawyAB9vmqN3uQ7FjRGTy";
    /// let result = client.get_album_tracks(album_id).await;
    /// if let Ok(album_tracks) = result {
    ///     for track in album_tracks.items {
    ///         println!("Track: {}", track.name);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_album_tracks(&mut self, album_id: &str) -> Result<AlbumTracks, RustyError> {
        let path = format!("/albums/{album_id}/tracks");
        self.get_spotify_data(&path).await
    }

    /// Fetches detailed information about a specific album from Spotify.
    ///
    /// This function retrieves all available data for a given album, identified by its unique Spotify ID. This includes tracks, artists, release date, and more, which can be useful for applications that require detailed album metadata.
    ///
    /// # Arguments
    /// * `album_id`: A `&str` representing the Spotify ID of the album to retrieve.
    ///
    /// # Returns
    /// * `Result<Album, RustyError>`: On success, returns an `Album` object containing detailed information about the specified album. On failure, returns a `RustyError` indicating what went wrong during the request.
    ///
    /// # Errors
    /// * An error is returned if the specified `album_id` does not exist or if there is a network issue that prevents the API call from completing successfully.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let album_id = "3ThQkHrQ6FSq8VIBv3WIEs";
    /// let result = client.get_album(album_id).await;
    /// match result {
    ///     Ok(album) => println!("Album found: {}", album.name),
    ///     Err(e) => eprintln!("An error occurred: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This method provides an efficient way to access detailed album information, including links to high-quality cover art, making it essential for music-related applications and servicess.
    pub async fn get_new_album_releases(
        &mut self,
        limit: Option<i32>,
        offset: Option<i32>
    ) -> Result<NewAlbumsResponse, RustyError> {
        let limit = limit.unwrap_or(20).min(50).max(1); // Ensures limit is within 1-50
        let offset = offset.unwrap_or(0).max(0); // Ensures offset is non-negative

        let query_params = format!("?limit={}&offset={}", limit, offset);
        let path = format!("/browse/new-releases{}", query_params);

        self.get_spotify_data::<NewAlbumsResponse>(&path).await
    }

    /// Fetches detailed information about a specific artist from the Spotify API.
    ///
    /// # Arguments
    /// * `artist_id` - A `&str` slice that holds the Spotify ID of the artist.
    ///
    /// # Returns
    /// `Result<Artist, RustyError>`
    /// - On success, returns the artist's detailed information wrapped in an `Artist`.
    /// - On failure, returns an error.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let artist = client.get_artist("artist_id").await?;
    /// println!("Artist Name: {}", artist.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_artist(&mut self, artist_id: &str) -> Result<Artist, RustyError> {
        let path = format!("/artists/{artist_id}");
        self.get_spotify_data(&path).await
    }

    /// Retrieves information for multiple artists based on their Spotify IDs.
    ///
    /// # Arguments
    /// * `artist_ids` - A slice of Spotify IDs for the artists. Maximum of 50 IDs allowed.
    ///
    /// # Returns
    /// * `Result<ArtistsResponse, Box<dyn Error>>`: On success, returns an `ArtistsResponse` containing a list of artists. On failure, returns an error detailing why the request failed.
    ///
    /// # Errors
    /// * Returns an error if no artist IDs are provided or if the number of IDs exceeds the limit of 50.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let artist_ids = vec!["artist_id1".to_string(), "artist_id2".to_string()];
    /// let artists = client.get_several_artists(&artist_ids).await?;
    /// for artist in artists.artists {
    ///     println!("Artist Name: {}", artist.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_several_artists(
        &mut self,
        artist_ids: &[String]
    ) -> Result<ArtistsResponse, Box<dyn Error>> {
        if artist_ids.len() == 0 {
            return Err(
                Box::new(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Please provide at least 1 artist id."
                    )
                )
            );
        }
        if artist_ids.len() > 50 {
            return Err(
                Box::new(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Maximum of 50 IDs.")
                )
            );
        }

        let ids_param = artist_ids.join(",");
        let path = format!("/artists?ids={ids_param}");

        Ok(self.get_spotify_data(&path).await.map_err(|e| Box::new(e) as Box<dyn Error>)?)
    }

    /// Retrieves the albums associated with a specific artist from the Spotify catalog.
    ///
    /// # Arguments
    /// * `artist_id` - The Spotify ID of the artist whose albums are being retrieved.
    ///
    /// # Returns
    /// * `Result<ArtistAlbumsResponse, Box<dyn Error>>`: On success, returns an `ArtistAlbumsResponse` containing the artist's albums. On failure, returns a boxed error detailing the failure reason.
    ///
    /// # Errors
    /// * Returns an error for invalid artist ID, network issues, or Spotify API errors.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # use spotify_client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClient::new("your_client_id".to_string(), "your_client_secret".to_string());
    /// let artist_id = "4tZwfgrHOc3mvqYlEYSvVi"; // Example artist ID for Daft Punk
    /// match spotify_client.get_artist_albums(artist_id).await {
    ///     Ok(response) => {
    ///         for album in response.items {
    ///             println!("Album: {} - Release Date: {}", album.name, album.release_date);
    ///         }
    ///     },
    ///     Err(e) => println!("Error occurred: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_artist_albums(
        &mut self,
        artist_id: &str
    ) -> Result<ArtistAlbumsResponse, Box<dyn Error>> {
        let path = format!("/artists/{artist_id}/albums");
        Ok(self.get_spotify_data(&path).await.map_err(|e| Box::new(e) as Box<dyn Error>)?)
    }

    /// Fetches an artist's top tracks from the Spotify catalog, optionally filtered by a specific market.
    ///
    /// # Arguments
    ///
    /// * `artist_id` - A `&str` slice representing the Spotify ID of the artist.
    /// * `market` - An optional `&str` slice representing an ISO 3166-1 alpha-2 country code to filter tracks available in a specific market.
    ///
    /// # Returns
    ///
    /// * `Result<ArtistTopTracksResponse, RustyError>`: On success, this function returns an `ArtistTopTracksResponse` containing the artist's top tracks. On failure, it returns a `RustyError` detailing the issue encountered.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let artist_id = "0TnOYISbd1XYRBk9myaseg";
    /// let market = Some("US");
    /// let top_tracks = spotify_client.get_artist_top_tracks(artist_id, market).await?;
    /// for track in top_tracks.tracks {
    ///     println!("Track name: {}", track.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_artist_top_tracks(
        &mut self,
        artist_id: &str,
        market: Option<&str>
    ) -> Result<ArtistTopTracksResponse, Box<dyn Error>> {
        let market_query = market.map_or(String::new(), |m| format!("?market={}", m));
        let path = format!("/artists/{}/top-tracks{}", artist_id, market_query);
        self.get_spotify_data::<ArtistTopTracksResponse>(&path).await.map_err(
            |e| Box::new(e) as Box<dyn Error>
        )
    }

    /// Fetches a list of artists related to a specified artist from the Spotify API.
    ///
    /// # Arguments
    ///
    /// * `artist_id` - A `&str` slice representing the Spotify ID of the target artist.
    ///
    /// # Returns
    ///
    /// * `Result<ArtistsResponse, RustyError>`: On success, this function returns an `ArtistsResponse` containing artists related to the specified artist. On failure, it returns a `RustyError` detailing the error encountered.
    ///
    /// # Examples
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let artist_id = "3TVXtAsR1Inumwj472S9r4";
    /// let related_artists = client.get_related_artists(artist_id).await?;
    /// println!("Related Artists: {:?}", related_artists);
    /// # Ok(())
    /// # }
    /// ```
    /// This function helps users explore the music landscape by introducing them to artists similar to their favorites.
    pub async fn get_related_artists(
        &mut self,
        artist_id: &str
    ) -> Result<ArtistsResponse, RustyError> {
        let path: String = format!("/artists/{}/related-artists", artist_id);
        self.get_spotify_data(&path).await
    }

    /// Fetches available genre seeds from the Spotify API for use in generating track recommendations.
    ///
    /// # Returns
    ///
    /// * `Result<GenreSeedsResponse, RustyError>`: On success, this function returns a `GenreSeedsResponse` containing available genre seeds for recommendation queries. On failure, it returns a `RustyError` detailing the error encountered.
    ///
    /// # Examples
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let genre_seeds = client.get_genre_seeds().await?;
    /// println!("Available Genre Seeds: {:?}", genre_seeds);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_genre_seeds(&mut self) -> Result<GenreSeedsResponse, RustyError> {
        let path = "/recommendations/available-genre-seeds";
        // Use the `get_spotify_data` method to make the request, specifying GenreSeedsResponse as the type parameter
        self.get_spotify_data::<GenreSeedsResponse>(path).await
    }

    /// Fetches detailed information about a specific track from the Spotify API.
    ///
    /// # Arguments
    /// * `track_id` - The Spotify ID of the track.
    ///
    /// # Returns
    /// * `Result<Track, RustyError>` - On success, returns the track's detailed information wrapped
    ///   in a `Track`. On failure, returns an error.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let track_id = "11dFghVXANMlKmJXsNCbNl";
    /// let track = client.get_track(track_id).await?;
    /// println!("Track Name: {}", track.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_track(&mut self, track_id: &str) -> Result<Track, RustyError> {
        let path = format!("/tracks/{track_id}");
        self.get_spotify_data(&path).await
    }

    /// Fetches detailed information for multiple tracks based on their Spotify IDs, optionally filtered by market.
    ///
    /// # Arguments
    /// * `track_ids` - A slice of Spotify IDs for the tracks.
    /// * `market` - An optional ISO 3166-1 alpha-2 country code to specify the market. Tracks will be returned only if they are available in that market.
    ///
    /// # Returns
    /// * `Result<TracksResponse, Box<dyn Error>>` - On success, returns a collection of track data wrapped in `TracksResponse`. On failure, returns an error.
    ///
    /// # Errors
    /// * Returns an error if no track ID is provided or if the number of track IDs exceeds the limit of 20.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let track_ids = vec!["track_id1".to_string(), "track_id2".to_string()];
    /// let tracks = client.get_several_tracks(&track_ids, Some("US")).await?;
    /// for track in tracks.tracks {
    ///     println!("Track name: {}", track.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_several_tracks(
        &mut self,
        track_ids: &[String],
        market: Option<&str>
    ) -> Result<TracksResponse, Box<dyn Error>> {
        if track_ids.len() == 0 {
            return Err(
                Box::new(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Need at least 1 track ID."
                    )
                )
            );
        }

        if track_ids.len() > 20 {
            return Err(
                Box::new(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Maximum of 50 IDs.")
                )
            );
        }
        let market_query = market.map_or(String::new(), |m| format!("&market={}", m));

        let ids_param = track_ids.join(",");
        let path = format!("/tracks?ids={ids_param}{market_query}");

        Ok(self.get_spotify_data(&path).await.map_err(|e| Box::new(e) as Box<dyn Error>)?)
    }

    /// Fetches track recommendations based on specified criteria from the Spotify API.
    ///
    /// This function allows you to generate a list of recommended tracks based on seed artists, tracks, genres, and tunable track attributes. It's ideal for creating personalized music recommendations for users.
    ///
    /// # Arguments
    ///
    /// * `request`: A `RecommendationsRequest` object that includes seed artists, genres, tracks, and tunable track attributes to customize the recommendations.
    ///
    /// # Returns
    ///
    /// * `Result<RecommendationsResponse, Box<dyn Error>>`: On success, it returns a `RecommendationsResponse` containing recommended tracks and their details. On error, it returns a `Box<dyn Error>` detailing what went wrong, such as invalid seed data or API request issues.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustyspoty::{services::client::SpotifyClient, models::recommendations::RecommendationsRequest};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let mut request = RecommendationsRequest::new();
    /// request.seed_genres = Some(vec!["pop".to_string()]);
    /// request.limit = Some(10);
    ///
    /// let recommendations = spotify_client.get_recommendations(&request).await?;
    /// for track in recommendations.tracks {
    ///     println!("Recommended track: {}", track.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_recommendations(
        &mut self,
        request: &RecommendationsRequest
    ) -> Result<RecommendationsResponse, Box<dyn Error>> {
        // Validation logic for seeds
        let total_seeds: usize =
            request.seed_artists.as_ref().map_or(0, Vec::len) +
            request.seed_genres.as_ref().map_or(0, Vec::len) +
            request.seed_tracks.as_ref().map_or(0, Vec::len);

        if total_seeds == 0 || total_seeds > 5 {
            let err_msg = if total_seeds == 0 {
                "At least one seed (artist, genre, or track) is required."
            } else {
                "No more than 5 seeds in total are allowed."
            };
            return Err(Box::new(SeedValidationError::new(err_msg)));
        }

        // Serialize the request object to a JSON value
        let request_json: Value = request.to_json()?;

        // Convert the JSON value to a query string and append it to the endpoint path
        let query_params: String = self.to_query_string(&request_json);
        let path: String = format!("/recommendations?{}", query_params);

        // Use the `get_spotify_data` method to make the request
        Ok(
            self
                .get_spotify_data::<RecommendationsResponse>(&path).await
                .map_err(|e| Box::new(e) as Box<dyn Error>)?
        )
    }

    /// Fetches data for a specific playlist from the Spotify API.
    ///
    /// # Arguments
    /// * `playlist_id` - A string representing the Spotify ID of the playlist.
    ///
    /// # Returns
    /// * `Result<Playlist, RustyError>`: On success, returns detailed information about the playlist. On failure, returns an error encapsulated in `RustyError`.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::services::client::SpotifyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClient::new("client_id".to_string(), "client_secret".to_string());
    /// let playlist_id = "37i9dQZF1DXcBWIGoYBM5M";
    /// let playlist_info = client.get_playlist(playlist_id).await?;
    /// println!("Playlist Name: {}", playlist_info.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_playlist(&mut self, playlist_id: &str) -> Result<Playlist, RustyError> {
        let path = format!("/playlists/{playlist_id}");
        self.get_spotify_data(&path).await
    }

    /// Converts a `serde_json::Value` into a URL-encoded query string.
    ///
    /// This utility function is used internally to convert API parameters stored in a `serde_json::Value` into a string format suitable for HTTP query parameters.
    ///
    /// # Arguments
    /// * `params` - A `serde_json::Value` representing the JSON object containing the query parameters.
    ///
    /// # Returns
    /// * `String`: A URL-encoded string representing the query parameters.
    fn to_query_string(&self, params: &Value) -> String {
        params.as_object().map_or_else(String::new, |obj| {
            obj.iter()
                .filter_map(|(key, value)| {
                    if value.is_array() {
                        let vals: Vec<String> = value
                            .as_array()?
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        Some(format!("{}={}", key, vals.join(",")))
                    } else {
                        value.as_str().map(|v| format!("{}={}", key, v))
                    }
                })
                .collect::<Vec<String>>()
                .join("&")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    // use serde_json::json;

    fn setup() -> SpotifyClient {
        dotenv::dotenv().ok();
        // Setup
        let client_id: String = env::var("SPOTIFY_CLIENT_ID").expect("Expected a client id");
        let client_secret: String = env
            ::var("SPOTIFY_CLIENT_SECRET")
            .expect("Expected a client secret");
        SpotifyClient::new(client_id, client_secret)
    }

    #[tokio::test]
    async fn test_client() {
        let mut client = setup();

        // Test fetching a track
        let track_result = client.get_track("4iV5W9uYEdYUVa79Axb7Rh").await;
        assert!(track_result.is_ok());

        // Test fetching an album
        let album_result = client.get_album("1vi1WySkgPGkbR8NnQzlXu").await;
        assert!(album_result.is_ok());

        // Test fetching an artist
        let artist_result = client.get_artist("0TnOYISbd1XYRBk9myaseg").await;
        assert!(artist_result.is_ok());

        // Test fetching a playlist
        let playlist_result = client.get_playlist("37i9dQZF1DXcBWIGoYBM5M").await;
        assert!(playlist_result.is_ok());

        // Test fetching genre seeds
        let genre_seeds_result = client.get_genre_seeds().await;
        assert!(genre_seeds_result.is_ok());

        // Test fetching recommendations
        let mut recommendations_request = RecommendationsRequest::new(); // Assuming default() gives a valid request
        recommendations_request.seed_genres = Some(
            genre_seeds_result.unwrap().genres[0..2].to_vec()
        );
        let recommendations_result = client.get_recommendations(&recommendations_request).await;
        assert!(recommendations_result.is_ok());

        // Extend with more tests as needed
    }
}