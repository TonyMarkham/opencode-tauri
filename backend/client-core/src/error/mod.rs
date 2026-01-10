pub mod auth_sync;
pub mod config;
pub mod discovery;
pub mod ipc;
pub mod opencode_client;
pub mod spawn;
pub mod ws;
pub use auth_sync::{AuthSyncError, KeyValidationFailure};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Discovery(#[from] discovery::DiscoveryError),

    #[error(transparent)]
    Spawn(#[from] spawn::SpawnError),

    #[error(transparent)]
    Ws(#[from] ws::WsError),

    #[error(transparent)]
    Config(#[from] config::ConfigError),
}
