use common::ErrorLocation;

use std::panic::Location;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum OpencodeClientError {
    #[error("HTTP Error: {message} {location}")]
    Http {
        message: String,
        location: ErrorLocation,
    },

    #[error("JSON Error: {message} {location}")]
    Json {
        message: String,
        location: ErrorLocation,
    },

    #[error("URL Parse Error: {message} {location}")]
    UrlParse {
        message: String,
        location: ErrorLocation,
    },

    #[error("Server Error: {message} {location}")]
    Server {
        message: String,
        location: ErrorLocation,
    },
}

impl From<url::ParseError> for OpencodeClientError {
    #[track_caller]
    fn from(error: url::ParseError) -> Self {
        OpencodeClientError::UrlParse {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<reqwest::Error> for OpencodeClientError {
    #[track_caller]
    fn from(error: reqwest::Error) -> Self {
        OpencodeClientError::Http {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<serde_json::Error> for OpencodeClientError {
    #[track_caller]
    fn from(error: serde_json::Error) -> Self {
        OpencodeClientError::Json {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}
