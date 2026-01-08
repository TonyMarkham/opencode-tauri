// Library exports for testing
// The binary (main.rs) imports these as well

pub mod error;
pub mod ipc_config;
pub mod logger;
pub mod state;
pub mod tauri_commands;

#[cfg(test)]
mod tests;
