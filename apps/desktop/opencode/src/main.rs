// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use opencode::error::OpencodeError;
use opencode::ipc_config::IpcConfig;
use opencode::logger::initialize as LoggerInitialize;
use opencode::state::AppState;
use opencode::tauri_commands;

use client_core::ipc::{ConfigState, start_ipc_server};

use common::ErrorLocation;

use std::fs::create_dir_all;
use std::panic::Location;

use log::{info, warn};
use tauri::Manager;
use uuid::Uuid;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tauri_commands::ipc_config_response::get_ipc_config,
        ])
        .setup(|app| {
            // Get app data directory for logs
            let log_dir = app
                .path()
                .app_log_dir()
                .map_err(|e| OpencodeError::Opencode {
                    message: format!("Failed to get log directory: {e}"),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            // Ensure log directory exists
            create_dir_all(&log_dir).map_err(|e| OpencodeError::Opencode {
                message: format!("Failed to create log directory: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

            // Initialize logger FIRST
            LoggerInitialize(&log_dir)?;

            info!("OpenCode Tauri application starting");
            info!("Log directory: {}", log_dir.display());

            // Get platform-specific paths
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|e| OpencodeError::Opencode {
                    message: format!("Failed to get config directory: {e}"),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            let resource_dir = app
                .path()
                .resource_dir()
                .map_err(|e| OpencodeError::Opencode {
                    message: format!("Failed to get resource directory: {e}"),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            info!("Config directory: {}", config_dir.display());
            info!("Resource directory: {}", resource_dir.display());

            // Load configs (never crash - use defaults on error)
            let app_config =
                client_core::config::AppConfig::load(&config_dir).unwrap_or_else(|e| {
                    warn!("Failed to load config.json, using defaults: {}", e);
                    client_core::config::AppConfig::default()
                });

            let models_config = client_core::config::ModelsConfig::load(&resource_dir)
                .unwrap_or_else(|e| {
                    warn!("Failed to load models.toml, using defaults: {}", e);
                    client_core::config::ModelsConfig::default()
                });

            info!(
                "Config loaded: auto_start={}, font_size={:?}, default_model={}, providers={}",
                app_config.server.auto_start,
                app_config.ui.font_size,
                models_config.models.default_model,
                models_config.providers.len()
            );

            // Create config state
            let config_state = ConfigState::new(config_dir.clone(), app_config, models_config);

            // Initialize AppState AFTER Tauri runtime is running
            app.manage(AppState::default());

            // Start IPC WebSocket server
            let ipc_port = 19876;
            let auth_token = Uuid::new_v4().to_string();

            info!("Starting IPC server on port {ipc_port}");
            info!("IPC auth token: {auth_token}");

            let token_clone = auth_token.clone();

            // Start IPC server and verify it binds successfully
            let config_state_clone = config_state.clone(); // 🆕 ADD THIS LINE
            let rt = tauri::async_runtime::handle();
            let _ipc_handle = rt
                .block_on(async {
                    start_ipc_server(ipc_port, Some(token_clone), config_state_clone).await // 🆕 ADD config_state_clone
                })
                .map_err(|e| OpencodeError::Opencode {
                    message: format!("Failed to start IPC server: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            info!("IPC server started successfully");

            // Store IPC config for Blazor to retrieve
            app.manage(IpcConfig::new(ipc_port, auth_token));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
