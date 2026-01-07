use common::ErrorLocation;

use std::io::Error as IoError;
use std::panic::Location;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum IpcError {
    #[error("Handshake Error: {message} {location}")]
    Handshake {
        message: String,
        location: ErrorLocation,
    },

    #[error("Send Error: {message} {location}")]
    Send {
        message: String,
        location: ErrorLocation,
    },

    #[error("Read Error: {message} {location}")]
    Read {
        message: String,
        location: ErrorLocation,
    },

    #[error("IO Error: {message} {location}")]
    Io {
        message: String,
        location: ErrorLocation,
    },

    #[error("Auth Error: {message} {location}")]
    Auth {
        message: String,
        location: ErrorLocation,
    },

    #[error("Protobuf Decode Error: {message} {location}")]
    ProtobufDecode {
        message: String,
        location: ErrorLocation,
    },

    #[error("Protobuf Encode Error: {message} {location}")]
    ProtobufEncode {
        message: String,
        location: ErrorLocation,
    },
}

impl From<IoError> for IpcError {
    #[track_caller]
    fn from(error: IoError) -> Self {
        IpcError::Io {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<prost::DecodeError> for IpcError {
    #[track_caller]
    fn from(error: prost::DecodeError) -> Self {
        IpcError::ProtobufDecode {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<prost::EncodeError> for IpcError {
    #[track_caller]
    fn from(error: prost::EncodeError) -> Self {
        IpcError::ProtobufEncode {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}
