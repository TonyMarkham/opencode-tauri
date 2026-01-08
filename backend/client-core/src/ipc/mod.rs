//! IPC (Inter-Process Communication) layer for Blazor-to-Rust communication.
//!
//! This module implements WebSocket-based IPC per ADR-0003. It provides:
//!
//! - WebSocket server (localhost-only)
//! - Binary protobuf protocol (type-safe)
//! - Authentication handshake (security)
//! - Server management handlers (discover, spawn, health, stop)
//!
//! # Architecture
//!
//! Per ADR-0002 (Thin Tauri Layer), all IPC logic lives here in `client-core`,
//! not in the Tauri layer. The Tauri layer only calls [`start_ipc_server`] during
//! app initialization.
//!
//! # Protocol
//!
//! See `proto/ipc.proto` for message definitions. Key message types:
//! - `IpcClientMessage` - Client → Server
//! - `IpcServerMessage` - Server → Client
//! - `IpcAuthHandshake` - Authentication (first message)
//!
//! # Security
//!
//! - Localhost-only binding (`127.0.0.1`)
//! - Non-loopback connections rejected
//! - Authentication token required (generated on server start)

pub mod config_state;
mod connection_state;
mod handle;
mod server;
mod state;

pub use config_state::{ConfigCommand, ConfigState};
pub use handle::IpcServerHandle;
pub use server::start_ipc_server;
pub use state::{IpcState, StateCommand};
