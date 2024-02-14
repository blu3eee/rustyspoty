use std::{ fmt::Debug, time::Duration };

use reqwest::{ Client as ReqwestClient, StatusCode };
use serde::{ de::DeserializeOwned, Serialize };
use serde_json::Value;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    cache::Cache,
    models::{ album::*, artist::*, page::Page, playlist::*, recommendations::*, track::* },
    token_manager::SpotifyTokenManager,
    RustyError,
    RustyResult,
};

/// A client for interacting with the Spotify Web API.
///
/// This client simplifies the process of making authenticated requests to the Spotify API. It handles
/// OAuth token management, caching of responses to reduce load and improve performance, and provides
/// a unified interface for various Spotify API endpoints. The client supports server-to-server
/// interactions with Spotify, where user authorization is not required.
///
/// The `SpotifyClient` includes a token manager for handling OAuth tokens, a `reqwest` HTTP client for
/// making requests, and a cache for storing and retrieving API responses. The cache reduces the
/// number of requests made to the Spotify API by temporarily storing data that is likely to be reused.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use rustyspoty::SpotifyClientCredentials;
///
/// #[tokio::main]
/// async fn main() {
///     let client_id = "your_spotify_client_id".to_string();
///     let client_secret = "your_spotify_client_secret".to_string();
///
///     // Create a new SpotifyClient instance.
///     let mut spotify_client = SpotifyClientCredentials::new(client_id, client_secret);
///
///     // Example: Fetch details for a specific album.
///     let album_id = "4aawyAB9vmqN3uQ7FjRGTy";
///     match spotify_client.get_album(album_id).await {
///         Ok(album) => println!("Album Name: {}", album.name),
///         Err(e) => eprintln!("Error occurred: {}", e),
///     }
/// }
/// ```
///
/// The client automatically handles token refreshes and caches responses for efficient use.
pub struct SpotifyClientCredentials {
    /// Manages the Spotify API authentication tokens, abstracting away the details of token
    /// acquisition, refresh, and storage.
    token_manager: SpotifyTokenManager,

    /// A `reqwest::Client` instance for making HTTP requests. This client is used to send requests
    /// to the Spotify Web API, handling aspects like setting request headers and parsing responses.
    http_client: ReqwestClient,

    /// A cache for storing responses from the Spotify API. The cache aims to reduce the number of
    /// API requests by reusing previously fetched data. The cache stores data as `serde_json::Value`,
    /// allowing for flexible handling of different response structures.
    cache: AsyncMutex<Cache<Value>>,
}

// Define the base URL for the Spotify API as a constant
const SPOTIFY_API_BASE_URL: &str = "https://api.spotify.com/v1";

