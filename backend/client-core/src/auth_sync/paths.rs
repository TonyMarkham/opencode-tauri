//! Platform-aware detection of OpenCode data directories.
//!
//! Lookup order:
//! 1. OPENCODE_DATA_DIR environment variable (explicit override)
//! 2. Platform-specific data directory via `dirs` crate
//! 3. Fallback paths for common configurations
//!
//! Returns Result, never silently falls back to wrong path.

use crate::error::AuthSyncError;

use std::env;
use std::path::PathBuf;

use log::{debug, info, warn};

/// OpenCode data directory detection result.
#[derive(Debug, Clone)]
pub struct OpenCodePaths {
    /// Base data directory (e.g., ~/.local/share/opencode on Linux).
    pub data_dir: PathBuf,
    /// Path to auth.json file.
    pub auth_file: PathBuf,
    /// How the path was determined.
    pub source: PathSource,
}

/// How the path was determined (for debugging/logging).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSource {
    /// Set via OPENCODE_DATA_DIR environment variable.
    EnvVar,
    /// Detected via platform-specific XDG/AppData/Library path.
    PlatformDefault,
    /// Linux fallback (~/.local/share/opencode).
    LinuxFallback,
    /// macOS fallback (~/Library/Application Support/opencode).
    MacOSFallback,
    /// Windows fallback (%APPDATA%/opencode).
    WindowsFallback,
}

impl std::fmt::Display for PathSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSource::EnvVar => write!(f, "OPENCODE_DATA_DIR"),
            PathSource::PlatformDefault => write!(f, "platform default"),
            PathSource::LinuxFallback => write!(f, "Linux fallback"),
            PathSource::MacOSFallback => write!(f, "macOS fallback"),
            PathSource::WindowsFallback => write!(f, "Windows fallback"),
        }
    }
}

/// Detect OpenCode data paths.
///
/// # Errors
/// Returns `AuthSyncError::AuthPathDetection` if no valid path can be determined.
///
/// # Platform Behavior
/// - **Linux**: `$XDG_DATA_HOME/opencode` or `~/.local/share/opencode`
/// - **macOS**: `~/Library/Application Support/opencode`
/// - **Windows**: `%APPDATA%/opencode`
pub fn detect_opencode_paths() -> Result<OpenCodePaths, AuthSyncError> {
    // 1. Check environment variable override
    if let Ok(custom_dir) = env::var("OPENCODE_DATA_DIR") {
        let data_dir = PathBuf::from(&custom_dir);
        let auth_file = data_dir.join("auth.json");

        info!("Using OPENCODE_DATA_DIR override: {:?}", data_dir);

        return Ok(OpenCodePaths {
            data_dir,
            auth_file,
            source: PathSource::EnvVar,
        });
    }

    // 2. Try platform-specific detection via dirs crate
    if let Some(data_dir) = dirs::data_local_dir() {
        let opencode_dir = data_dir.join("opencode");
        let auth_file = opencode_dir.join("auth.json");

        debug!("Platform data dir: {:?}", opencode_dir);

        return Ok(OpenCodePaths {
            data_dir: opencode_dir,
            auth_file,
            source: PathSource::PlatformDefault,
        });
    }

    // 3. Platform-specific fallbacks
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = env::var("HOME") {
            let data_dir = PathBuf::from(home).join(".local/share/opencode");
            let auth_file = data_dir.join("auth.json");

            warn!("Using Linux fallback path: {:?}", data_dir);

            return Ok(OpenCodePaths {
                data_dir,
                auth_file,
                source: PathSource::LinuxFallback,
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            let data_dir = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("opencode");
            let auth_file = data_dir.join("auth.json");

            warn!("Using macOS fallback path: {:?}", data_dir);

            return Ok(OpenCodePaths {
                data_dir,
                auth_file,
                source: PathSource::MacOSFallback,
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            let data_dir = PathBuf::from(appdata).join("opencode");
            let auth_file = data_dir.join("auth.json");

            warn!("Using Windows fallback path: {:?}", data_dir);

            return Ok(OpenCodePaths {
                data_dir,
                auth_file,
                source: PathSource::WindowsFallback,
            });
        }
    }

    // No valid path could be determined
    Err(AuthSyncError::auth_path_detection(
        "Cannot determine OpenCode data directory. Set OPENCODE_DATA_DIR environment variable."
    ))
}