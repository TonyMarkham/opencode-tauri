// Library exports for testing
// The binary (main.rs) imports these as well

pub mod tauri_commands;
pub mod error;
pub mod ipc_config;
pub mod logger;
pub mod state;

#[cfg(test)]
mod tests;
