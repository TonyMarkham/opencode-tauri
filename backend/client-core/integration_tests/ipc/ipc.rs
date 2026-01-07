use client_core::ipc::start_ipc_server;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ============================================================================
// Public API tests for IPC WebSocket server
// These test the PUBLIC interface from an external consumer's perspective
// ============================================================================

// ----------------------------------------------------------------------------
// start_ipc_server() - Echo functionality tests
// ----------------------------------------------------------------------------

/// **VALUE**: Verifies that the IPC server echoes text messages back to the client.
///
/// **WHY THIS MATTERS**: The echo behavior is the foundation for all IPC communication.
/// If the server can't receive and send back messages, the entire Blazor ↔ Rust IPC
/// layer is broken. This validates the WebSocket infrastructure works before we add
/// protobuf and complex message handling.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - WebSocket server fails to bind to localhost
/// - Server doesn't accept connections
/// - Message reading/writing is broken
/// - Server crashes on receiving messages
/// - Messages are corrupted or lost in transit
#[tokio::test]
async fn given_running_ipc_server_when_client_sends_text_then_receives_echo() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19876;
    let _handle = start_ipc_server(ipc_port, Some(String::from("test-token")))
        .await
        .expect("Failed to start IPC server");

    // Give server time to bind (avoids race condition)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected WebSocket client
    let url = format!("ws://127.0.0.1:{}", ipc_port);
    let (mut ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect to WebSocket server");

    // WHEN: Client sends text message
    let test_message = "hello";
    ws_stream
        .send(Message::Text(test_message.to_string().into()))
        .await
        .expect("Failed to send message");

    // THEN: Client receives exact same message back (echo)
    let response = ws_stream
        .next()
        .await
        .expect("No response received")
        .expect("Error receiving message");

    assert_eq!(
        response.into_text().expect("Response was not text"),
        test_message,
        "Server should echo back the exact message"
    );
}

/// **VALUE**: Verifies that the IPC server handles binary messages (not just text).
///
/// **WHY THIS MATTERS**: Protobuf messages (coming in Session 6) are sent as binary
/// over WebSocket. If the server only handles text, the entire protobuf IPC layer
/// will fail. This ensures binary message transport works before we add protobuf.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Server filters out or drops binary messages
/// - Binary data is corrupted during echo
/// - Server crashes on non-UTF8 data
#[tokio::test]
async fn given_running_ipc_server_when_client_sends_binary_then_receives_echo() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19877;
    let _handle = start_ipc_server(ipc_port, Some(String::from("test-token")))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected WebSocket client
    let url = format!("ws://127.0.0.1:{}", ipc_port);
    let (mut ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect to WebSocket server");

    // WHEN: Client sends binary message
    let test_data = vec![0x01, 0x02, 0x03, 0xff];
    ws_stream
        .send(Message::Binary(test_data.clone().into()))
        .await
        .expect("Failed to send binary message");

    // THEN: Client receives exact same binary data back
    let response = ws_stream
        .next()
        .await
        .expect("No response received")
        .expect("Error receiving message");

    assert_eq!(
        response.into_data(),
        test_data,
        "Server should echo back exact binary data"
    );
}

/// **VALUE**: Verifies that the server handles multiple sequential messages correctly.
///
/// **WHY THIS MATTERS**: In production, Blazor will send many messages over the same
/// connection (request/response pairs). If the server corrupts state or loses messages
/// after the first one, the IPC layer becomes unreliable.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Server closes connection after first message
/// - Message ordering is lost
/// - Server state corruption causes subsequent messages to fail
#[tokio::test]
async fn given_running_ipc_server_when_client_sends_multiple_messages_then_receives_all_echoes() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19878;
    let _handle = start_ipc_server(ipc_port, Some(String::from("test-token")))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected WebSocket client
    let url = format!("ws://127.0.0.1:{}", ipc_port);
    let (mut ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect to WebSocket server");

    // WHEN: Client sends multiple messages
    let messages = vec!["first", "second", "third"];
    for msg in &messages {
        ws_stream
            .send(Message::Text(msg.to_string().into()))
            .await
            .expect("Failed to send message");
    }

    // THEN: Client receives all echoes in order
    for expected in &messages {
        let response = ws_stream
            .next()
            .await
            .expect("No response received")
            .expect("Error receiving message");

        assert_eq!(
            response.into_text().expect("Response was not text"),
            *expected,
            "Messages should be echoed in order"
        );
    }
}
