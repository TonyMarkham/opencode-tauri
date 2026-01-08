//! Config state management using actor pattern.
//!
//! This module provides thread-safe config management for the IPC server.
//! It parallels the design of `IpcState` but manages app configuration instead of server state.
//!
//! # Architecture
//!
//! Uses the same actor pattern as `IpcState`:
//! - Commands sent via mpsc channel
//! - Dedicated task processes commands sequentially
//! - Reads use Arc<RwLock<T>> for concurrent access
//!
//! # Why Separate from IpcState?
//!
//! - Config is loaded once at startup (different lifecycle than server connection)
//! - Config paths come from Tauri (not runtime-discovered)
//! - Config needs validation before updates

use crate::config::{AppConfig, ModelsConfig};
use crate::error::ipc::IpcError;

use common::ErrorLocation;

use std::panic::Location;
use std::path::PathBuf;
use std::sync::Arc;

use log::{error, info, warn};
use tokio::sync::{Mutex, RwLock, mpsc};

/// Commands that mutate config state.
#[derive(Debug, Clone)]
pub enum ConfigCommand {
    /// Update app config (validates, updates memory, saves to disk)
    UpdateAppConfig(AppConfig),
}

/// Config state manager for IPC server.
///
/// Manages app and models configuration with actor pattern for thread-safety.
///
/// # Thread Safety
///
/// This type is `Clone` and can be shared across threads/tasks.
#[derive(Clone)]
pub struct ConfigState {
    /// Channel to send config mutation commands
    command_tx: Arc<Mutex<Option<mpsc::Sender<ConfigCommand>>>>,

    /// Shared read-only access to app config
    app_config: Arc<RwLock<AppConfig>>,

    /// Shared read-only access to models config
    models_config: Arc<RwLock<ModelsConfig>>,

    /// Config directory path (for saving)
    config_dir: Arc<PathBuf>,

    /// Track if actor initialized
    actor_init: Arc<Mutex<bool>>,
}

impl ConfigState {
    /// Create new config state.
    ///
    /// # Arguments
    ///
    /// * `config_dir` - Directory for config.json (from Tauri `app_config_dir()`)
    /// * `app_config` - Initial app config (loaded at startup)
    /// * `models_config` - Initial models config (loaded at startup)
    pub fn new(config_dir: PathBuf, app_config: AppConfig, models_config: ModelsConfig) -> Self {
        Self {
            command_tx: Arc::new(Mutex::new(None)),
            app_config: Arc::new(RwLock::new(app_config)),
            models_config: Arc::new(RwLock::new(models_config)),
            config_dir: Arc::new(config_dir),
            actor_init: Arc::new(Mutex::new(false)),
        }
    }

    /// Send config update command.
    ///
    /// Spawns actor on first call (lazy initialization).
    pub async fn update(&self, cmd: ConfigCommand) -> Result<(), IpcError> {
        self.ensure_actor().await;

        let tx_guard = self.command_tx.lock().await;
        let tx = tx_guard.as_ref().ok_or_else(|| IpcError::Io {
            message: "Config actor not initialized".to_string(),
            location: ErrorLocation::from(Location::caller()),
        })?;

        tx.send(cmd).await.map_err(|e| IpcError::Io {
            message: format!("Config actor died: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })
    }

    /// Get current app config (read-only).
    pub async fn get_app_config(&self) -> AppConfig {
        self.app_config.read().await.clone()
    }

    /// Get current models config (read-only).
    pub async fn get_models_config(&self) -> ModelsConfig {
        self.models_config.read().await.clone()
    }

    /// Ensure actor is spawned (lazy init).
    async fn ensure_actor(&self) {
        let mut init_guard = self.actor_init.lock().await;
        if !*init_guard {
            let (tx, rx) = mpsc::channel(100);
            let app_config_clone = Arc::clone(&self.app_config);
            let models_config_clone = Arc::clone(&self.models_config);
            let config_dir_clone = Arc::clone(&self.config_dir);

            // Store tx BEFORE spawning
            let mut tx_guard = self.command_tx.lock().await;
            *tx_guard = Some(tx);
            drop(tx_guard);

            tokio::spawn(config_actor(
                rx,
                app_config_clone,
                models_config_clone,
                config_dir_clone,
            ));

            *init_guard = true;
            info!("Config state actor spawned");
        }
    }
}

/// Config actor task.
///
/// Processes config update commands sequentially.
async fn config_actor(
    mut command_rx: mpsc::Receiver<ConfigCommand>,
    app_config: Arc<RwLock<AppConfig>>,
    _models_config: Arc<RwLock<ModelsConfig>>, // Future: UpdateModelsConfig command
    config_dir: Arc<PathBuf>,
) {
    info!("Config state actor started");

    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            ConfigCommand::UpdateAppConfig(new_config) => {
                // Validate first (before any changes)
                if let Err(e) = new_config.validate() {
                    error!("Config validation failed: {}", e);
                    continue;
                }

                // Update memory first (can't fail)
                {
                    let mut app_config_write = app_config.write().await;
                    *app_config_write = new_config.clone();
                }
                info!("App config updated in memory");

                // Then persist (if this fails, memory still updated)
                match new_config.save(&config_dir) {
                    Ok(_) => info!("App config saved to disk"),
                    Err(e) => error!("App config saved to memory but disk write failed: {}", e),
                }
            }
        }
    }

    warn!("Config state actor stopped - this should not happen during normal operation");
}
