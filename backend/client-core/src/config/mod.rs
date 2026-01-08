pub mod models;

pub use models::ModelsConfig;

use crate::error::config::ConfigError;

use common::ErrorLocation;

use std::panic::Location;
use std::path::Path;

use log::{info, warn};
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_VERSION: u32 = 1;

// ============================================
// ENUMS WITH DEFAULTS
// ============================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontSizePreset {
    Small,
    Standard,
    Large,
}

impl Default for FontSizePreset {
    fn default() -> Self {
        FontSizePreset::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatDensity {
    Compact,
    Normal,
    Comfortable,
}

impl Default for ChatDensity {
    fn default() -> Self {
        ChatDensity::Normal
    }
}

// ============================================
// CONFIG STRUCTS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub last_opencode_url: Option<String>,
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    pub directory_override: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            last_opencode_url: None,
            auto_start: default_auto_start(),
            directory_override: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    #[serde(default)]
    pub font_size: FontSizePreset,
    #[serde(default = "default_base_font_points")]
    pub base_font_points: f32,
    #[serde(default)]
    pub chat_density: ChatDensity,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            font_size: FontSizePreset::default(),
            base_font_points: default_base_font_points(),
            chat_density: ChatDensity::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_push_to_talk_key")]
    pub push_to_talk_key: String,
    pub whisper_model_path: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            push_to_talk_key: default_push_to_talk_key(),
            whisper_model_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub ui: UiPreferences,

    #[serde(default)]
    pub audio: AudioConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            server: ServerConfig::default(),
            ui: UiPreferences::default(),
            audio: AudioConfig::default(),
        }
    }
}

// ============================================
// DEFAULT FUNCTIONS
// ============================================

fn default_version() -> u32 {
    CONFIG_VERSION
}
fn default_auto_start() -> bool {
    true
}
fn default_base_font_points() -> f32 {
    14.0
}
fn default_push_to_talk_key() -> String {
    "AltRight".to_string()
}

// ============================================
// IMPLEMENTATION
// ============================================

impl AppConfig {
    /// Load config from {config_dir}/config.json.
    ///
    /// Falls back to defaults on any error (missing file, parse error, validation error).
    ///
    /// # Returns
    ///
    /// Returns `Ok(AppConfig)` if loaded successfully or defaults if file missing.
    /// Returns `Err(ConfigError)` if file exists but is corrupted/invalid.
    pub fn load(config_dir: &Path) -> Result<Self, ConfigError> {
        let config_path = config_dir.join(CONFIG_FILE_NAME);

        if !config_path.exists() {
            info!(
                "Config file not found at {}, using defaults",
                config_path.display()
            );
            return Ok(Self::default());
        }

        // Read file
        let contents = std::fs::read_to_string(&config_path).map_err(|e| {
            warn!("Failed to read config file, using defaults: {}", e);
            ConfigError::ReadError {
                location: ErrorLocation::from(Location::caller()),
                path: config_path.clone(),
                source: e,
            }
        })?;

        // Parse JSON
        let config: AppConfig = serde_json::from_str(&contents).map_err(|e| {
            warn!("Failed to parse config JSON, using defaults: {}", e);
            ConfigError::ParseError {
                location: ErrorLocation::from(Location::caller()),
                path: config_path.clone(),
                reason: e.to_string(),
            }
        })?;

        // Validate
        config.validate()?;

        info!("Config loaded from {}", config_path.display());
        Ok(config)
    }

    /// Save config to {config_dir}/config.json using atomic write.
    ///
    /// Uses temp file + rename for atomicity (no corruption on crash).
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - Directory creation fails
    /// - Serialization fails
    /// - Write fails
    /// - Rename fails
    pub fn save(&self, config_dir: &Path) -> Result<(), ConfigError> {
        // Validate before saving
        self.validate()?;

        // Ensure directory exists
        std::fs::create_dir_all(config_dir).map_err(|e| ConfigError::WriteError {
            location: ErrorLocation::from(Location::caller()),
            path: config_dir.to_path_buf(),
            source: e,
        })?;

        let config_path = config_dir.join(CONFIG_FILE_NAME);
        let temp_path = config_dir.join(format!("{}.tmp", CONFIG_FILE_NAME));

        // Serialize to JSON
        let json = serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializeError {
            location: ErrorLocation::from(Location::caller()),
            reason: e.to_string(),
        })?;

        // Write to temp file
        std::fs::write(&temp_path, json).map_err(|e| ConfigError::WriteError {
            location: ErrorLocation::from(Location::caller()),
            path: temp_path.clone(),
            source: e,
        })?;

        // Atomic rename (POSIX guarantees atomicity)
        std::fs::rename(&temp_path, &config_path).map_err(|e| ConfigError::WriteError {
            location: ErrorLocation::from(Location::caller()),
            path: config_path.clone(),
            source: e,
        })?;

        info!("Config saved to {}", config_path.display());
        Ok(())
    }

    /// Validate config values.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationError`] if any value is invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Version check
        if self.version == 0 || self.version > CONFIG_VERSION {
            return Err(ConfigError::ValidationError {
                location: ErrorLocation::from(Location::caller()),
                reason: format!(
                    "Invalid version: {} (expected 1-{})",
                    self.version, CONFIG_VERSION
                ),
            });
        }

        // Font size bounds
        if self.ui.base_font_points < 8.0 || self.ui.base_font_points > 72.0 {
            return Err(ConfigError::ValidationError {
                location: ErrorLocation::from(Location::caller()),
                reason: format!(
                    "Invalid font size: {} (must be 8.0-72.0)",
                    self.ui.base_font_points
                ),
            });
        }

        // URL validation (if set)
        if let Some(ref url) = self.server.last_opencode_url {
            if url.is_empty() {
                return Err(ConfigError::ValidationError {
                    location: ErrorLocation::from(Location::caller()),
                    reason: "last_opencode_url cannot be empty string".to_string(),
                });
            }

            // Basic URL format check
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ConfigError::ValidationError {
                    location: ErrorLocation::from(Location::caller()),
                    reason: format!("Invalid URL format: {}", url),
                });
            }
        }

        Ok(())
    }
}
