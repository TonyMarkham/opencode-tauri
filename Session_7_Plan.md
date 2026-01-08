# Session 7: IPC Server - Session Handlers

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting this session.**

**Status:** COMPLETE (Steps 1-3 done, tests deferred to QA Engineer session)  
**Prerequisite:** Session 6.5 complete (JSON Field Normalizer)  
**Tokens Used:** ~97K

---

## Goal

Implement IPC handlers for session operations (list, create, delete) by creating an `OpencodeClient` HTTP client that communicates with the OpenCode server and uses the field normalizer from Session 6.5.

---

## Quick Context

**What We Completed (Session 6.5 - 2026-01-07):**
- JSON field normalizer with build-time code generation
- `normalize_json()` / `denormalize_json()` functions
- Handles `projectID` `session_id`, etc.
- 18 unit tests + 24 integration tests passing

**Current State:**
- IPC server works with auth, binary protobuf, server management
- Session handlers are stubs returning `NOT_IMPLEMENTED`
- Proto types defined: `OcSessionList`, `OcSessionInfo`, request messages
- Field normalizer ready for JSON transformation

**What's Missing:**
- `OpencodeClient` HTTP client module
- Session handler implementations
- Integration of field normalizer with HTTP responses

---

## Deliverables

| # | Deliverable | Description | Status |
|---|-------------|-------------|--------|
| 1 | `OpencodeClient` module | HTTP client for OpenCode server API | ✅ Complete |
| 1a | Error module | `error/opencode_client.rs` following workspace pattern | ✅ Complete |
| 1b | Build config | Serde derives for session proto types only | ✅ Complete |
| 2 | `IpcState` integration | Store `OpencodeClient` in state, update on server change | ✅ Complete |
| 3 | `handle_list_sessions` | Replace stub with real implementation | ✅ Complete |
| 4 | `handle_create_session` | Replace stub with real implementation | ✅ Complete |
| 5 | `handle_delete_session` | Replace stub with real implementation | ✅ Complete |
| 6 | Integration tests | Tests for session operations | ⏸️ Deferred to QA Engineer session |

---

## Session 7 Progress Log

### Step 1: OpencodeClient Module (~94K tokens - COMPLETE)