impl SpotifyClientCredentials {
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
        SpotifyClientCredentials {
            token_manager,
            http_client,
            cache: AsyncMutex::new(Cache::new(Duration::from_secs(600))),
        }
    }

    /// Updates the cache with a new value for a given key or inserts it if the key does not exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which to store the value in the cache.
    /// * `value` - The value to store, of generic type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn example() {
    /// # let mut client_credentials = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// client_credentials.update_cache("artist:1".to_string(), serde_json::json!({"name": "Artist Name"})).await;
    /// # }
    /// ```
    pub async fn update_cache(&self, key: String, value: Value) {
        self.cache.lock().await.set(key, value);
    }

    /// Retrieves a value from the cache if it exists and has not expired.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the cache entry to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<T>` which is `Some(T)` if the key exists and has not expired, or `None` if the key does not exist or has expired.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn example() -> Option<serde_json::Value> {
    /// # let mut client_credentials = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let value = client_credentials.check_cache("artist:1").await;
    /// value
    /// # }
    /// ```
    pub async fn check_cache(&self, key: &str) -> Option<Value> {
        self.cache.lock().await.get(key)
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
    async fn get_spotify_data<T>(&mut self, path: &str) -> RustyResult<T>
        where
            T: DeserializeOwned + Serialize + Debug // Ensure T can be serialized for caching
    {
        let cache_key = path.to_string();

        // Attempt to retrieve from cache first
        {
            // Scope for the cache lock to ensure it's dropped before await points
            let cache_lock = self.cache.lock().await;
            if let Some(cached) = cache_lock.get(&cache_key) {
                // Deserialize the cached JSON to the requested type
                if let Ok(cached_data) = serde_json::from_value::<T>(cached.clone()) {
                    return Ok(cached_data);
                }
            }
        } // Cache lock is dropped here

        // Proceed with API request if not found in cache or cache is stale
        let token = self.token_manager.get_valid_token().await?;
        let url = format!("{SPOTIFY_API_BASE_URL}{path}");
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send().await?;

        // Handle rate limiting or other errors as needed here
        match response.status() {
            StatusCode::OK => {
                let data = response.json::<T>().await?;
                {
                    // Scope for the cache lock to ensure it's dropped right after use
                    let cache_lock = self.cache.lock().await;
                    cache_lock.set(cache_key, serde_json::to_value(&data)?);
                } // Cache lock is dropped here
                Ok(data)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                if
                    let Some(retry_after) = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                {
                    // Convert retry_after to a Duration
                    // let wait_time = Duration::from_secs(retry_after);
                    // Retry the request or return an error indicating rate limiting
                    // For simplicity, here we return a RateLimited error
                    Err(RustyError::SpotifyRateLimited(retry_after))
                } else {
                    // If the Retry-After header is missing or invalid
                    Err(
                        RustyError::Unexpected(
                            "Rate limited by Spotify Web API, but no retry time provided.".into()
                        )
                    )
                }
            }
            _ => {
                // Handle other errors based on status code
                Err(
                    RustyError::Unexpected(
                        format!("API request failed with status: {}", response.status())
                    )
                )
            }
        }
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
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let album_id = "1DFixLWuPkv3KT3TnV35m3";
    /// let album = spotify_client.get_album(album_id).await?;
    /// println!("Album name: {}", album.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_album(&mut self, album_id: &str) -> RustyResult<Album> {
        let path = format!("/albums/{album_id}");
        self.get_spotify_data(&path).await
    }

    /// Fetches detailed information for several albums based on their Spotify IDs.
    ///
    /// This method first checks if the requested album information is available in the cache
    /// and not expired. If so, it returns the cached data directly, minimizing the number
    /// of API calls. For any albums not found in the cache or if the cached data is expired,
    /// it fetches the data from the Spotify API, updates the cache with the new data, and
    /// returns the combined results.
    ///
    /// # Arguments
    /// * `album_ids`: A slice of Spotify album IDs. Each ID must correspond to an album on Spotify.
    ///
    /// # Returns
    /// * `RustyResult<Albums>`: On success, returns an `Albums` object containing detailed
    ///   information about each requested album. On failure, returns a `RustyError` detailing
    ///   the issue, such as exceeding the maximum number of IDs allowed.
    ///
    /// # Errors
    /// * Returns an error if the provided list of album IDs is empty or exceeds 20, as this is
    ///   the Spotify API's limit for this type of request.
    /// * Returns a `RustyError::InvalidInput` for invalid input parameters.
    ///
    /// # Caching
    /// * The method optimizes data fetching by leveraging a caching mechanism. It checks the cache
    ///   for each requested album ID and uses the cached data if available and not expired.
    /// * For any missing or expired albums, it fetches the data for all requested albums from the
    ///   Spotify API and updates the cache accordingly.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let album_ids = ["1o2NpYGqHiCq7FoiYdyd1x".to_string(), "4tZwfgrHOc3mvqYlEYSvVi".to_string()];
    /// let result = client.get_several_albums(&album_ids).await;
    /// if let Ok(albums_response) = result {
    ///     for album in albums_response.albums {
    ///         println!("Album: {}", album.name);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_several_albums(&mut self, album_ids: &[String]) -> RustyResult<Albums> {
        if album_ids.is_empty() {
            return Err(RustyError::invalid_input("Please provide at least 1 album ID."));
        }
        if album_ids.len() > 20 {
            return Err(RustyError::invalid_input("Maximum of 20 IDs."));
        }

        let mut albums_to_fetch = Vec::new();
        let mut albums_from_cache = Vec::new();

        // Check cache first
        for id in album_ids {
            let cache_key = format!("/albums/{id}");
            if let Some(cached_album) = self.check_cache(&cache_key).await {
                albums_from_cache.push(serde_json::from_value::<Album>(cached_album)?);
            } else {
                albums_to_fetch.push(id.clone());
            }
        }

        // If all albums were found in cache, return them directly
        if albums_to_fetch.is_empty() {
            return Ok(Albums { albums: albums_from_cache });
        }

        // Fetch missing albums from Spotify API
        let ids_param = album_ids.join(",");
        let path = format!("/albums?ids={}", ids_param);
        let fetched_albums: Albums = self.get_spotify_data(&path).await?;

        // Update cache with fetched albums
        for album in &fetched_albums.albums {
            let cache_key = format!("/albums/{}", album.id);
            self.update_cache(cache_key, serde_json::to_value(album)?).await;
        }

        // Combine cached albums with fetched albums before returning
        let combined_albums = [albums_from_cache, fetched_albums.albums].concat();
        Ok(Albums { albums: combined_albums })
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    pub async fn get_album_tracks(&mut self, album_id: &str) -> RustyResult<Page<SimplifiedTrack>> {
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    ) -> RustyResult<NewAlbums> {
        let limit = limit.unwrap_or(20).min(50).max(1); // Ensures limit is within 1-50
        let offset = offset.unwrap_or(0).max(0); // Ensures offset is non-negative

        let query_params = format!("?limit={}&offset={}", limit, offset);
        let path = format!("/browse/new-releases{}", query_params);

        self.get_spotify_data::<NewAlbums>(&path).await
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let artist = client.get_artist("artist_id").await?;
    /// println!("Artist Name: {}", artist.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_artist(&mut self, artist_id: &str) -> RustyResult<Artist> {
        let path = format!("/artists/{artist_id}");
        self.get_spotify_data(&path).await
    }

    /// Retrieves information for multiple artists based on their Spotify IDs.
    ///
    /// This method first checks if the requested artist information is available in the cache
    /// and not expired. If so, it returns the cached data directly, minimizing the number
    /// of API calls. For any artists not found in the cache or if the cached data is expired,
    /// it fetches the data from the Spotify API, updates the cache with the new data, and
    /// returns the combined results.
    ///
    /// # Arguments
    /// * `artist_ids` - A slice of Spotify IDs for the artists. Maximum of 50 IDs allowed.
    ///
    /// # Returns
    /// * `RustyResult<Artists>`: On success, returns an `Artists` object containing detailed
    ///   information about each requested artist. On failure, returns a `RustyError` detailing
    ///   the issue.
    ///
    /// # Errors
    /// * Returns an error if no artist IDs are provided or if the number of IDs exceeds the limit of 50.
    ///
    /// # Caching
    /// * The method leverages a caching mechanism to optimize data fetching. It checks the cache
    ///   for each requested artist ID and uses the cached data if available and not expired.
    /// * For any missing or expired artists, it fetches the data from the Spotify API and updates
    ///   the cache accordingly.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let artist_ids = vec!["artist_id1".to_string(), "artist_id2".to_string()];
    /// let artists = client.get_several_artists(&artist_ids).await?;
    /// for artist in artists.artists {
    ///     println!("Artist Name: {}", artist.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_several_artists(&mut self, artist_ids: &[String]) -> RustyResult<Artists> {
        if artist_ids.is_empty() {
            return Err(RustyError::invalid_input("Please provide at least 1 artist ID."));
        }
        if artist_ids.len() > 50 {
            return Err(RustyError::invalid_input("Maximum of 50 IDs."));
        }

        let mut artists_to_fetch = Vec::new();
        let mut artists_from_cache = Vec::new();

        // Check cache first
        for id in artist_ids {
            let cache_key = format!("/artists/{id}");
            if let Some(cached_artist) = self.check_cache(&cache_key).await {
                artists_from_cache.push(serde_json::from_value::<Artist>(cached_artist)?);
            } else {
                artists_to_fetch.push(id.clone());
            }
        }

        // If all artists were found in cache, return them directly
        if artists_to_fetch.is_empty() {
            return Ok(Artists { artists: artists_from_cache });
        }

        // Fetch missing artists from Spotify API
        let ids_param = artists_to_fetch.join(",");
        let path = format!("/artists?ids={ids_param}");
        let fetched_artists: Artists = self.get_spotify_data(&path).await?;

        // Update cache with fetched artists
        for artist in &fetched_artists.artists {
            let cache_key = format!("/artists/{}", artist.id);
            self.update_cache(cache_key, serde_json::to_value(artist)?).await;
        }

        // Combine cached artists with fetched artists before returning
        let combined_artists = [artists_from_cache, fetched_artists.artists].concat();
        Ok(Artists { artists: combined_artists })
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClientCredentials::new("your_client_id".to_string(), "your_client_secret".to_string());
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
    ) -> RustyResult<Page<SimplifiedAlbum>> {
        let path = format!("/artists/{artist_id}/albums");
        self.get_spotify_data(&path).await
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    ) -> RustyResult<TracksResponse> {
        let market_query = market.map_or(String::new(), |m| format!("?market={}", m));
        let path = format!("/artists/{}/top-tracks{}", artist_id, market_query);
        self.get_spotify_data::<TracksResponse>(&path).await
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let artist_id = "3TVXtAsR1Inumwj472S9r4";
    /// let related_artists = client.get_related_artists(artist_id).await?;
    /// println!("Related Artists: {:?}", related_artists);
    /// # Ok(())
    /// # }
    /// ```
    /// This function helps users explore the music landscape by introducing them to artists similar to their favorites.
    pub async fn get_related_artists(&mut self, artist_id: &str) -> Result<Artists, RustyError> {
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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

    /// Fetches detailed information for multiple tracks based on their Spotify IDs,
    /// using caching to optimize API usage.
    ///
    /// # Arguments
    /// * `track_ids` - A slice of Spotify IDs for the tracks.
    /// * `market` - An optional market code to filter tracks available in a specific market.
    ///
    /// # Returns
    /// * `RustyResult<TracksResponse>`: On success, returns a `TracksResponse` object containing detailed
    ///   information about each requested track. On failure, returns a `RustyError` detailing the issue.
    ///
    /// # Caching
    /// * Checks the cache for each requested track ID and uses cached data if available and valid.
    /// * Updates the cache with new data fetched from the Spotify API for missing or expired tracks.
    ///
    /// # Example
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    ) -> RustyResult<TracksResponse> {
        if track_ids.is_empty() {
            return Err(RustyError::invalid_input("Please provide at least 1 track ID."));
        }
        if track_ids.len() > 20 {
            return Err(RustyError::invalid_input("Maximum of 20 IDs."));
        }

        let market_query = market.map_or(String::new(), |m| format!("&market={}", m));
        let mut tracks_to_fetch = Vec::new();
        let mut tracks_from_cache: Vec<Track> = Vec::new();

        // Check cache first
        for id in track_ids {
            let cache_key = format!("/tracks/{id}{market_query}");
            if let Some(cached_track) = self.check_cache(&cache_key).await {
                tracks_from_cache.push(serde_json::from_value::<Track>(cached_track)?);
            } else {
                tracks_to_fetch.push(id.clone());
            }
        }

        // If all tracks were found in cache, return them directly
        if tracks_to_fetch.is_empty() {
            return Ok(TracksResponse { tracks: tracks_from_cache });
        }

        // Fetch missing tracks from Spotify API
        let ids_param = tracks_to_fetch.join(",");
        let path = format!("/tracks?ids={ids_param}{market_query}");
        let fetched_tracks: TracksResponse = self.get_spotify_data(&path).await?;

        // Update cache with fetched tracks
        for track in &fetched_tracks.tracks {
            let cache_key = format!("/tracks/{}/{}", track.id, market.unwrap_or_default());
            self.update_cache(cache_key, serde_json::to_value(track)?).await;
        }

        // Combine cached tracks with fetched tracks before returning
        let combined_tracks = [tracks_from_cache, fetched_tracks.tracks].concat();
        Ok(TracksResponse { tracks: combined_tracks })
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
    /// # use rustyspoty::{SpotifyClientCredentials, models::recommendations::RecommendationsRequest};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut spotify_client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
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
    ) -> RustyResult<RecommendationsResponse> {
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
            return Err(RustyError::invalid_input(err_msg));
        }

        // Serialize the request object to a JSON value
        let request_json: Value = request.to_json()?;

        // Convert the JSON value to a query string and append it to the endpoint path
        let query_params: String = self.to_query_string(&request_json);
        let path: String = format!("/recommendations?{}", query_params);

        self.get_spotify_data::<RecommendationsResponse>(&path).await
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
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let playlist_id = "37i9dQZF1DXcBWIGoYBM5M";
    /// let playlist_info = client.get_playlist(playlist_id).await?;
    /// println!("Playlist Name: {}", playlist_info.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_playlist(&mut self, playlist_id: &str) -> RustyResult<Playlist> {
        let path = format!("/playlists/{playlist_id}");
        self.get_spotify_data(&path).await
    }

    /// Converts a `serde_json::Value` into a URL-encoded query string.
    ///
    /// This utility function is designed to serialize API parameters stored in a `serde_json::Value`
    /// into a string format suitable for use as HTTP query parameters. It supports arrays, strings,
    /// numbers, and booleans, accurately representing these types in the query string. This ensures
    /// that API requests can include a diverse range of parameters.
    ///
    /// # Arguments
    /// * `params` - A `serde_json::Value` representing the JSON object with the API query parameters.
    ///
    /// # Returns
    /// * `String` - A URL-encoded string of the query parameters.
    ///
    /// # Examples
    ///
    /// Converting a JSON object with various types of parameters into a query string:
    ///
    /// ```
    /// # use rustyspoty::SpotifyClientCredentials;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SpotifyClientCredentials::new("client_id".to_string(), "client_secret".to_string());
    /// let params = serde_json::json!({
    ///     "limit": 10,
    ///     "seed_genres": ["acoustic", "afrobeat"],
    ///     "market": "US",
    ///     "min_energy": 0.4
    /// });
    ///
    /// let query_string = client.to_query_string(&params);
    /// assert_eq!(query_string, "limit=10&seed_genres=acoustic,afrobeat&market=US&min_energy=0.4");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: This function ignores null values and objects, focusing on directly serializable types.
    pub fn to_query_string(&self, params: &Value) -> String {
        params.as_object().map_or_else(String::new, |obj| {
            obj.iter()
                .filter_map(|(key, value)| {
                    match value {
                        Value::Array(vals) => {
                            // Handle arrays: join their string representations with commas
                            let vals_str: Vec<String> = vals
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                            Some(format!("{}={}", key, vals_str.join(",")))
                        }
                        Value::String(str_val) => {
                            // Handle strings directly
                            Some(format!("{}={}", key, str_val))
                        }
                        // Handle numerical and boolean values by converting them to strings
                        Value::Number(num_val) => Some(format!("{}={}", key, num_val)),
                        Value::Bool(bool_val) => Some(format!("{}={}", key, bool_val)),
                        // Ignore other types (e.g., null, objects)
                        _ => None,
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

    fn setup() -> SpotifyClientCredentials {
        dotenv::dotenv().ok();
        // Setup
        let client_id: String = env::var("SPOTIFY_CLIENT_ID").expect("Expected a client id");
        let client_secret: String = env
            ::var("SPOTIFY_CLIENT_SECRET")
            .expect("Expected a client secret");
        SpotifyClientCredentials::new(client_id, client_secret)
    }

    #[tokio::test]
    async fn test_client() {
        let mut client = setup();

        let genres_result = client.get_genre_seeds().await;
        assert!(genres_result.is_ok());

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
        recommendations_request.limit = Some(10);

        let recommendations_result = client.get_recommendations(&recommendations_request).await;
        assert!(recommendations_result.is_ok());
        dbg!(recommendations_result.unwrap().tracks.len());

        // Extend with more tests as needed
    }
}
