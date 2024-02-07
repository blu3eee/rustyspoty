use serde::{ Deserialize, Serialize };

#[derive(Serialize)]
pub struct ClientCredsAuthRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct ClientCredsAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}
