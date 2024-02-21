use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rand::{ distributions::Alphanumeric, Rng };
use reqwest::{ Client as HttpClient, Url };
use serde::{ Deserialize, Serialize };
use sha2::{ Digest, Sha256 };
use std::str;

/// Represents errors that might occur during the OAuth process.
#[derive(Debug)]
pub enum OAuthError {
    HttpError(reqwest::Error),
    UrlParseError(url::ParseError),
    Base64DecodeError(base64::DecodeError),
    Other(String),
}

impl From<reqwest::Error> for OAuthError {
    fn from(err: reqwest::Error) -> Self {
        OAuthError::HttpError(err)
    }
}

impl From<url::ParseError> for OAuthError {
    fn from(err: url::ParseError) -> Self {
        OAuthError::UrlParseError(err)
    }
}

impl From<base64::DecodeError> for OAuthError {
    fn from(err: base64::DecodeError) -> Self {
        OAuthError::Base64DecodeError(err)
    }
}

/// Represents the OAuth client for performing the Authorization Code with PKCE Flow.
pub struct SpotifyOAuth {
    client_id: String,
    redirect_uri: String,
    scope: String,
    code_verifier: String,
    http_client: HttpClient,
}

impl SpotifyOAuth {
    pub fn new(client_id: String, redirect_uri: String, scope: String) -> Self {
        let code_verifier = Self::generate_code_verifier();
        let http_client = HttpClient::new();

        SpotifyOAuth {
            client_id,
            redirect_uri,
            scope,
            code_verifier,
            http_client,
        }
    }

    /// Generates a code verifier for the PKCE flow.
    fn generate_code_verifier() -> String {
        rand::thread_rng().sample_iter(&Alphanumeric).take(128).map(char::from).collect()
    }

    /// Generates the code challenge from the code verifier using SHA256 and base64 URL-safe encoding without padding.
    fn generate_code_challenge(&self) -> Result<String, OAuthError> {
        let digest = Sha256::digest(self.code_verifier.as_bytes());
        let encoded = URL_SAFE_NO_PAD.encode(&digest);
        Ok(encoded)
    }

    /// Constructs the authorization URL to which the user should be redirected.
    pub async fn get_authorize_url(&self) -> Result<String, OAuthError> {
        let code_challenge = self.generate_code_challenge()?;
        let mut auth_url: Url = Url::parse("https://accounts.spotify.com/authorize")?;
        auth_url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("scope", &self.scope)
            .append_pair("code_challenge_method", "S256")
            .append_pair("code_challenge", &code_challenge);

        Ok(auth_url.to_string())
    }

    /// Exchanges the authorization code for an access token.
    pub async fn request_access_token(
        &self,
        code: &str
    ) -> Result<AccessTokenResponse, OAuthError> {
        let token_url = "https://accounts.spotify.com/api/token";
        let params = [
            ("client_id", self.client_id.as_str()), // Convert String to &str
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()), // Convert String to &str
            ("code_verifier", self.code_verifier.as_str()), // Convert String to &str
        ];

        let response = self.http_client
            .post(token_url)
            .form(&params)
            .send().await?
            .json::<AccessTokenResponse>().await?;

        Ok(response)
    }
}

/// Represents the response from Spotify after exchanging an authorization code for an access token.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: usize,
    refresh_token: Option<String>,
}
