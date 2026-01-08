use std::path::PathBuf;

use common::ErrorLocation;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config Read Error: {path}: {source} {location}")]
    ReadError {
        location: ErrorLocation,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Config Parse Error: {path}: {reason} {location}")]
    ParseError {
        location: ErrorLocation,
        path: PathBuf,
        reason: String,
    },

    #[error("Config Write Error: {path}: {source} {location}")]
    WriteError {
        location: ErrorLocation,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Config Directory Not Found Error: {path} {location}")]
    DirectoryNotFound {
        location: ErrorLocation,
        path: PathBuf,
    },

    #[error("Config Serialization Error: {reason} {location}")]
    SerializeError {
        location: ErrorLocation,
        reason: String,
    },

    #[error("Config Validation Error: {reason} {location}")]
    ValidationError {
        location: ErrorLocation,
        reason: String,
    },
}
