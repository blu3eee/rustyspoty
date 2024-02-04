// Error handling utilities for a Rust application interfacing with web APIs and handling JSON data.

use std::error::Error;
use std::fmt;

/// A custom error type for the application, covering various error scenarios encountered.
///
/// This enum encapsulates different kinds of errors that can occur in the application, including
/// network errors, JSON parsing errors, authentication errors, and other unexpected errors.
#[derive(Debug)]
pub enum RustyError {
    /// Represents errors that occur during network requests.
    Network(reqwest::Error),
    /// Represents errors that occur while parsing JSON data.
    Parse(serde_json::Error),
    /// Represents errors related to token authentication failures.
    TokenAuthentication(String),
    /// Represents unexpected or miscellaneous errors.
    Unexpected(String),
}

impl fmt::Display for RustyError {
    /// Provides a human-readable description of the error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RustyError::Network(e) => write!(f, "network error: {}", e),
            RustyError::Parse(e) => write!(f, "failed to parse data: {}", e),
            RustyError::TokenAuthentication(msg) =>
                write!(f, "token authentication error: {}", msg),
            RustyError::Unexpected(msg) => write!(f, "an unexpected error occurred: {}", msg),
        }
    }
}

impl std::error::Error for RustyError {}

impl From<reqwest::Error> for RustyError {
    /// Converts `reqwest::Error` into `RustyError::Network`.
    fn from(err: reqwest::Error) -> RustyError {
        RustyError::Network(err)
    }
}

impl From<serde_json::Error> for RustyError {
    /// Converts `serde_json::Error` into `RustyError::Parse`.
    fn from(err: serde_json::Error) -> RustyError {
        RustyError::Parse(err)
    }
}

/// Represents validation errors for seed data used in generating recommendations.
///
/// This struct encapsulates validation errors related to seed data inputs. It's used to signal
/// when seed inputs for generating recommendations do not meet the required criteria.
#[derive(Debug)]
pub struct SeedValidationError {
    /// Detailed message describing the validation error.
    details: String,
}

impl SeedValidationError {
    /// Creates a new `SeedValidationError` with a given message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A message describing the validation error.
    pub fn new(msg: &str) -> SeedValidationError {
        SeedValidationError { details: msg.to_string() }
    }
}

impl fmt::Display for SeedValidationError {
    /// Provides a human-readable description of the validation error.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for SeedValidationError {
    /// Returns a description of the error.
    fn description(&self) -> &str {
        &self.details
    }
}
