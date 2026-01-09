//! Secure API key handling with redacted Debug output.

use crate::{ErrorLocation, RedactError};

use std::fmt;
use std::panic::Location;

use serde::ser::Error;
use zeroize::Zeroize;

/// An API key that never exposes its value in logs or debug output.
#[derive(Clone)]
pub struct RedactedApiKey {
    inner: String,
}

impl RedactedApiKey {
    /// Create a new redacted API key.
    pub fn new(key: String) -> Self {
        Self { inner: key }
    }

    /// Get the actual key value for transmission.
    ///
    /// # Security Note
    /// Only call this when actually sending the key to the server.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the key length (safe to log).
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the key is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Debug for RedactedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RedactedApiKey([REDACTED])")
    }
}

impl fmt::Display for RedactedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED API KEY]")
    }
}

impl Drop for RedactedApiKey {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

// Prevent accidental serialization
impl serde::Serialize for RedactedApiKey {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(S::Error::custom(
            RedactError::Serialization {
                message: String::from("RedactedApiKey cannot be serialized - use as_str() explicitly"),
                location: ErrorLocation::from(Location::caller()),
            }
        ))
    }
}