//! Error types for auth sync operations.
//!
//! Key design decisions:
//! - HTTP status codes stored directly (not parsed from strings)
//! - `is_retryable()` uses status codes, not message content
//! - All errors include ErrorLocation for debugging
//! - `#[track_caller]` for automatic location capture

use common::{ErrorLocation, HttpStatusCode};
use std::panic::Location;
use thiserror::Error as ThisError;

/// Errors that can occur during auth sync operations.
#[derive(Debug, ThisError)]
pub enum AuthSyncError {
    #[error("Environment load failed: {message} {location}")]
    EnvLoad {
        message: String,
        location: ErrorLocation,
    },

    #[error("Provider sync failed for '{provider}': HTTP {status_code} - {message} {location}")]
    ProviderSync {
        provider: String,
        message: String,
        status_code: HttpStatusCode,
        location: ErrorLocation,
    },

    #[error("Network error for '{provider}': {message} {location}")]
    Network {
        provider: String,
        message: String,
        is_timeout: bool,
        is_connection: bool,
        location: ErrorLocation,
    },

    #[error("Auth sync cancelled {location}")]
    Cancelled { location: ErrorLocation },

    #[error("No OpenCode server connected {location}")]
    NoServer { location: ErrorLocation },

    #[error("OAuth check failed for '{provider}': {message} {location}")]
    OAuthCheck {
        provider: String,
        message: String,
        location: ErrorLocation,
    },

    #[error("Auth path detection failed: {message} {location}")]
    AuthPathDetection {
        message: String,
        location: ErrorLocation,
    },

    #[error("Key validation failed for '{provider}': {reason} {location}")]
    KeyValidation {
        provider: String,
        reason: KeyValidationFailure,
        location: ErrorLocation,
    },

    #[error("Operation timeout after {timeout_secs}s {location}")]
    GlobalTimeout {
        timeout_secs: u64,
        location: ErrorLocation,
    },
}

/// Specific reasons for key validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValidationFailure {
    Empty,
    TooShort { min: usize, actual: usize },
    TooLong { max: usize, actual: usize },
    InvalidPrefix { expected: &'static str, actual: String },
    PlaceholderDetected { pattern: &'static str },
    InvalidCharacters,
}

impl std::fmt::Display for KeyValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "key is empty"),
            Self::TooShort { min, actual } => {
                write!(f, "key too short ({} chars, minimum {})", actual, min)
            }
            Self::TooLong { max, actual } => {
                write!(f, "key too long ({} chars, maximum {})", actual, max)
            }
            Self::InvalidPrefix { expected, actual } => {
                write!(f, "expected prefix '{}', got '{}'", expected, actual)
            }
            Self::PlaceholderDetected { pattern } => {
                write!(f, "detected placeholder pattern '{}'", pattern)
            }
            Self::InvalidCharacters => write!(f, "contains invalid characters"),
        }
    }
}

