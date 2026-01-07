//! IPC server handle type.
//!
//! This module defines the handle returned when starting an IPC server.
//! The handle represents the running server and can be used for lifecycle management.

/// Handle to a running IPC WebSocket server.
///
/// This handle is returned by [`start_ipc_server`](crate::ipc::start_ipc_server) and represents
/// a background task serving WebSocket connections on localhost.
///
/// # Lifecycle
///
/// Currently, dropping this handle does **not** stop the server. The server runs until
/// the process exits. Future versions will implement graceful shutdown via `Drop`.
///
/// # Examples
///
/// ```no_run
/// use client_core::ipc::start_ipc_server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let handle = start_ipc_server(19876).await?;
///     // Server is now running in background
///     // Keep handle alive to keep server running
///     Ok(())
/// }
/// ```
///
/// # Future Enhancements
///
/// - Graceful shutdown on drop
/// - Query server statistics (connection count, message count)
/// - Programmatic port discovery
pub struct IpcServerHandle {}
