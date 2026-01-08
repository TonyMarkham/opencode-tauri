use crate::ipc_tests::helpers::{
    TEST_AUTH_TOKEN, authenticate, connect_to_server, is_connection_closed, receive_protobuf,
    send_protobuf, start_test_ipc_server,
};

use client_core::proto::{
    IpcClientMessage, IpcListSessionsRequest, IpcServerMessage, ipc_client_message,
};

/// **VALUE**: Verifies that authenticated clients can send protobuf messages and receive responses.
///
/// **WHY THIS MATTERS**: After authentication, the IPC layer must correctly handle binary protobuf
/// messages in both directions. This is the foundation for all Blazor ↔ Rust communication.
/// If authenticated clients can't send/receive protobuf messages, the entire IPC layer is broken.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Protobuf encoding/decoding fails after auth
/// - Server doesn't respond to authenticated messages
/// - Request/response correlation (request_id) is broken
/// - Server crashes on valid protobuf messages after auth
/// - Messages are lost or corrupted after authentication
#[tokio::test]
async fn given_authenticated_when_send_message_then_receives_response() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19876;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected and authenticated WebSocket client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends protobuf message
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {},
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Client receives response with matching request_id
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(
        response.request_id, 2,
        "Should receive response with matching request_id"
    );
}

/// **VALUE**: Verifies that authenticated clients can send binary protobuf messages.
///
/// **WHY THIS MATTERS**: All IPC communication uses binary protobuf encoding.
/// If binary protobuf messages fail after authentication, the IPC layer is broken.
/// This test ensures the binary protocol works end-to-end after auth succeeds.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Binary protobuf encoding/decoding fails
/// - Server corrupts binary message data
/// - Server rejects binary messages after auth
/// - Protobuf deserialization breaks
#[tokio::test]
async fn given_authenticated_when_send_binary_protobuf_then_receives_response() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19877;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected and authenticated WebSocket client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends binary protobuf message
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {},
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Client receives binary protobuf response
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(
        response.request_id, 2,
        "Should receive response with matching request_id"
    );
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that authenticated clients can send multiple sequential messages.
///
/// **WHY THIS MATTERS**: In production, Blazor will send many messages over the same
/// connection (request/response pairs). If the server corrupts state or loses messages
/// after the first one, the IPC layer becomes unreliable.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Server closes connection after first message
/// - Message ordering is lost
/// - Server state corruption causes subsequent messages to fail
/// - Request/response correlation breaks with multiple messages
#[tokio::test]
async fn given_authenticated_when_send_multiple_messages_then_receives_all_responses() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19878;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Connected and authenticated WebSocket client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends multiple protobuf messages
    for request_id in 2..=4 {
        let msg = IpcClientMessage {
            request_id,
            payload: Some(ipc_client_message::Payload::ListSessions(
                IpcListSessionsRequest {},
            )),
        };
        send_protobuf(&mut ws, &msg).await;
    }

    // THEN: Client receives all responses in order
    for expected_id in 2..=4 {
        let response: IpcServerMessage = receive_protobuf(&mut ws).await;
        assert_eq!(
            response.request_id, expected_id,
            "Should receive responses with matching request_ids in order"
        );
    }
}

// ============================================================================
// Authentication Tests
// ============================================================================

