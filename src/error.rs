// Error handling utilities for a Rust application interfacing with web APIs and handling JSON data.

use std::error::Error;
use std::fmt;

pub type RustyResult<T> = Result<T, RustyError>;

/// A custom error type for the application, covering various error scenarios encountered.
///
/// This enum encapsulates different kinds of errors that can occur in the application, including specific handling for rate limiting by the Spotify API.
#[derive(Debug)]
pub enum RustyError {
    /// Represents errors that occur during network requests.
    Network(reqwest::Error),
    /// Represents errors that occur while parsing JSON data.
    ParseJson(serde_json::Error),
    Io(std::io::Error),
    /// Represents errors related to token authentication failures.
    TokenAuthentication(String),
    /// Represents being rate limited by the Spotify API and includes the duration to wait.
    SpotifyRateLimited(u64), // Duration in seconds to wait before retrying
    /// Represents unexpected or miscellaneous errors.
    Unexpected(String),
}

impl RustyError {
    pub fn invalid_input(msg: &str) -> Self {
        RustyError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, msg))
    }
}

impl fmt::Display for RustyError {
    /// Provides a human-readable description of the error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RustyError::Network(e) => write!(f, "network error: {e}"),
            RustyError::ParseJson(e) => write!(f, "failed to parse data: {e}"),
            RustyError::TokenAuthentication(msg) => write!(f, "token authentication error: {msg}"),
            RustyError::SpotifyRateLimited(duration) =>
                write!(f, "rate limited by Spotify API, retry after {duration} seconds"),
            RustyError::Unexpected(msg) => write!(f, "an unexpected error occurred: {msg}"),
            RustyError::Io(e) => write!(f, "input/output error: {e}"),
        }
    }
}

impl Error for RustyError {}

impl From<reqwest::Error> for RustyError {
    /// Converts `reqwest::Error` into `RustyError::Network`.
    fn from(err: reqwest::Error) -> RustyError {
        RustyError::Network(err)
    }
}

impl From<serde_json::Error> for RustyError {
    /// Converts `serde_json::Error` into `RustyError::Parse`.
    fn from(err: serde_json::Error) -> RustyError {
        RustyError::ParseJson(err)
    }
}

impl From<std::io::Error> for RustyError {
    fn from(value: std::io::Error) -> Self {
        RustyError::Io(value)
    }
}