impl AuthSyncError {
    #[track_caller]
    pub fn cancelled() -> Self {
        AuthSyncError::Cancelled {
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub fn no_server() -> Self {
        AuthSyncError::NoServer {
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub fn env_load(message: impl Into<String>) -> Self {
        AuthSyncError::EnvLoad {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub fn global_timeout(timeout_secs: u64) -> Self {
        AuthSyncError::GlobalTimeout {
            timeout_secs,
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub fn auth_path_detection(message: impl Into<String>) -> Self {
        AuthSyncError::AuthPathDetection {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub fn key_validation(provider: impl Into<String>, reason: KeyValidationFailure) -> Self {
        AuthSyncError::KeyValidation {
            provider: provider.into(),
            reason,
            location: ErrorLocation::from(Location::caller()),
        }
    }

    /// Create from reqwest error with proper categorization.
    #[track_caller]
    pub fn from_reqwest(provider: impl Into<String>, error: &reqwest::Error) -> Self {
        let provider = provider.into();

        // Check for specific error types BEFORE converting to string
        let is_timeout = error.is_timeout();
        let is_connect = error.is_connect();

        if is_timeout || is_connect {
            return AuthSyncError::Network {
                provider,
                message: error.to_string(),
                is_timeout,
                is_connection: is_connect,
                location: ErrorLocation::from(Location::caller()),
            };
        }

        // Check for HTTP status in the error
        if let Some(status) = error.status() {
            return AuthSyncError::ProviderSync {
                provider,
                message: error.to_string(),
                status_code: HttpStatusCode(status.as_u16()),  // No longer wrapped in Some()
                location: ErrorLocation::from(Location::caller()),
            };
        }

        // Generic network error (no status available)
        AuthSyncError::Network {
            provider,
            message: error.to_string(),
            is_timeout: false,
            is_connection: false,
            location: ErrorLocation::from(Location::caller()),
        }
    }

    /// Create from HTTP response with explicit status code.
    #[track_caller]
    pub fn from_http_response(
        provider: impl Into<String>,
        status_code: u16,
        body: impl Into<String>,
    ) -> Self {
        AuthSyncError::ProviderSync {
            provider: provider.into(),
            message: body.into(),
            status_code: HttpStatusCode(status_code),  // No longer wrapped in Some()
            location: ErrorLocation::from(Location::caller()),
        }
    }

    /// Check if this error is retryable based on error category, NOT string content.
    pub fn is_retryable(&self) -> bool {
        match self {
            // Network errors: timeouts and connection failures are retryable
            AuthSyncError::Network { is_timeout, is_connection, .. } => {
                *is_timeout || *is_connection
            }

            // HTTP errors: check status code directly (no longer Option)
            AuthSyncError::ProviderSync { status_code, .. } => {
                status_code.is_retryable()
            }

            // These are never retryable
            AuthSyncError::Cancelled { .. } => false,
            AuthSyncError::NoServer { .. } => false,
            AuthSyncError::EnvLoad { .. } => false,
            AuthSyncError::OAuthCheck { .. } => false,
            AuthSyncError::AuthPathDetection { .. } => false,
            AuthSyncError::KeyValidation { .. } => false,
            AuthSyncError::GlobalTimeout { .. } => false,
        }
    }

    /// Get error category for metrics.
    pub fn error_category(&self) -> &'static str {
        match self {
            AuthSyncError::EnvLoad { .. } => "env_load",
            AuthSyncError::ProviderSync { status_code, .. } if status_code.is_client_error() => "client_error",
            AuthSyncError::ProviderSync { status_code, .. } if status_code.is_server_error() => "server_error",
            AuthSyncError::ProviderSync { .. } => "provider_sync",
            AuthSyncError::Network { is_timeout: true, .. } => "timeout",
            AuthSyncError::Network { is_connection: true, .. } => "connection",
            AuthSyncError::Network { .. } => "network",
            AuthSyncError::Cancelled { .. } => "cancelled",
            AuthSyncError::NoServer { .. } => "no_server",
            AuthSyncError::OAuthCheck { .. } => "oauth_check",
            AuthSyncError::AuthPathDetection { .. } => "path_detection",
            AuthSyncError::KeyValidation { .. } => "validation",
            AuthSyncError::GlobalTimeout { .. } => "global_timeout",
        }
    }

    /// Get the provider name if applicable.
    pub fn provider(&self) -> Option<&str> {
        match self {
            AuthSyncError::ProviderSync { provider, .. } => Some(provider),
            AuthSyncError::Network { provider, .. } => Some(provider),
            AuthSyncError::OAuthCheck { provider, .. } => Some(provider),
            AuthSyncError::KeyValidation { provider, .. } => Some(provider),
            _ => None,
        }
    }

    /// Get HTTP status code if applicable.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            AuthSyncError::ProviderSync { status_code, .. } => Some(status_code.0),
            _ => None,
        }
    }
}