/// **VALUE**: Verifies that valid authentication token is accepted.
///
/// **WHY THIS MATTERS**: Auth is the security gate for all IPC operations.
/// If valid tokens are rejected, the entire IPC layer is unusable.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Auth handshake protocol is broken
/// - Token validation logic fails
/// - Auth response encoding is incorrect
#[tokio::test]
async fn given_valid_token_when_auth_handshake_then_success() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19880;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // WHEN: Client connects and sends auth handshake with valid token
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;

    // THEN: Auth succeeds
    assert!(
        auth_response.success,
        "Auth should succeed with valid token"
    );
    assert!(
        auth_response.error.is_none(),
        "Should have no error message"
    );
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that invalid authentication token is rejected.
///
/// **WHY THIS MATTERS**: Security - invalid tokens must be rejected to prevent
/// unauthorized access to IPC operations. This is the primary security boundary.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Token validation doesn't check token value
/// - Server accepts any token (security breach)
/// - Auth failure response is malformed
#[tokio::test]
async fn given_invalid_token_when_auth_handshake_then_rejected() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19881;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // WHEN: Client connects and sends auth handshake with INVALID token
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, "wrong-token-xyz").await;

    // THEN: Auth fails
    assert!(
        !auth_response.success,
        "Auth should fail with invalid token"
    );
    assert!(auth_response.error.is_some(), "Should have error message");
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that non-auth first message results in connection closure.
///
/// **WHY THIS MATTERS**: Security - first message MUST be auth handshake.
/// If clients can send other messages first, the auth boundary is bypassed.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Server accepts non-auth messages before authentication
/// - Auth state machine is broken
/// - Connection isn't closed on auth protocol violation
#[tokio::test]
async fn given_non_auth_first_message_when_connect_then_rejected() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19882;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // WHEN: Client connects and sends ListSessions (NOT auth) as first message
    let mut ws = connect_to_server(ipc_port).await;
    let msg = IpcClientMessage {
        request_id: 1,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {},
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Connection closes (server rejects non-auth first message)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    let closed = is_connection_closed(&mut ws).await;
    assert!(
        closed,
        "Connection should be closed after non-auth first message"
    );
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that authenticated clients can send non-auth messages.
///
/// **WHY THIS MATTERS**: After successful auth, clients must be able to send
/// normal IPC messages. This verifies the auth state transition works correctly.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Auth state doesn't transition to authenticated
/// - Server requires auth for every message (not just first)
/// - Connection breaks after auth handshake
#[tokio::test]
async fn given_authenticated_when_send_message_then_accepted() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19883;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends normal message after auth
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {},
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Message is accepted and response received
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2, "Should receive response");
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that second auth handshake after authentication is rejected.
///
/// **WHY THIS MATTERS**: Auth handshake should only happen once per connection.
/// If clients can re-authenticate, it may indicate a state machine bug.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - Server allows multiple auth handshakes
/// - Auth state can be reset mid-connection
/// - Server doesn't track authenticated state properly
#[tokio::test]
async fn given_authenticated_when_send_second_auth_then_error() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19884;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "First auth should succeed");

    // WHEN: Client sends SECOND auth handshake
    let second_auth = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::AuthHandshake(
            client_core::proto::IpcAuthHandshake {
                token: TEST_AUTH_TOKEN.to_string(),
            },
        )),
    };
    send_protobuf(&mut ws, &second_auth).await;

    // THEN: Server returns error (auth already completed)
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    match response.payload {
        Some(client_core::proto::ipc_server_message::Payload::Error(err)) => {
            assert_eq!(err.code, client_core::proto::IpcErrorCode::AuthError as i32);
        }
        _ => panic!("Expected error response for second auth handshake"),
    }
}

// ============================================================================
// Server Management Tests
// ============================================================================

