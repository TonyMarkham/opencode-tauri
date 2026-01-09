use crate::ErrorLocation;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum RedactError {
    #[error("Serialization Error: {message} {location}")]
    Serialization {
        message: String,
        location: ErrorLocation,
    },
}
