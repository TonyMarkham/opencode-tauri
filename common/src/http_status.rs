//! HTTP status code utilities for error handling and retry logic.

/// HTTP status code for error categorization.
///
/// Stored directly rather than parsed from error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatusCode(pub u16);

impl HttpStatusCode {
    /// 4xx client errors (not retryable).
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.0)
    }

    /// 5xx server errors (potentially retryable).
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.0)
    }

    /// Specific codes that indicate transient failures.
    pub fn is_retryable(&self) -> bool {
        matches!(self.0, 502 | 503 | 504 | 429)
    }
}

impl From<u16> for HttpStatusCode {
    fn from(code: u16) -> Self {
        HttpStatusCode(code)
    }
}

impl std::fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