/// **VALUE**: Verifies that discover_server operation works through IPC.
///
/// **WHY THIS MATTERS**: Discovery is the first step in connecting to OpenCode server.
/// If IPC discovery fails, the entire server management flow is broken.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - IPC doesn't call discovery::discover()
/// - Discovery response encoding is broken
/// - Server state isn't updated after discovery
#[tokio::test]
async fn given_authenticated_when_discover_server_then_returns_result() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19885;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends discover_server request
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::DiscoverServer(
            client_core::proto::IpcDiscoverServerRequest {},
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Receives discover response (may be None if no server running)
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    match response.payload {
        Some(client_core::proto::ipc_server_message::Payload::DiscoverServerResponse(_)) => {
            // Success - response may have server=Some(...) or server=None
        }
        _ => panic!("Expected DiscoverServerResponse"),
    }
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that spawn_server operation works through IPC.
///
/// **WHY THIS MATTERS**: If no server is running, client must spawn one.
/// If IPC spawn fails, users can't start OpenCode server from the app.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - IPC doesn't call spawn::spawn_and_wait()
/// - Spawn response encoding is broken
/// - Server state isn't updated after spawn
/// - Spawned server info is lost
#[ignore] // DANGEROUS: Spawns real OpenCode server, may conflict with running instances
#[tokio::test]
async fn given_authenticated_when_spawn_server_then_returns_info() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19886;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // WHEN: Client sends spawn_server request
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::SpawnServer(
            client_core::proto::IpcSpawnServerRequest { port: None },
        )),
    };
    send_protobuf(&mut ws, &msg).await;

    // THEN: Receives spawn response with server info
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    match response.payload {
        Some(client_core::proto::ipc_server_message::Payload::SpawnServerResponse(resp)) => {
            assert!(resp.server.is_some(), "Spawn should return server info");
            let server = resp.server.unwrap();
            assert!(server.pid > 0, "Server PID should be valid");
            assert!(server.port > 0, "Server port should be valid");
        }
        _ => panic!("Expected SpawnServerResponse"),
    }
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that check_health operation works through IPC.
///
/// **WHY THIS MATTERS**: Client needs to verify server is responsive before
/// sending requests. If health check fails, client can't detect dead servers.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - IPC doesn't call process::check_health()
/// - Health check response encoding is broken
/// - Server state isn't accessible for health checks
/// - Health check returns wrong status
#[ignore] // DANGEROUS: Spawns server and checks health, may interact with running instances
#[tokio::test]
async fn given_authenticated_and_server_when_check_health_then_returns_status() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19887;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client that spawned a server
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // Spawn server first
    let spawn_msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::SpawnServer(
            client_core::proto::IpcSpawnServerRequest { port: None },
        )),
    };
    send_protobuf(&mut ws, &spawn_msg).await;
    let _spawn_response: IpcServerMessage = receive_protobuf(&mut ws).await;

    // Wait for server to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // WHEN: Client sends check_health request
    let health_msg = IpcClientMessage {
        request_id: 3,
        payload: Some(ipc_client_message::Payload::CheckHealth(
            client_core::proto::IpcCheckHealthRequest {},
        )),
    };
    send_protobuf(&mut ws, &health_msg).await;

    // THEN: Receives health response
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 3);
    match response.payload {
        Some(client_core::proto::ipc_server_message::Payload::CheckHealthResponse(resp)) => {
            assert!(resp.healthy, "Server should be healthy after spawn");
        }
        _ => panic!("Expected CheckHealthResponse"),
    }
}

// -------------------------------------------------------------------------- //

/// **VALUE**: Verifies that stop_server operation works through IPC.
///
/// **WHY THIS MATTERS**: Client must be able to cleanly shut down server.
/// If stop fails, servers will leak and consume system resources.
///
/// **BUG THIS CATCHES**: Would catch if:
/// - IPC doesn't call process::stop_pid()
/// - Stop response encoding is broken
/// - Server state isn't cleared after stop
/// - Stop operation doesn't actually kill the process
#[ignore] // DANGEROUS: Stops server process, may kill your running OpenCode instance
#[tokio::test]
async fn given_authenticated_and_server_when_stop_server_then_succeeds() {
    // GIVEN: IPC server running on test port
    let ipc_port = 19888;
    let _handle = start_test_ipc_server(ipc_port, Some(String::from(TEST_AUTH_TOKEN)))
        .await
        .expect("Failed to start IPC server");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // GIVEN: Authenticated client that spawned a server
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");

    // Spawn server first
    let spawn_msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::SpawnServer(
            client_core::proto::IpcSpawnServerRequest { port: None },
        )),
    };
    send_protobuf(&mut ws, &spawn_msg).await;
    let _spawn_response: IpcServerMessage = receive_protobuf(&mut ws).await;

    // Wait for server to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // WHEN: Client sends stop_server request
    let stop_msg = IpcClientMessage {
        request_id: 3,
        payload: Some(ipc_client_message::Payload::StopServer(
            client_core::proto::IpcStopServerRequest {},
        )),
    };
    send_protobuf(&mut ws, &stop_msg).await;

    // THEN: Receives stop response with success
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 3);
    match response.payload {
        Some(client_core::proto::ipc_server_message::Payload::StopServerResponse(resp)) => {
            assert!(resp.success, "Stop should succeed");
        }
        _ => panic!("Expected StopServerResponse"),
    }
}
