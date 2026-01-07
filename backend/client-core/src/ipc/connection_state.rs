//! Connection state tracking for authentication.
//!
//! This module provides per-connection state to track whether a client
//! has successfully authenticated with the IPC server.

/// Connection state for auth tracking.
///
/// Tracks whether a connection has been authenticated and what token is expected.
pub(crate) struct ConnectionState {
    authenticated: bool,
    expected_token: String,
}

impl ConnectionState {
    /// Create new connection state with expected token.
    pub(crate) fn new(token: String) -> Self {
        Self {
            authenticated: false,
            expected_token: token,
        }
    }

    /// Validate token and mark as authenticated if correct.
    ///
    /// Returns true if token matches, false otherwise.
    pub(crate) fn validate_token(&mut self, token: &str) -> bool {
        if token == self.expected_token {
            self.authenticated = true;
            true
        } else {
            false
        }
    }

    /// Check if connection is authenticated.
    pub(crate) fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}
