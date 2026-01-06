use models::ErrorLocation;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum WsError {
    #[error("Validation Error: {message} {location}")]
    Validation {
        message: String,
        location: ErrorLocation,
    },
}