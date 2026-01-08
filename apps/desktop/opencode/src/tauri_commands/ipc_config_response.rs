use crate::ipc_config::IpcConfig;
use log::info;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct IpcConfigResponse {
    pub port: u16,
    pub auth_token: String,
}

#[tauri::command]
pub fn get_ipc_config(config: State<'_, IpcConfig>) -> IpcConfigResponse {
    info!("Blazor requested IPC config");

    IpcConfigResponse {
        port: config.port(),
        auth_token: config.auth_token().to_string(),
    }
}
