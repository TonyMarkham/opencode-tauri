//! API key synchronization from .env to OpenCode server.
//!
//! # Features
//! - Loads .env from cwd or executable directory
//! - Uses ModelsConfig for provider definitions (no hardcoding)
//! - Provider-specific key validation
//! - OAuth detection to skip configured providers
//! - Retry with exponential backoff
//! - Global operation timeout
//! - Secure handling via RedactedApiKey
//!
//! # Security
//! - API keys wrapped in RedactedApiKey (safe Debug impl)
//! - Keys zeroized on drop
//! - Never logged or serialized

pub mod oauth;
pub mod paths;
pub mod validation;

// Re-export key types for convenience
pub use oauth::OAuthStatus;

use crate::config::ModelsConfig;
use crate::error::AuthSyncError;

use common::RedactedApiKey;

use validation::KeyValidator;

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use log::{debug, info, warn};

/// Result of loading API keys from environment.
#[derive(Debug)]
pub struct LoadedKeys {
    /// Valid keys by provider name.
    pub keys: HashMap<String, RedactedApiKey>,
    /// Keys that failed validation (provider -> error).
    pub validation_errors: HashMap<String, AuthSyncError>,
}

impl LoadedKeys {
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn total_found(&self) -> usize {
        self.keys.len() + self.validation_errors.len()
    }
}

/// Result of attempting to load .env file.
#[derive(Debug)]
pub struct EnvLoadResult {
    /// Path to loaded .env file, if found.
    pub path: Option<std::path::PathBuf>,
    /// Whether any .env file was loaded.
    pub loaded: bool,
}

/// Load API keys from .env file and environment using provider config.
///
/// # Arguments
/// - `config`: The loaded models config with provider definitions
///
/// # Returns
/// - `LoadedKeys` with valid keys and validation errors
///
/// # Security
/// - Keys wrapped in RedactedApiKey (never exposed in Debug)
/// - Skips empty and placeholder values
pub fn load_env_api_keys(config: &ModelsConfig) -> LoadedKeys {
    // Try to load .env file (non-fatal if missing)
    let env_result = try_load_dotenv();
    if !env_result.loaded {
        debug!("No .env file found - will check existing environment variables");
    }

    let mut keys = HashMap::new();
    let mut validation_errors = HashMap::new();

    // Use provider config to know exactly which env vars to look for
    for provider in &config.providers {
        if provider.api_key_env.is_empty() {
            debug!("Provider '{}' has no api_key_env configured, skipping", provider.name);
            continue;
        }

        match env::var(&provider.api_key_env) {
            Ok(value) => {
                // Validate using provider-specific rules
                let validator = KeyValidator::from_config(provider);

                match validator.validate_and_wrap(value) {
                    Ok(redacted_key) => {
                        info!(
                              "Found valid API key for provider: {} (from {}, {} chars)",
                              provider.name,
                              provider.api_key_env,
                              redacted_key.len()
                          );
                        keys.insert(provider.name.clone(), redacted_key);
                    }
                    Err(e) => {
                        warn!(
                              "Invalid API key for provider '{}': {}",
                              provider.name, e
                          );
                        validation_errors.insert(provider.name.clone(), e);
                    }
                }
            }
            Err(env::VarError::NotPresent) => {
                debug!(
                      "No {} env var found for provider {}",
                      provider.api_key_env, provider.name
                  );
            }
            Err(env::VarError::NotUnicode(_)) => {
                warn!("Env var {} contains invalid unicode", provider.api_key_env);
                validation_errors.insert(
                    provider.name.clone(),
                    AuthSyncError::env_load(format!(
                        "{} contains invalid unicode",
                        provider.api_key_env
                    )),
                );
            }
        }
    }

    LoadedKeys { keys, validation_errors }
}

/// Attempts to load .env from known locations.
fn try_load_dotenv() -> EnvLoadResult {
    // Try current directory first
    if let Ok(path) = dotenvy::dotenv() {
        info!("Loaded .env from: {:?}", path);
        return EnvLoadResult { path: Some(path), loaded: true };
    }

    // Try executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            if env_path.exists() {
                match dotenvy::from_path(&env_path) {
                    Ok(_) => {
                        info!("Loaded .env from: {:?}", env_path);
                        return EnvLoadResult { path: Some(env_path), loaded: true };
                    }
                    Err(e) => {
                        warn!("Failed to parse .env at {:?}: {}", env_path, e);
                    }
                }
            }
        }
    }

    EnvLoadResult { path: None, loaded: false }
}

/// Configuration for sync operation.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Skip providers with OAuth configured.
    pub skip_oauth_providers: bool,
    /// Overall operation timeout.
    pub timeout: Duration,
    /// Maximum retries per provider.
    pub max_retries: u32,
    /// Initial retry delay.
    pub initial_delay: Duration,
    /// Maximum retry delay.
    pub max_delay: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            skip_oauth_providers: true,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
        }
    }
}
