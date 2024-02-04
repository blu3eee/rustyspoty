use serde::{ Deserialize, Serialize };
use serde_json::Value;

use super::track::Track;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecommendationsRequest {
    /// The target size of the list of recommended tracks.
    ///
    /// For seeds with unusually small pools or when highly restrictive filtering is applied,
    /// it may be impossible to generate the requested number of recommended tracks.
    /// Debugging information for such cases is available in the response.
    /// Default: 20. Minimum: 1. Maximum: 100.
    ///
    /// Default: limit=20
    /// Range: 1 - 100
    /// Example: limit=10
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    /// An ISO 3166-1 alpha-2 country code.
    ///
    /// If a country code is specified, only content that is available in that market will be returned.
    ///
    /// If a valid user access token is specified in the request header,
    /// the country associated with the user account will take priority over this parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    /// A comma separated list of any genres in the set of available genre seeds.
    /// Up to 5 seed values may be provided in any combination of seed_artists, seed_tracks and seed_genres.
    ///
    /// Note: only required if seed_artists and seed_tracks are not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_genres: Option<Vec<String>>,
    /// A comma separated list of `Spotify IDs` for seed artists.
    /// Up to 5 seed values may be provided in any combination of seed_artists, seed_tracks and seed_genres.
    ///
    /// Note: only required if seed_genres and seed_tracks are not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_artists: Option<Vec<String>>,
    /// A comma separated list of Spotify IDs for a seed track.
    /// Up to 5 seed values may be provided in any combination of seed_artists, seed_tracks and seed_genres.
    ///
    /// Note: only required if seed_artists and seed_genres are not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_tracks: Option<Vec<String>>,
    // Tunable track attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_acousticness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_acousticness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_acousticness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_danceability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_danceability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_danceability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_duration_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_duration_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_energy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_energy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_energy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_instrumentalness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_instrumentalness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_instrumentalness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_key: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_key: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_key: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_liveness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_liveness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_liveness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_loudness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_loudness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_loudness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mode: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mode: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mode: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_popularity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_popularity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_popularity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_speechiness: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_speechiness: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_speechiness: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_tempo: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tempo: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tempo: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_time_signature: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_signature: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_time_signature: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_valence: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_valence: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_valence: Option<i32>,
}

impl RecommendationsRequest {
    // Create a new instance with default values
    pub fn new() -> Self {
        RecommendationsRequest {
            seed_genres: None,
            seed_artists: None,
            seed_tracks: None,
            limit: None,
            market: None,
            min_acousticness: None,
            max_acousticness: None,
            target_acousticness: None,
            min_danceability: None,
            max_danceability: None,
            target_danceability: None,
            min_duration_ms: None,
            max_duration_ms: None,
            target_duration_ms: None,
            min_energy: None,
            max_energy: None,
            target_energy: None,
            min_instrumentalness: None,
            max_instrumentalness: None,
            target_instrumentalness: None,
            min_key: None,
            max_key: None,
            target_key: None,
            min_liveness: None,
            max_liveness: None,
            target_liveness: None,
            min_loudness: None,
            max_loudness: None,
            target_loudness: None,
            min_mode: None,
            max_mode: None,
            target_mode: None,
            min_popularity: None,
            max_popularity: None,
            target_popularity: None,
            min_speechiness: None,
            max_speechiness: None,
            target_speechiness: None,
            min_tempo: None,
            max_tempo: None,
            target_tempo: None,
            min_time_signature: None,
            max_time_signature: None,
            target_time_signature: None,
            min_valence: None,
            max_valence: None,
            target_valence: None,
        }
    }

    // Deserialize from JSON using serde
    pub fn from_json(json: &Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone())
    }

    // Serialize to JSON using serde
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenreSeedsResponse {
    pub genres: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Seed {
    pub afterFilteringSize: i32,
    pub afterRelinkingSize: i32,
    pub href: Option<String>,
    pub id: String,
    pub initialPoolSize: i32,
    pub r#type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RecommendationsResponse {
    pub seeds: Vec<Seed>,
    pub tracks: Vec<Track>,
}
