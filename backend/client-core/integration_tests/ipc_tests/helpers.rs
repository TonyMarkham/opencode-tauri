//! Test helpers for IPC integration tests.
//!
//! This module provides utilities for testing the IPC WebSocket server:
//! - Connecting to server
//! - Sending/receiving protobuf messages
//! - Authentication helpers
//! - Connection state checks

use client_core::proto::{
    IpcAuthHandshake, IpcAuthHandshakeResponse, IpcClientMessage, IpcServerMessage,
    ipc_client_message, ipc_server_message,
};

use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

/// Test constants for authentication
pub const TEST_AUTH_TOKEN: &str = "test-token-12345";

/// Test helper: Connect to IPC server and return WebSocket stream.
pub async fn connect_to_server(ipc_port: u16) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let url = format!("ws://127.0.0.1:{}", ipc_port);
    let (ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect to WebSocket server");
    ws_stream
}

/// Test helper: Send protobuf message over WebSocket.
pub async fn send_protobuf<T: ProstMessage>(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    message: &T,
) {
    let mut buf = Vec::new();
    message.encode(&mut buf).expect("Failed to encode protobuf");
    ws.send(Message::Binary(buf.into()))
        .await
        .expect("Failed to send message");
}

/// Test helper: Receive and decode protobuf message.
pub async fn receive_protobuf<T: ProstMessage + Default>(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> T {
    let msg = ws
        .next()
        .await
        .expect("No message received")
        .expect("Error receiving message");

    let bytes = msg.into_data();
    T::decode(&bytes[..]).expect("Failed to decode protobuf")
}

/// Test helper: Send auth handshake and return response.
pub async fn authenticate(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    token: &str,
) -> IpcAuthHandshakeResponse {
    let auth_msg = IpcClientMessage {
        request_id: 1,
        payload: Some(ipc_client_message::Payload::AuthHandshake(
            IpcAuthHandshake {
                token: token.to_string(),
            },
        )),
    };

    send_protobuf(ws, &auth_msg).await;

    let response: IpcServerMessage = receive_protobuf(ws).await;
    match response.payload {
        Some(ipc_server_message::Payload::AuthHandshakeResponse(resp)) => resp,
        _ => panic!("Expected AuthHandshakeResponse, got something else"),
    }
}

/// Test helper: Check if WebSocket connection is closed.
pub async fn is_connection_closed(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> bool {
    match tokio::time::timeout(tokio::time::Duration::from_millis(100), ws.next()).await {
        Err(_) => true,
        Ok(None) => true,
        Ok(Some(Ok(Message::Close(_)))) => true,
        Ok(Some(Ok(_))) => false,
        Ok(Some(Err(_))) => true,
    }
}
