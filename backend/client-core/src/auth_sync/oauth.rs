//! OAuth detection for skipping API key sync.
//!
//! Returns `Result<OAuthStatus, Error>` instead of silent `bool` fallback.
//! Caller decides how to handle uncertainty.

use super::paths::detect_opencode_paths;
use crate::error::AuthSyncError;
use log::{debug, info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// OAuth detection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthStatus {
    /// OAuth is configured and valid for this provider.
    Configured,
    /// API key auth is configured (not OAuth).
    ApiKeyConfigured,
    /// WellKnown auth is configured (not OAuth).
    WellKnownConfigured,
    /// No auth configured for this provider.
    NotConfigured,
    /// Could not determine status (file missing, etc.).
    Unknown { reason: String },
}

impl OAuthStatus {
    /// Should we skip API key sync for this status?
    pub fn should_skip_api_key_sync(&self) -> bool {
        matches!(self, OAuthStatus::Configured)
    }

    /// Is this status definitive (not unknown)?
    pub fn is_definitive(&self) -> bool {
        !matches!(self, OAuthStatus::Unknown { .. })
    }
}

/// Auth info from OpenCode's auth.json file.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AuthInfo {
    #[serde(rename = "oauth")]
    OAuth {
        access: String,
        refresh: String,
        expires: f64,
    },
    #[serde(rename = "api")]
    ApiKey { key: String },
    #[serde(rename = "wellknown")]
    WellKnown { key: String, token: String },
}

impl AuthInfo {
    pub fn auth_type(&self) -> &'static str {
        match self {
            AuthInfo::OAuth { .. } => "oauth",
            AuthInfo::ApiKey { .. } => "api",
            AuthInfo::WellKnown { .. } => "wellknown",
        }
    }

    pub fn to_oauth_status(&self) -> OAuthStatus {
        match self {
            AuthInfo::OAuth { .. } => OAuthStatus::Configured,
            AuthInfo::ApiKey { .. } => OAuthStatus::ApiKeyConfigured,
            AuthInfo::WellKnown { .. } => OAuthStatus::WellKnownConfigured,
        }
    }
}

/// Check OAuth status for a provider.
///
/// # Returns
/// - `Ok(OAuthStatus)` - Definitive status or Unknown with reason
/// - `Err(AuthSyncError)` - Only for truly fatal errors (not file-not-found)
///
/// # Design
/// Unlike returning `bool`, this allows the caller to:
/// - Distinguish "no OAuth" from "couldn't check"
/// - Log/metric the reason for unknown status
/// - Decide whether to proceed with API key sync on uncertainty
pub fn check_oauth_status(provider: &str) -> Result<OAuthStatus, AuthSyncError> {
    // Get auth.json path
    let paths = match detect_opencode_paths() {
        Ok(p) => p,
        Err(_) => {
            // Can't determine paths - return Unknown, not error
            return Ok(OAuthStatus::Unknown {
                reason: "Cannot determine OpenCode data directory".to_string(),
            });
        }
    };

    debug!(
        "Checking OAuth status in {:?} (source: {})",
        paths.auth_file, paths.source
    );

    // Check if file exists
    if !paths.auth_file.exists() {
        debug!("auth.json not found at {:?}", paths.auth_file);
        return Ok(OAuthStatus::NotConfigured);
    }

    // Read file
    let content = match fs::read_to_string(&paths.auth_file) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(OAuthStatus::NotConfigured);
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            warn!("Permission denied reading auth.json: {}", e);
            return Ok(OAuthStatus::Unknown {
                reason: format!("Permission denied: {}", e),
            });
        }
        Err(e) => {
            warn!("Failed to read auth.json: {}", e);
            return Ok(OAuthStatus::Unknown {
                reason: format!("Read error: {}", e),
            });
        }
    };

    // Parse as HashMap<provider, AuthInfo>
    let auth_data: HashMap<String, serde_json::Value> = match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to parse auth.json: {}", e);
            return Ok(OAuthStatus::Unknown {
                reason: format!("Parse error: {}", e),
            });
        }
    };

    // Check for this provider
    let provider_auth = match auth_data.get(provider) {
        Some(v) => v,
        None => {
            debug!("No auth entry for provider '{}'", provider);
            return Ok(OAuthStatus::NotConfigured);
        }
    };

    // Parse the provider's auth info
    match serde_json::from_value::<AuthInfo>(provider_auth.clone()) {
        Ok(auth_info) => {
            let status = auth_info.to_oauth_status();
            if status == OAuthStatus::Configured {
                info!(
                    "Provider '{}' has OAuth configured - will skip API key sync",
                    provider
                );
            } else {
                debug!(
                    "Provider '{}' has {} auth configured",
                    provider,
                    auth_info.auth_type()
                );
            }
            Ok(status)
        }
        Err(e) => {
            warn!("Failed to parse auth info for '{}': {}", provider, e);
            Ok(OAuthStatus::Unknown {
                reason: format!("Auth info parse error: {}", e),
            })
        }
    }
}

/// Batch check OAuth status for multiple providers.
///
/// More efficient than calling check_oauth_status repeatedly
/// because it reads auth.json once.
pub fn check_oauth_status_batch(providers: &[&str]) -> HashMap<String, OAuthStatus> {
    let mut results = HashMap::new();

    // Get auth.json path
    let paths = match detect_opencode_paths() {
        Ok(p) => p,
        Err(_) => {
            // Return Unknown for all providers
            for provider in providers {
                results.insert(
                    provider.to_string(),
                    OAuthStatus::Unknown {
                        reason: "Cannot determine OpenCode data directory".to_string(),
                    },
                );
            }
            return results;
        }
    };

    // Read and parse file once
    let auth_data: HashMap<String, serde_json::Value> = match fs::read_to_string(&paths.auth_file)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
    {
        Some(data) => data,
        None => {
            // File missing or invalid - all providers are NotConfigured
            for provider in providers {
                results.insert(provider.to_string(), OAuthStatus::NotConfigured);
            }
            return results;
        }
    };

    // Check each provider
    for provider in providers {
        let status = match auth_data.get(*provider) {
            None => OAuthStatus::NotConfigured,
            Some(value) => match serde_json::from_value::<AuthInfo>(value.clone()) {
                Ok(auth_info) => auth_info.to_oauth_status(),
                Err(_) => OAuthStatus::Unknown {
                    reason: "Parse error".to_string(),
                },
            },
        };
        results.insert(provider.to_string(), status);
    }

    results
}
