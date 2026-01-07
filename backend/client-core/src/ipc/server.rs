//! IPC WebSocket server implementation.
//!
//! This module implements a WebSocket server for IPC between the Blazor frontend
//! and the Rust backend. The server:
//!
//! - Listens on localhost only (security)
//! - Uses binary protobuf messages (type safety)
//! - Requires authentication handshake (security)
//! - Handles concurrent connections (scalability)
//!
//! # Architecture
//!
//! Per ADR-0003, this is the canonical IPC mechanism replacing Tauri invoke.
//! Per ADR-0002, this lives in client-core (not Tauri layer).
//!
//! # Security
//!
//! - Binds to `127.0.0.1` only (no network exposure)
//! - Rejects non-loopback connections
//! - Requires auth token in first message (future: Step 6)
//!
//! # Protocol
//!
//! WebSocket with binary protobuf frames. See `proto/ipc.proto` for message definitions.

use crate::discovery::{process, spawn};
use crate::error::ipc::IpcError;
use crate::ipc::connection_state::ConnectionState;
use crate::ipc::handle::IpcServerHandle;
use crate::ipc::state::{IpcState, StateCommand};
use crate::proto::IpcErrorCode::{AuthError, InternalError, InvalidMessage, NotImplemented};
use crate::proto::{
    IpcAuthHandshakeResponse, IpcCheckHealthResponse, IpcClientMessage, IpcDiscoverServerResponse,
    IpcErrorCode, IpcErrorResponse, IpcServerMessage, IpcSpawnServerRequest,
    IpcSpawnServerResponse, IpcStopServerResponse, ipc_client_message, ipc_server_message,
};

use common::ErrorLocation;

use std::net::SocketAddr;
use std::panic::Location;

use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use prost::Message as ProstMessage;
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn as TokioSpawn;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

/// Starts the IPC WebSocket server on the specified port.
///
/// This function binds to `127.0.0.1:<ipc_port>` and spawns a background task
/// to accept WebSocket connections. The server echoes all messages (text or binary)
/// back to clients.
///
/// # Arguments
///
/// * `ipc_port` - Port to bind on localhost (e.g., 19876)
///
/// # Returns
///
/// Returns [`IpcServerHandle`] on success, representing the running server.
///
/// # Errors
///
/// Returns [`IpcError::Io`] if:
/// - Port is already in use
/// - Insufficient permissions to bind port
/// - Network interface unavailable
///
/// # Security
///
/// - Binds to `127.0.0.1` only (localhost)
/// - Individual connections reject non-loopback clients
/// - Future: Requires auth token in first message (Step 6)
///
/// # Panics
///
/// Does not panic under normal conditions. Background tasks may panic on:
/// - Out of memory (cannot spawn new connections)
/// - System resource exhaustion
pub async fn start_ipc_server(
    ipc_port: u16,
    auth_token: Option<String>,
) -> Result<IpcServerHandle, IpcError> {
    // Generate token if not provided
    let auth_token = auth_token.unwrap_or_else(|| {
        let token = Uuid::new_v4().to_string();
        info!("Generated IPC auth token: {}", token);
        token
    });

    let address = format!("127.0.0.1:{ipc_port}");
    let listener = TcpListener::bind(&address).await?;

    info!("IPC server listening on {}", address);

    TokioSpawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            info!("Client connecting from {}", addr);
            let token_clone = auth_token.clone();
            TokioSpawn(handle_connection(stream, addr, token_clone));
        }
    });

    Ok(IpcServerHandle {})
}

