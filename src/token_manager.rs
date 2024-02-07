use crate::{ models::auth::{ ClientCredsAuthRequest, ClientCredsAuthResponse }, RustyError };
use std::time::{ SystemTime, UNIX_EPOCH };

/// Manages authentication tokens for Spotify API.
///
/// This struct is responsible for obtaining and refreshing Spotify access tokens
/// as needed, using the client credentials grant flow.
pub struct SpotifyTokenManager {
    /// The current access token for API requests, if available.
    access_token: Option<String>,
    /// The UNIX timestamp at which the current access token expires.
    expires_at: Option<u64>,
    /// The Spotify API client ID.
    client_id: String,
    /// The Spotify API client secret.
    client_secret: String,
}

impl SpotifyTokenManager {
    /// Creates a new `SpotifyTokenManager` with the provided client credentials.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Your application's Spotify client ID.
    /// * `client_secret` - Your application's Spotify client secret.
    pub fn new(client_id: String, client_secret: String) -> Self {
        SpotifyTokenManager {
            access_token: None,
            expires_at: None,
            client_id,
            client_secret,
        }
    }

    /// Checks if the stored access token is still valid.
    ///
    /// Compares the current time with the token's expiration time to determine validity.
    fn is_token_valid(&self) -> bool {
        self.expires_at
            .map(|expiry| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() < expiry)
            .unwrap_or(false)
    }

    /// Requests a new access token from the Spotify Accounts service.
    ///
    /// Uses the client credentials grant to obtain a new token and updates `access_token` and `expires_at`.
    async fn request_new_token(&mut self) -> Result<(), RustyError> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://accounts.spotify.com/api/token")
            .form(
                &(ClientCredsAuthRequest {
                    grant_type: "client_credentials".to_owned(),
                    client_id: self.client_id.clone(),
                    client_secret: self.client_secret.clone(),
                })
            )
            .send().await?;

        // Check for authentication error (e.g., HTTP status 400/401)
        if !response.status().is_success() {
            let error_message = response
                .text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            // You might want to parse the response body to provide a more specific error message
            return Err(RustyError::TokenAuthentication(error_message));
        }

        let res = response.json::<ClientCredsAuthResponse>().await?;

        // Update the token and expiration time, subtracting 60 seconds to account for potential timing issues
        self.access_token = Some(res.access_token);
        self.expires_at = Some(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + res.expires_in - 60
        );

        Ok(())
    }

    /// Returns a valid access token.
    ///
    /// Checks the validity of the current token and requests a new one if necessary.
    /// Returns the current token if it's valid, or a new one if it was refreshed.
    pub async fn get_valid_token(&mut self) -> Result<String, RustyError> {
        if !self.is_token_valid() {
            self.request_new_token().await?;
        }
        Ok(self.access_token.clone().unwrap()) // Safe unwrap because request_new_token() ensures access_token is Some
    }
}