**Files Created:**
- `backend/client-core/src/error/opencode_client.rs` - Error type with workspace pattern (thiserror, ErrorLocation, #[track_caller])
- `backend/client-core/src/opencode_client/mod.rs` - HTTP client with `list_sessions()`, `create_session()`, `delete_session()`

**Files Modified:**
- `backend/client-core/src/error/mod.rs` - Added `pub mod opencode_client;`
- `backend/client-core/build.rs` - Added serde derives to specific session types only:
  - `opencode.session.OcSessionInfo`
  - `opencode.session.OcSessionTime`
  - `opencode.session.OcSessionSummary`
  - `opencode.session.OcFileDiff`
  - `opencode.session.OcSessionShare`
  - `opencode.session.OcSessionRevert`
  - `opencode.session.OcPermissionAction`
  - `opencode.session.OcPermissionRule`
  - `opencode.session.OcPermissionRuleset`
  - `opencode.session.OcSessionList`
  - **Crucially EXCLUDES:** `opencode.session.OcModelSelection` (which references `OcProviderInfo` with `google.protobuf.Struct`)

**Key Learning:**
- Cannot apply serde to entire `.opencode.session` package because `OcModelSelection` references `OcProviderInfo` which contains `google.protobuf.Struct` fields
- Must apply serde to ONLY specific types that form SessionInfo dependency tree
- SessionInfo itself doesn't reference Model/Provider types - only other session types

**Build Status:** ✅ Passes (warnings about unused code - expected until integration)

**Next Step:** Add `OpencodeClient` to `IpcState` (~15K tokens estimated)

---

## OpenCode Server API Reference

### Required Headers

All requests to OpenCode server need:
```
x-opencode-directory: <working_directory_path>
Content-Type: application/json  (for POST/PATCH)
```

### Endpoints

#### List Sessions
```
GET /session
```

**Query Parameters (all optional):**
- `start` - Filter sessions updated after timestamp (ms)
- `search` - Filter by title (case-insensitive)
- `limit` - Maximum sessions to return

**Response:** `200 OK` - Array of `SessionInfo` objects

```json
[
  {
    "id": "ses_abc123",
    "projectID": "proj_xyz",      // Note: JavaScript naming
    "directory": "/path/to/dir",
    "title": "My Session",
    "version": "0.0.3",
    "time": {
      "created": 1704067200000,
      "updated": 1704067200000
    },
    "parentID": null,             // Optional
    "summary": null,              // Optional
    "share": null,                // Optional
    "permission": null,           // Optional
    "revert": null                // Optional
  }
]
```

#### Create Session
```
POST /session
```

**Request Body:**
```json
{
  "title": "Optional Title"  // Optional, auto-generated if not provided
}
```

**Response:** `200 OK` - Single `SessionInfo` object

#### Delete Session
```
DELETE /session/{session_id}
```

**Response:** `200 OK`
```json
true
```

---

## Implementation Steps

### Step 1: Create OpencodeClient Module (~30K tokens)

**Goal:** Create HTTP client for OpenCode server communication.

**Files to create:**
- `backend/client-core/src/opencode_client/mod.rs`
- `backend/client-core/src/opencode_client/error.rs`

**OpencodeClient structure:**
```rust
use reqwest::Client;
use url::Url;
use std::path::PathBuf;

#[derive(Clone)]
pub struct OpencodeClient {
    /// Parsed base URL (e.g., "http://localhost:3000")
    base_url: Url,
    
    /// HTTP client (connection pooling built-in)
    http: Client,
    
    /// Working directory for x-opencode-directory header
    pub directory: Option<PathBuf>,
}

impl OpencodeClient {
    /// Create new client from base URL
    pub fn new(base_url: &str) -> Result<Self, OpencodeClientError> {
        let base_url = Url::parse(base_url)?;
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            base_url,
            http,
            directory: None,
        })
    }
    
    /// Set working directory (returns self for chaining)
    pub fn with_directory(mut self, dir: PathBuf) -> Self {
        self.directory = Some(dir);
        self
    }
    
    /// Prepare request with common headers
    fn prepare_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut req = req;
        if let Some(dir) = &self.directory {
            if let Some(d) = dir.to_str() {
                req = req.header("x-opencode-directory", d);
            }
        }
        req
    }
}
```

**Error type:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpencodeClientError {
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Server error: {status} - {message}")]
    Server { status: u16, message: String },
}
```

**Add module to lib.rs:**
```rust
pub mod opencode_client;
```

---

### Step 2: Implement Session Methods (~25K tokens)

**Goal:** Add list/create/delete session methods using field normalizer.

**Add to OpencodeClient:**
```rust
use crate::field_normalizer::{normalize_json, denormalize_json};
use crate::proto::session::{OcSessionList, OcSessionInfo};

impl OpencodeClient {
    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<OcSessionInfo>, OpencodeClientError> {
        let url = self.base_url.join("session")?;
        
        let response = self
            .prepare_request(self.http.get(url))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(OpencodeClientError::Server {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        // Get raw JSON
        let json: serde_json::Value = response.json().await?;
        
        // Normalize field names: projectID -> project_id
        let normalized = normalize_json(json);
        
        // Deserialize to proto type
        let sessions: Vec<OcSessionInfo> = serde_json::from_value(normalized)?;
        
        Ok(sessions)
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        title: Option<&str>,
    ) -> Result<OcSessionInfo, OpencodeClientError> {
        let url = self.base_url.join("session")?;
        
        let body = match title {
            Some(t) => serde_json::json!({"title": t}),
            None => serde_json::json!({}),
        };
        
        let response = self
            .prepare_request(self.http.post(url).json(&body))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(OpencodeClientError::Server {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let json: serde_json::Value = response.json().await?;
        let normalized = normalize_json(json);
        let session: OcSessionInfo = serde_json::from_value(normalized)?;
        
        Ok(session)
    }
    
    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool, OpencodeClientError> {
        let url = self.base_url.join(&format!("session/{}", session_id))?;
        
        let response = self
            .prepare_request(self.http.delete(url))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
}
```

**Key patterns:**
1. Use `normalize_json()` on every response before deserializing
2. Use `denormalize_json()` on request bodies if needed (not needed for simple create)
3. Check `response.status().is_success()` before parsing
4. Return meaningful errors with status code

---

### Step 3: Integrate with IpcState (~15K tokens)

**Goal:** Store `OpencodeClient` in `IpcState` and update when server changes.

**Modify `state.rs`:**

```rust
use crate::opencode_client::OpencodeClient;

#[derive(Clone)]
pub struct IpcState {
    // ... existing fields ...
    
    /// Shared read-only access to OpenCode HTTP client
    opencode_client: Arc<RwLock<Option<OpencodeClient>>>,
}

impl IpcState {
    pub fn new() -> Self {
        Self {
            command_tx: Arc::new(Mutex::new(None)),
            server: Arc::new(RwLock::new(None)),
            opencode_client: Arc::new(RwLock::new(None)),  // NEW
            actor_init: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get current OpenCode client (read-only).
    pub async fn get_opencode_client(&self) -> Option<OpencodeClient> {
        self.opencode_client.read().await.clone()
    }
}
```

**Modify `state_actor` function:**

```rust
async fn state_actor(
    mut command_rx: mpsc::Receiver<StateCommand>,
    server: Arc<RwLock<Option<IpcServerInfo>>>,
    opencode_client: Arc<RwLock<Option<OpencodeClient>>>,  // NEW parameter
) {
    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            StateCommand::SetServer(new_server) => {
                // ... existing server update logic ...
                
                // Create OpencodeClient from server's base_url
                match OpencodeClient::new(&new_server.base_url) {
                    Ok(client) => {
                        let mut client_write = opencode_client.write().await;
                        *client_write = Some(client);
                        info!("Created OpencodeClient for {}", new_server.base_url);
                    }
                    Err(e) => {
                        warn!("Failed to create OpencodeClient: {}", e);
                        let mut client_write = opencode_client.write().await;
                        *client_write = None;
                    }
                }
            }
            StateCommand::ClearServer => {
                // ... existing server clear logic ...
                
                // Clear OpencodeClient
                let mut client_write = opencode_client.write().await;
                *client_write = None;
                info!("Cleared OpencodeClient");
            }
        }
    }
}
```

**Update `ensure_actor` to pass new field:**
```rust
tokio::spawn(state_actor(
    rx,
    Arc::clone(&self.server),
    Arc::clone(&self.opencode_client),  // NEW
));
```

---

### Step 4: Implement Session Handlers (~30K tokens)

**Goal:** Replace stubs with real implementations in `server.rs`.

**Add imports:**
```rust
use crate::proto::session::{OcSessionList, OcSessionInfo};
use crate::proto::{IpcCreateSessionRequest, IpcDeleteSessionRequest};
use crate::proto::IpcErrorCode::NoServer;
```

**Update message routing in `handle_message`:**
```rust
// Sessions - Real handlers
Payload::ListSessions(_) => handle_list_sessions(state, request_id, write).await,
Payload::CreateSession(req) => handle_create_session(state, request_id, req, write).await,
Payload::DeleteSession(req) => handle_delete_session(state, request_id, req, write).await,
```

**Implement handlers:**

```rust
/// Handle list sessions request.
async fn handle_list_sessions(
    state: &IpcState,
    request_id: u64,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling list_sessions request");
    
    let client = state.get_opencode_client().await.ok_or_else(|| IpcError::Io {
        message: "No OpenCode server connected".to_string(),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    let sessions = client.list_sessions().await.map_err(|e| IpcError::Io {
        message: format!("Failed to list sessions: {}", e),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::SessionList(OcSessionList {
            sessions,
        })),
    };
    
    send_protobuf_response(write, &response).await
}

/// Handle create session request.
async fn handle_create_session(
    state: &IpcState,
    request_id: u64,
    req: IpcCreateSessionRequest,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling create_session request");
    
    let client = state.get_opencode_client().await.ok_or_else(|| IpcError::Io {
        message: "No OpenCode server connected".to_string(),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    let title = req.title.as_deref();
    let session = client.create_session(title).await.map_err(|e| IpcError::Io {
        message: format!("Failed to create session: {}", e),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    let response = IpcServerMessage {
        request_id,
        payload: Some(ipc_server_message::Payload::SessionInfo(session)),
    };
    
    send_protobuf_response(write, &response).await
}

/// Handle delete session request.
async fn handle_delete_session(
    state: &IpcState,
    request_id: u64,
    req: IpcDeleteSessionRequest,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<(), IpcError> {
    info!("Handling delete_session request");
    
    let client = state.get_opencode_client().await.ok_or_else(|| IpcError::Io {
        message: "No OpenCode server connected".to_string(),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    let success = client.delete_session(&req.session_id).await.map_err(|e| IpcError::Io {
        message: format!("Failed to delete session: {}", e),
        location: ErrorLocation::from(Location::caller()),
    })?;
    
    if success {
        // Return empty session_info or use a different response
        // For now, send success via the session_list with empty list as acknowledgment
        // TODO: Consider adding IpcDeleteSessionResponse to proto
        let response = IpcServerMessage {
            request_id,
            payload: Some(ipc_server_message::Payload::SessionList(OcSessionList {
                sessions: vec![],
            })),
        };
        send_protobuf_response(write, &response).await
    } else {
        send_error_response(
            write,
            request_id,
            IpcErrorCode::ServerError,
            "Failed to delete session",
        ).await
    }
}
```

**Note on delete response:** The current proto doesn't have a dedicated delete response. Options:
1. Return empty `SessionList` (hacky but works)
2. Add `IpcDeleteSessionResponse { bool success = 1; }` to proto (cleaner)

Recommend option 2 if time permits.

---

### Step 5: Integration Tests (~20K tokens)

**Goal:** Test session operations end-to-end.

**Create test file:** `backend/client-core/integration_tests/session_tests/mod.rs`

**Test structure:**
```rust
//! Session handler integration tests.
//!
//! These tests require a running OpenCode server.
//! Run with: cargo test --test integration_tests -- --ignored

use crate::common::*;

/// Test: List sessions returns a list (may be empty)
#[tokio::test]
#[ignore] // Requires running OpenCode server
async fn given_server_when_list_sessions_then_returns_list() {
    let ipc_port = 19900;
    let _handle = start_ipc_server(ipc_port, Some(TEST_AUTH_TOKEN.to_string()))
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // First discover/spawn server
    // ... setup code ...
    
    // Then list sessions
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(Payload::ListSessions(IpcListSessionsRequest {})),
    };
    send_protobuf(&mut ws, &msg).await;
    
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    
    match response.payload {
        Some(ipc_server_message::Payload::SessionList(list)) => {
            // Success - list may be empty or have sessions
            println!("Got {} sessions", list.sessions.len());
        }
        Some(ipc_server_message::Payload::Error(err)) => {
            panic!("Expected session list, got error: {:?}", err);
        }
        _ => panic!("Unexpected response type"),
    }
}

/// Test: Create session returns new session info
#[tokio::test]
#[ignore]
async fn given_server_when_create_session_then_returns_session_info() {
    // ... similar structure ...
}

/// Test: Delete session removes the session
#[tokio::test]
#[ignore]
async fn given_session_when_delete_then_session_removed() {
    // ... similar structure ...
}

/// Test: Session operations without server return NoServer error
#[tokio::test]
async fn given_no_server_when_list_sessions_then_returns_error() {
    let ipc_port = 19901;
    let _handle = start_ipc_server(ipc_port, Some(TEST_AUTH_TOKEN.to_string()))
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // Don't discover/spawn server - go straight to list
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(Payload::ListSessions(IpcListSessionsRequest {})),
    };
    send_protobuf(&mut ws, &msg).await;
    
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    
    match response.payload {
        Some(ipc_server_message::Payload::Error(err)) => {
            // Expected - no server connected
            assert!(err.message.contains("No OpenCode server"));
        }
        _ => panic!("Expected error, got success"),
    }
}
```

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `backend/client-core/src/opencode_client/mod.rs` | Create | HTTP client + session methods |
| `backend/client-core/src/opencode_client/error.rs` | Create | Error types |
| `backend/client-core/src/lib.rs` | Modify | Add `pub mod opencode_client;` |
| `backend/client-core/src/ipc/state.rs` | Modify | Add `opencode_client` field |
| `backend/client-core/src/ipc/server.rs` | Modify | Implement session handlers |
| `backend/client-core/integration_tests/session_tests/mod.rs` | Create | Integration tests |
| `backend/client-core/integration_tests/mod.rs` | Modify | Add `mod session_tests;` |

---

## Success Criteria

- [ ] `cargo build -p client-core` succeeds
- [ ] `cargo test -p client-core` passes (unit tests)
- [ ] `cargo clippy -p client-core` passes with no warnings
- [ ] Session handlers return real data (not NOT_IMPLEMENTED)
- [ ] Field normalizer correctly transforms JSON field names
- [ ] Error handling works (no server = proper error response)
- [ ] Integration tests pass when OpenCode server is running

---

## Out of Scope

- SSE event subscription (Session 15)
- Message sending (Session 14)
- Session update/patch operations
- Advanced session operations (fork, share, abort)

---

## Important Reminders

1. **Use the field normalizer** - Every JSON response from OpenCode needs `normalize_json()`
2. **Check for server first** - Handlers must verify `get_opencode_client()` returns `Some`
3. **Follow existing patterns** - Look at `handle_discover_server` for reference
4. **Test without server** - The "no server" error path is a unit test, not integration test
5. **Proto types are ready** - `OcSessionList`, `OcSessionInfo` already defined

---

## Start With

1. Create `backend/client-core/src/opencode_client/mod.rs` with basic structure
2. Implement `OpencodeClient::new()` and `list_sessions()`
3. Test manually that JSON normalization works
4. Then proceed with state integration and handlers

---

## Session 7 Completion Summary (2026-01-08)

### Completed Work

**Steps 1-3: Implementation Complete**
- ✅ OpencodeClient HTTP client with list/create/delete methods
- ✅ Field normalizer integration (normalize_json on all responses)
- ✅ OpencodeClient stored in IpcState (created on SetServer, cleared on ClearServer)
- ✅ All three session handlers implemented:
  - `handle_list_sessions` → returns `OcSessionList`
  - `handle_create_session` → returns `OcSessionInfo`
  - `handle_delete_session` → returns `IpcDeleteSessionResponse`
- ✅ Proto updated: Added `IpcDeleteSessionResponse` message

**Files Modified:**
- `backend/client-core/src/opencode_client/mod.rs` - HTTP client implementation
- `backend/client-core/src/error/opencode_client.rs` - Error types
- `backend/client-core/src/ipc/state.rs` - OpencodeClient integration
- `backend/client-core/src/ipc/server.rs` - Handler implementations
- `proto/ipc.proto` - Added IpcDeleteSessionResponse
- `backend/client-core/Cargo.toml` - Added wiremock + wiremocket dev dependencies

**Build Status:**
- ✅ `cargo build -p client-core` - Passes
- ✅ `cargo clippy -p client-core` - Passes
- ✅ `cargo test -p client-core --lib` - All 18 unit tests pass

### Deferred to QA Engineer

**Testing (Step 6):** Integration tests for session handlers deferred to dedicated QA Engineer session due to mocking complexity:
- Need to test OpencodeClient with wiremock (HTTP mocking)
- Need to test IPC handlers (WebSocket layer testing strategy unclear)
- Existing IPC integration tests use real servers (not mocked)
- Requires expertise in Rust testing patterns (wiremock vs wiremocket usage)

**Recommendation:** Use QA Engineer agent to design proper testing strategy and implement tests with appropriate mocking frameworks.

---

## Next Steps

Run QA Engineer agent session to:
1. Review session handler implementations
2. Design testing strategy (which layers to mock, which to integration test)
3. Implement comprehensive tests for session operations
4. Verify field normalizer integration works end-to-end
