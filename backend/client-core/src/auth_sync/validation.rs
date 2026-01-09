//! API key format validation with provider-specific rules.
//!
//! Validates keys BEFORE sending to server to fail fast on obviously
//! invalid values.

use crate::ProviderConfig;
use crate::error::{AuthSyncError, KeyValidationFailure};
use common::RedactedApiKey;

/// Validation result for an API key.
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(KeyValidationFailure),
}

/// Provider-specific validation rules.
///
/// Loaded from config where possible, with hardcoded fallbacks
/// only for well-known providers where format is stable.
pub struct KeyValidator {
    /// Provider name for error messages.
    provider: String,
    /// Expected prefix (e.g., "sk-" for OpenAI).
    expected_prefix: Option<&'static str>,
    /// Minimum key length.
    min_length: usize,
    /// Maximum key length.
    max_length: usize,
}

impl KeyValidator {
    /// Create validator from provider config.
    ///
    /// Uses well-known rules for recognized providers,
    /// generic rules for unknown providers.
    pub fn from_config(config: &ProviderConfig) -> Self {
        // Well-known provider formats (these are stable, documented APIs)
        match config.name.as_str() {
            "openai" => Self {
                provider: config.name.clone(),
                expected_prefix: Some("sk-"),
                min_length: 20,  // Shortest observed OpenAI key
                max_length: 200, // Allow for project keys which are longer
            },
            "anthropic" => Self {
                provider: config.name.clone(),
                expected_prefix: Some("sk-ant-"),
                min_length: 40,
                max_length: 200,
            },
            "google" | "google_generativeai" => Self {
                provider: config.name.clone(),
                expected_prefix: Some("AI"),  // Google keys start with AI
                min_length: 30,
                max_length: 100,
            },
            "mistral" => Self {
                provider: config.name.clone(),
                expected_prefix: None,  // Mistral uses UUIDs
                min_length: 32,
                max_length: 64,
            },
            "cohere" => Self {
                provider: config.name.clone(),
                expected_prefix: None,
                min_length: 30,
                max_length: 100,
            },
            // Unknown provider: use permissive defaults
            _ => Self {
                provider: config.name.clone(),
                expected_prefix: None,
                min_length: 10,   // Minimum reasonable key length
                max_length: 500,  // Allow long keys
            },
        }
    }

    /// Validate a key value.
    ///
    /// Returns `ValidationResult::Valid` if the key passes all checks,
    /// or `ValidationResult::Invalid` with the specific failure reason.
    pub fn validate(&self, key: &str) -> ValidationResult {
        let trimmed = key.trim();

        // Check empty
        if trimmed.is_empty() {
            return ValidationResult::Invalid(KeyValidationFailure::Empty);
        }

        // Check length
        if trimmed.len() < self.min_length {
            return ValidationResult::Invalid(KeyValidationFailure::TooShort {
                min: self.min_length,
                actual: trimmed.len(),
            });
        }

        if trimmed.len() > self.max_length {
            return ValidationResult::Invalid(KeyValidationFailure::TooLong {
                max: self.max_length,
                actual: trimmed.len(),
            });
        }

        // Check prefix (if required)
        if let Some(expected) = self.expected_prefix {
            if !trimmed.starts_with(expected) {
                let actual_prefix: String = trimmed.chars().take(expected.len()).collect();
                return ValidationResult::Invalid(KeyValidationFailure::InvalidPrefix {
                    expected,
                    actual: actual_prefix,
                });
            }
        }

        // Check for placeholder patterns
        if let Some(pattern) = detect_placeholder(trimmed) {
            return ValidationResult::Invalid(KeyValidationFailure::PlaceholderDetected { pattern });
        }

        // Check for invalid characters (keys should be alphanumeric + limited symbols)
        if !is_valid_key_chars(trimmed) {
            return ValidationResult::Invalid(KeyValidationFailure::InvalidCharacters);
        }

        ValidationResult::Valid
    }

    /// Validate and wrap in RedactedApiKey if valid.
    ///
    /// Returns the key wrapped in RedactedApiKey, or an error.
    #[track_caller]
    pub fn validate_and_wrap(&self, key: String) -> Result<RedactedApiKey, AuthSyncError> {
        match self.validate(&key) {
            ValidationResult::Valid => Ok(RedactedApiKey::new(key)),
            ValidationResult::Invalid(reason) => {
                Err(AuthSyncError::key_validation(&self.provider, reason))
            }
        }
    }
}

/// Detect common placeholder patterns.
///
/// Returns the matched pattern name if detected.
fn detect_placeholder(key: &str) -> Option<&'static str> {
    let lower = key.to_lowercase();

    static PATTERNS: &[(&str, &str)] = &[
        ("...", "ellipsis"),
        ("your-api-key", "your-api-key"),
        ("your_api_key", "your_api_key"),
        ("insert", "INSERT"),
        ("<your", "<your...>"),
        ("xxx", "xxx"),
        ("placeholder", "placeholder"),
        ("example", "example"),
        ("test-key", "test-key"),
        ("dummy", "dummy"),
        ("fake", "fake"),
        ("replace", "replace"),
        ("put-your", "put-your"),
        ("add-your", "add-your"),
        ("enter-your", "enter-your"),
    ];

    for (pattern, name) in PATTERNS {
        if lower.contains(pattern) {
            return Some(name);
        }
    }

    // Check for repeated characters (e.g., "xxxxxxxxxx")
    if key.len() >= 10 {
        let first_char = key.chars().next().unwrap();
        if key.chars().all(|c| c == first_char) {
            return Some("repeated_char");
        }
    }

    None
}

/// Check if key contains only valid characters.
///
/// Valid: alphanumeric, hyphen, underscore, period, colon
fn is_valid_key_chars(key: &str) -> bool {
    key.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':')
    })
}