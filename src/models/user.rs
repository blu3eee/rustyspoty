use serde::{ Deserialize, Serialize };

use super::ExternalUrls;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub display_name: Option<String>,
    pub external_urls: ExternalUrls,
    pub r#type: String,
}