/// Handles a single WebSocket connection.
///
/// This function:
/// 1. Performs WebSocket handshake
/// 2. **Rejects non-localhost connections** (security)
/// 3. **Requires auth handshake as first message** (security)
/// 4. Validates auth token
/// 5. Processes subsequent messages (currently echoes them)
///
/// # Arguments
///
/// * `stream` - TCP stream from accepted connection
/// * `addr` - Client address (for security checks)
/// * `auth_token` - Expected auth token
///
/// # Returns
///
/// Returns `Ok(())` on clean disconnect, or [`IpcError`] on failure.
///
/// # Errors
///
/// - [`IpcError::Handshake`] - WebSocket upgrade failed
/// - [`IpcError::Auth`] - Authentication failed (wrong token, wrong first message, non-localhost)
/// - [`IpcError::ProtobufDecode`] - Failed to decode protobuf message
/// - [`IpcError::ProtobufEncode`] - Failed to encode protobuf response
/// - [`IpcError::Send`] - Failed to send message to client
/// - [`IpcError::Read`] - Failed to read message from client
///
/// # Protocol
///
/// 1. **First message MUST be** `IpcAuthHandshake` with valid token
/// 2. Server responds with `IpcAuthHandshakeResponse` (success or failure)
/// 3. If auth fails, connection closes immediately
/// 4. If auth succeeds, subsequent messages are processed (currently echoed)
///
/// # Security
///
/// - Non-loopback connections are rejected immediately
/// - First message must be auth handshake (not any other message type)
/// - Token must match server's expected token
/// - All failures close the connection (fail-closed security model)
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    auth_token: String,
) -> Result<(), IpcError> {
    // SECURITY: Reject non-loopback connections
    if !addr.ip().is_loopback() {
        warn!("Rejected non-loopback connection from {}", addr);
        return Ok(()); // Silent rejection (don't give attackers info)
    }

    let ws_stream = match accept_async(stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return Err(IpcError::Handshake {
                message: format!("WebSocket handshake failed: {}", e),
                location: ErrorLocation::from(Location::caller()),
            });
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let mut state = ConnectionState::new(auth_token);

    // SECURITY: First message MUST be auth handshake
    if let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                // Decode protobuf message
                let client_msg = IpcClientMessage::decode(&data[..])?;

                // Check if it's auth handshake
                match client_msg.payload {
                    Some(ipc_client_message::Payload::AuthHandshake(auth)) => {
                        // Validate token
                        if state.validate_token(&auth.token) {
                            info!("Client {} authenticated successfully", addr);

                            // Send success response
                            send_auth_response(&mut write, true, None).await?;
                        } else {
                            warn!("Client {} auth failed: invalid token", addr);

                            // Send failure response
                            send_auth_response(
                                &mut write,
                                false,
                                Some("Invalid authentication token"),
                            )
                            .await?;

                            return Ok(()); // Close connection
                        }
                    }
                    _ => {
                        warn!(
                            "Client {} auth failed: first message was not auth handshake",
                            addr
                        );
                        return Ok(()); // Close connection (no response)
                    }
                }
            }
            Ok(_) => {
                warn!("Client {} sent non-binary first message", addr);
                return Ok(()); // Close connection
            }
            Err(e) => {
                error!("Error reading first message from {}: {}", addr, e);
                return Err(IpcError::Read {
                    message: format!("Error reading first message: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                });
            }
        }
    } else {
        warn!("Client {} disconnected before sending auth", addr);
        return Ok(());
    }

    // Create shared state for server management
    let ipc_state = IpcState::new();

    // Main message loop (authenticated)
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                // Decode protobuf client message
                let client_msg = match IpcClientMessage::decode(&data[..]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to decode protobuf from {}: {}", addr, e);
                        send_error_response(
                            &mut write,
                            0,
                            InvalidMessage,
                            "Invalid protobuf message",
                        )
                        .await?;
                        continue;
                    }
                };

                // Handle the message
                let request_id = client_msg.request_id;
                if let Some(payload) = client_msg.payload {
                    match handle_message(payload, &ipc_state, request_id, &mut write).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error handling message from {}: {}", addr, e);
                            send_error_response(
                                &mut write,
                                request_id,
                                InternalError,
                                &e.to_string(),
                            )
                            .await?;
                        }
                    }
                } else {
                    warn!("Client {} sent message with no payload", addr);
                    send_error_response(
                        &mut write,
                        request_id,
                        InvalidMessage,
                        "No payload in message",
                    )
                    .await?;
                }
            }
            Ok(_) => {
                warn!("Client {} sent non-binary message after auth", addr);
                // Ignore non-binary messages
            }
            Err(e) => {
                error!("Error reading message from {}: {}", addr, e);
                return Err(IpcError::Read {
                    message: format!("Error reading message: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                });
            }
        }
    }

    info!("Client {} disconnected", addr);
    Ok(())
}

