// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use opencode::commands;
use opencode::error::OpencodeError;
use opencode::ipc_config::IpcConfig;
use opencode::logger::initialize as LoggerInitialize;
use opencode::state::AppState;

use client_core::ipc::start_ipc_server;

use common::ErrorLocation;

use std::fs::create_dir_all;
use std::panic::Location;

use log::info;
use tauri::Manager;
use uuid::Uuid;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::server::discover_server,
            commands::server::spawn_server,
            commands::server::check_health,
            commands::server::stop_server,
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

            // Initialize AppState AFTER Tauri runtime is running
            app.manage(AppState::default());

            // Start IPC WebSocket server
            let ipc_port = 19876;
            let auth_token = Uuid::new_v4().to_string();

            info!("Starting IPC server on port {ipc_port}");
            info!("IPC auth token: {auth_token}");

            let token_clone = auth_token.clone();

            // Start IPC server and verify it binds successfully
            let rt = tokio::runtime::Handle::current();
            let _ipc_handle = rt
                .block_on(async { start_ipc_server(ipc_port, Some(token_clone)).await })
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
