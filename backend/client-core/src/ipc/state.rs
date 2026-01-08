//! IPC state management using actor pattern.
//!
//! This module provides thread-safe state management for the IPC server.
//! It tracks:
//! - Current OpenCode server connection (PID, port, base_url, owned)
//!
//! # Architecture
//!
//! Uses an actor pattern to ensure all state mutations are serialized:
//! - Commands are sent via an mpsc channel
//! - A dedicated task processes commands sequentially
//! - Reads use Arc<RwLock<T>> for lock-free concurrent access
//!
//! # Why Actor Pattern?
//!
//! - **Race-free:** All mutations are serialized by design
//! - **Fast reads:** RwLock allows concurrent reads without blocking on writes
//! - **Simple:** No need to reason about lock ordering or deadlocks

use crate::error::ipc::IpcError;
use crate::opencode_client::OpencodeClient;
use crate::proto::IpcServerInfo;

use common::ErrorLocation;

use std::panic::Location;
use std::sync::Arc;

use log::{info, warn};
use tokio::sync::{Mutex, RwLock, mpsc};

/// Commands that mutate IPC state.
///
/// All state mutations go through the state actor via these commands.
/// This ensures serialized access and prevents race conditions.
#[derive(Debug, Clone)]
pub enum StateCommand {
    /// Set the current server info (from discovery or spawn)
    SetServer(IpcServerInfo),

    /// Clear the current server info (after stop)
    ClearServer,
}

/// IPC state manager.
///
/// Uses an actor pattern to ensure all state mutations are serialized.
/// Commands send StateCommand messages which are processed sequentially
/// by a dedicated task.
///
/// Reads are lock-free using Arc<RwLock<T>> for optimal performance.
///
/// # Thread Safety
///
/// This type is `Clone` and can be shared across threads/tasks. All clones
/// share the same underlying state.
#[derive(Clone)]
pub struct IpcState {
    /// Channel to send state mutation commands to the actor
    command_tx: Arc<Mutex<Option<mpsc::Sender<StateCommand>>>>,

    /// Shared read-only access to server info
    server: Arc<RwLock<Option<IpcServerInfo>>>,

    /// Track if actor has been initialized
    actor_init: Arc<Mutex<bool>>,

    /// Shared read-only access to OpenCode HTTP client
    opencode_client: Arc<RwLock<Option<OpencodeClient>>>,
}

impl IpcState {
    /// Create a new IPC state manager.
    ///
    /// The actor will be lazily spawned on first use within an async context.
    pub fn new() -> Self {
        Self {
            command_tx: Arc::new(Mutex::new(None)),
            server: Arc::new(RwLock::new(None)),
            actor_init: Arc::new(Mutex::new(false)),
            opencode_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Send a state update command.
    ///
    /// This will spawn the actor on first call (lazy initialization).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::Io`] if the state actor has died (should never happen).
    pub async fn update(&self, cmd: StateCommand) -> Result<(), IpcError> {
        self.ensure_actor().await;

        let tx_guard = self.command_tx.lock().await;
        let tx = tx_guard.as_ref().ok_or_else(|| IpcError::Io {
            message: "State actor not initialized".to_string(),
            location: ErrorLocation::from(Location::caller()),
        })?;

        tx.send(cmd).await.map_err(|e| IpcError::Io {
            message: format!("State actor died: {}", e),
            location: ErrorLocation::from(Location::caller()),
        })
    }

    /// Get current server info (read-only).
    ///
    /// This is a lock-free read using RwLock, so it's fast and won't
    /// block on state mutations.
    ///
    /// # Returns
    ///
    /// Returns `Some(IpcServerInfo)` if a server is connected, or `None` if not.
    pub async fn get_server(&self) -> Option<IpcServerInfo> {
        self.server.read().await.clone()
    }

    /// Get current OpenCode client (read-only).
    ///
    /// Returns `Some(OpencodeClient)` if connected to a server, or `None` if not.
    pub async fn get_opencode_client(&self) -> Option<OpencodeClient> {
        self.opencode_client.read().await.clone()
    }

    /// Ensure actor is spawned (called lazily from async context).
    ///
    /// This is an internal implementation detail. The actor is spawned
    /// on first call to `update()`.
    async fn ensure_actor(&self) {
        let mut init_guard = self.actor_init.lock().await;
        if !*init_guard {
            let (tx, rx) = mpsc::channel(100);
            let server_clone = Arc::clone(&self.server);
            let client_clone = Arc::clone(&self.opencode_client);

            // Store tx BEFORE spawning to avoid race
            let mut tx_guard = self.command_tx.lock().await;
            *tx_guard = Some(tx);
            drop(tx_guard); // Release before spawn

            tokio::spawn(state_actor(rx, server_clone, client_clone));
            *init_guard = true;
            info!("IPC state actor spawned");
        }
    }
}

impl Default for IpcState {
    fn default() -> Self {
        Self::new()
    }
}

/// The state actor task.
///
/// Owns the mutable state and processes commands sequentially.
/// This ensures that all state mutations are serialized and
/// prevents race conditions between concurrent operations.
///
/// This function runs in a dedicated tokio task and processes commands
/// until the channel is closed (which happens when all IpcState handles are dropped).
async fn state_actor(
    mut command_rx: mpsc::Receiver<StateCommand>,
    server: Arc<RwLock<Option<IpcServerInfo>>>,
    opencode_client: Arc<RwLock<Option<OpencodeClient>>>,
) {
    info!("IPC state actor started");

    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            StateCommand::SetServer(new_server) => {
                let mut server_write = server.write().await;

                if let Some(ref existing) = *server_write {
                    warn!(
                        "Replacing existing server (PID {}, port {}) with new server (PID {}, port {})",
                        existing.pid, existing.port, new_server.pid, new_server.port
                    );
                } else {
                    info!(
                        "Setting server state: PID={}, port={}, owned={}",
                        new_server.pid, new_server.port, new_server.owned
                    );
                }

                *server_write = Some(new_server.clone());

                // Create OpencodeClient
                match OpencodeClient::new(&new_server.base_url) {
                    Ok(client) => {
                        let mut client_write = opencode_client.write().await;
                        *client_write = Some(client);
                        info!("Created OpencodeClient for {}", new_server.base_url);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create OpencodeClient: {} - session operations will fail",
                            e
                        );
                        let mut client_write = opencode_client.write().await;
                        *client_write = None;
                    }
                }
            }
            StateCommand::ClearServer => {
                let mut server_write = server.write().await;

                if let Some(ref old_server) = *server_write {
                    info!("Clearing server state: PID={}", old_server.pid);
                } else {
                    warn!("Clear server requested but no server was set");
                }

                *server_write = None;

                // Clear OpencodeClient
                let mut client_write = opencode_client.write().await;
                *client_write = None;
                info!("Cleared OpencodeClient");
            }
        }
    }

    warn!("IPC state actor stopped - this should not happen during normal operation");
}
