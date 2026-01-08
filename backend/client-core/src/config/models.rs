use crate::error::config::ConfigError;

use common::ErrorLocation;

use std::collections::HashMap;
use std::panic::Location;
use std::path::Path;

use log::{info, warn};
use serde::{Deserialize, Serialize};

const MODELS_FILE_NAME: &str = "models.toml";

// ============================================
// MODELS CONFIG STRUCTS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuratedModel {
    pub name: String,
    pub provider: String,
    pub model_id: String,
}

impl CuratedModel {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            model_id: model_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub display_name: String,
    pub api_key_env: String,
    pub models_url: String,
    pub auth_type: String,
    #[serde(default)]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub auth_param: Option<String>,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
    pub response_format: ResponseFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    pub models_path: String,
    pub model_id_field: String,
    #[serde(default)]
    pub model_id_strip_prefix: Option<String>,
    pub model_name_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsSection {
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default)]
    pub curated: Vec<CuratedModel>,
}

impl Default for ModelsSection {
    fn default() -> Self {
        Self {
            default_model: default_model(),
            curated: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub models: ModelsSection,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            models: ModelsSection::default(),
        }
    }
}

fn default_model() -> String {
    "openai/gpt-4".to_string()
}

// ============================================
// IMPLEMENTATION
// ============================================

impl ModelsConfig {
    /// Load models.toml from resource directory.
    ///
    /// Tries multiple paths for dev vs production:
    /// 1. {resource_dir}/config/models.toml (production bundle)
    /// 2. {resource_dir}/models.toml (alternative location)
    /// 3. Falls back to default (empty providers)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(ModelsConfig)` - either loaded or default.
    pub fn load(resource_dir: &Path) -> Result<Self, ConfigError> {
        // Try multiple paths (production vs dev)
        let paths = [
            resource_dir.join("config").join(MODELS_FILE_NAME),
            resource_dir.join(MODELS_FILE_NAME),
        ];

        for path in &paths {
            if path.exists() {
                match Self::load_from_path(path) {
                    Ok(config) => {
                        info!("Models config loaded from {}", path.display());
                        return Ok(config);
                    }
                    Err(e) => {
                        warn!("Failed to load models from {}: {}", path.display(), e);
                        // Try next path
                    }
                }
            }
        }

        warn!("No models.toml found in resource dir, using defaults");
        Ok(Self::default())
    }

    /// Load from specific path (internal helper).
    fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
            location: ErrorLocation::from(Location::caller()),
            path: path.to_path_buf(),
            source: e,
        })?;

        let config: ModelsConfig =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
                location: ErrorLocation::from(Location::caller()),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        // Validate providers
        config.validate()?;

        Ok(config)
    }

    /// Validate provider configurations.
    pub fn validate(&self) -> Result<(), ConfigError> {
        for provider in &self.providers {
            if provider.name.is_empty() {
                return Err(ConfigError::ValidationError {
                    location: ErrorLocation::from(Location::caller()),
                    reason: "Provider name cannot be empty".to_string(),
                });
            }

            if provider.models_url.is_empty() {
                return Err(ConfigError::ValidationError {
                    location: ErrorLocation::from(Location::caller()),
                    reason: format!("Provider '{}' missing models_url", provider.name),
                });
            }

            // Validate auth_type
            match provider.auth_type.as_str() {
                "bearer" | "header" | "query_param" => {}
                _ => {
                    return Err(ConfigError::ValidationError {
                        location: ErrorLocation::from(Location::caller()),
                        reason: format!(
                            "Invalid auth_type '{}' for provider '{}'",
                            provider.auth_type, provider.name
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get provider by name.
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Add curated model (avoids duplicates).
    pub fn add_curated_model(&mut self, model: CuratedModel) {
        let exists = self
            .models
            .curated
            .iter()
            .any(|m| m.provider == model.provider && m.model_id == model.model_id);

        if !exists {
            self.models.curated.push(model);
        }
    }

    /// Remove curated model.
    pub fn remove_curated_model(&mut self, provider: &str, model_id: &str) {
        self.models
            .curated
            .retain(|m| !(m.provider == provider && m.model_id == model_id));
    }

    /// Get all curated models.
    pub fn get_curated_models(&self) -> &[CuratedModel] {
        &self.models.curated
    }
}