/// Send authentication response to client.
///
/// # Arguments
///
/// * `write` - WebSocket write half
/// * `success` - Whether authentication succeeded
/// * `error` - Optional error message (if authentication failed)
///
/// # Errors
///
/// Returns [`IpcError::ProtobufEncode`] if encoding fails, or [`IpcError::Send`] if sending fails.
async fn send_auth_response(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
    success: bool,
    error: Option<&str>,
) -> Result<(), IpcError> {
    let response = IpcServerMessage {
        request_id: 1, // Auth handshake always uses request_id 1
        payload: Some(ipc_server_message::Payload::AuthHandshakeResponse(
            IpcAuthHandshakeResponse {
                success,
                error: error.map(|s| s.to_string()),
            },
        )),
    };

    let mut buf = Vec::new();
    response
        .encode(&mut buf)
        .map_err(|e| IpcError::ProtobufEncode {
            message: format!("Failed to encode auth response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })?;

    write
        .send(Message::Binary(buf.into()))
        .await
        .map_err(|e| IpcError::Send {
            message: format!("Failed to send auth response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })
}

/// Send an error response to client.
///
/// # Arguments
///
/// * `write` - WebSocket write half
/// * `request_id` - Request ID to correlate with original request
/// * `error_message` - Human-readable error message
///
/// # Errors
///
/// Returns [`IpcError`] if encoding or sending fails.
async fn send_error_response(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
    request_id: u64,
    error_code: IpcErrorCode,
    error_message: &str,
) -> Result<(), IpcError> {
    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::Error(IpcErrorResponse {
            code: error_code as i32,
            message: error_message.to_string(),
        })),
    };

    let mut buf = Vec::new();
    response
        .encode(&mut buf)
        .map_err(|e| IpcError::ProtobufEncode {
            message: format!("Failed to encode error response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })?;

    write
        .send(Message::Binary(buf.into()))
        .await
        .map_err(|e| IpcError::Send {
            message: format!("Failed to send error response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })
}

/// Handle a single IPC message payload.
///
/// Routes the message to the appropriate handler based on payload type.
async fn handle_message(
    payload: ipc_client_message::Payload,
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    use ipc_client_message::Payload;

    match payload {
        // Server Management - Call real handlers
        Payload::DiscoverServer(_req) => handle_discover_server(state, request_id, write).await,
        Payload::SpawnServer(_req) => handle_spawn_server(state, request_id, _req, write).await,
        Payload::CheckHealth(_req) => handle_check_health(state, request_id, write).await,
        Payload::StopServer(_req) => handle_stop_server(state, request_id, write).await,

        // Sessions (stub)
        Payload::ListSessions(_) => {
            send_error_response(
                write,
                request_id,
                NotImplemented,
                "Sessions not yet implemented",
            )
            .await
        }
        Payload::CreateSession(_) => {
            send_error_response(
                write,
                request_id,
                NotImplemented,
                "Sessions not yet implemented",
            )
            .await
        }
        Payload::DeleteSession(_) => {
            send_error_response(
                write,
                request_id,
                NotImplemented,
                "Sessions not yet implemented",
            )
            .await
        }

        // Auth handshake should not appear after initial auth
        Payload::AuthHandshake(_) => {
            send_error_response(
                write,
                request_id,
                AuthError,
                "Auth handshake already completed",
            )
            .await
        }

        // Catch-all for other operations
        _ => {
            send_error_response(
                write,
                request_id,
                NotImplemented,
                "Operation not yet implemented",
            )
            .await
        }
    }
}

/// Handle discover server request.
async fn handle_discover_server(
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling discover_server request");

    let result = process::discover().map_err(|e| IpcError::Io {
        message: format!("Discovery failed: {e}"),
        location: ErrorLocation::from(Location::caller()),
    })?;

    if let Some(ref server_info) = result {
        state
            .update(StateCommand::SetServer(server_info.clone()))
            .await?;
        info!(
            "Discovered server: PID={}, port={}",
            server_info.pid, server_info.port
        );
    } else {
        info!("No server found");
    }

    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::DiscoverServerResponse(
            IpcDiscoverServerResponse { server: result },
        )),
    };

    send_protobuf_response(write, &response).await
}

/// Handle spawn server request.
async fn handle_spawn_server(
    state: &IpcState,
    request_id: u64,
    _req: IpcSpawnServerRequest,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling spawn_server request");

    let server_info = spawn::spawn_and_wait().await.map_err(|e| IpcError::Io {
        message: format!("Spawn failed: {e}"),
        location: ErrorLocation::from(Location::caller()),
    })?;

    state
        .update(StateCommand::SetServer(server_info.clone()))
        .await?;
    info!(
        "Spawned server: PID={}, port={}",
        server_info.pid, server_info.port
    );

    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::SpawnServerResponse(
            IpcSpawnServerResponse {
                server: Some(server_info),
            },
        )),
    };

    send_protobuf_response(write, &response).await
}

/// Handle check health request.
async fn handle_check_health(
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling check_health request");

    let server_info = state.get_server().await.ok_or_else(|| IpcError::Io {
        message: "No server connected".to_string(),
        location: ErrorLocation::from(Location::caller()),
    })?;

    let healthy = process::check_health(&server_info.base_url).await;
    info!("Health check result: {healthy}");

    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::CheckHealthResponse(
            IpcCheckHealthResponse { healthy },
        )),
    };

    send_protobuf_response(write, &response).await
}

/// Handle stop server request.
async fn handle_stop_server(
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling stop_server request");

    let server_info = state.get_server().await.ok_or_else(|| IpcError::Io {
        message: "No server connected".to_string(),
        location: ErrorLocation::from(Location::caller()),
    })?;

    let success = process::stop_pid(server_info.pid);

    if success {
        state.update(StateCommand::ClearServer).await?;
        info!("Stopped server PID={}", server_info.pid);
    } else {
        warn!("Failed to stop server PID={}", server_info.pid);
    }

    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::StopServerResponse(
            IpcStopServerResponse { success },
        )),
    };

    send_protobuf_response(write, &response).await
}

/// Send a protobuf response message.
async fn send_protobuf_response(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
    response: &IpcServerMessage,
) -> Result<(), IpcError> {
    let mut buf = Vec::new();
    response
        .encode(&mut buf)
        .map_err(|e| IpcError::ProtobufEncode {
            message: format!("Failed to encode response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })?;

    write
        .send(Message::Binary(buf.into()))
        .await
        .map_err(|e| IpcError::Send {
            message: format!("Failed to send response: {e}"),
            location: ErrorLocation::from(Location::caller()),
        })
}
