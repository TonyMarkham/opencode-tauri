# Next Session: IPC Server - Session Handlers

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting.**

## Quick Context

**What We Completed (Session 6.5 - 2026-01-07):**
- JSON field normalizer with build-time code generation
- `normalize_json()` / `denormalize_json()` functions in `field_normalizer` module
- Handles OpenCode's JavaScript naming: `projectID` `project_id`, `sessionID` `session_id`
- All tests passing (18 unit + 24 integration)

**Current State:**
- IPC server works: auth handshake, binary protobuf, server management (discover/spawn/health/stop)
- Session handlers are stubs returning `NOT_IMPLEMENTED` error
- Proto types ready: `OcSessionList`, `OcSessionInfo`, `IpcListSessionsRequest`, etc.

**What's Missing:**
- `OpencodeClient` HTTP client to talk to OpenCode server
- Real session handler implementations
- Integration of field normalizer with HTTP responses

---

## Your Mission: Session 7

Implement session handlers (list, create, delete) by:
1. Creating an `OpencodeClient` HTTP module
2. Storing the client in `IpcState` (created when server is set)
3. Replacing stub handlers with real implementations

### Step 1: Create OpencodeClient Module

**Goal:** HTTP client for OpenCode server API

**Create:** `backend/client-core/src/opencode_client/mod.rs`

```rust
use reqwest::Client;
use url::Url;
use std::path::PathBuf;
use crate::field_normalizer::normalize_json;
use crate::proto::session::OcSessionInfo;

#[derive(Clone)]
pub struct OpencodeClient {
    base_url: Url,
    http: Client,
    pub directory: Option<PathBuf>,
}

impl OpencodeClient {
    pub fn new(base_url: &str) -> Result<Self, OpencodeClientError> { ... }
    
    pub async fn list_sessions(&self) -> Result<Vec<OcSessionInfo>, OpencodeClientError> {
        let url = self.base_url.join("session")?;
        let response = self.prepare_request(self.http.get(url)).send().await?;
        
        // IMPORTANT: Use field normalizer on JSON response
        let json: serde_json::Value = response.json().await?;
        let normalized = normalize_json(json);  // projectID -> project_id
        let sessions: Vec<OcSessionInfo> = serde_json::from_value(normalized)?;
        
        Ok(sessions)
    }
    
    pub async fn create_session(&self, title: Option<&str>) -> Result<OcSessionInfo, ...> { ... }
    pub async fn delete_session(&self, session_id: &str) -> Result<bool, ...> { ... }
    
    fn prepare_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // Add x-opencode-directory header if set
    }
}
```

**Add to lib.rs:** `pub mod opencode_client;`

### Step 2: Integrate with IpcState

**Goal:** Store `OpencodeClient` in state, create when server is set

**Modify:** `backend/client-core/src/ipc/state.rs`

```rust
// Add field to IpcState
opencode_client: Arc<RwLock<Option<OpencodeClient>>>,

// Add getter method
pub async fn get_opencode_client(&self) -> Option<OpencodeClient> {
    self.opencode_client.read().await.clone()
}

// In state_actor, when SetServer:
let client = OpencodeClient::new(&new_server.base_url)?;
*opencode_client_write = Some(client);

// When ClearServer:
*opencode_client_write = None;
```

### Step 3: Implement Session Handlers

**Goal:** Replace stubs in `server.rs` with real handlers

**Modify:** `backend/client-core/src/ipc/server.rs`

```rust
async fn handle_list_sessions(state: &IpcState, request_id: u64, write: &mut ...) -> Result<(), IpcError> {
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
        payload: Some(ipc_server_message::Payload::SessionList(OcSessionList { sessions })),
    };
    
    send_protobuf_response(write, &response).await
}
```

Similar pattern for `handle_create_session` and `handle_delete_session`.

### Step 4: Add Integration Tests

**Create:** `backend/client-core/integration_tests/session_tests/mod.rs`

Tests:
- `given_no_server_when_list_sessions_then_returns_error` (unit test - no server needed)
- `given_server_when_list_sessions_then_returns_list` (ignored - needs real server)
- `given_server_when_create_session_then_returns_session_info` (ignored)

---

## OpenCode API Reference

| Operation | Endpoint | Method | Request | Response |
|-----------|----------|--------|---------|----------|
| List | `/session` | GET | - | `SessionInfo[]` |
| Create | `/session` | POST | `{ title?: string }` | `SessionInfo` |
| Delete | `/session/{id}` | DELETE | - | `true` |

**Required Header:** `x-opencode-directory: <path>`

**JSON Field Names (OpenCode JavaScript):**
- `projectID`, `sessionID`, `parentID`, `messageID`
- Use `normalize_json()` to convert to snake_case before deserializing

---

## Key Files to Reference

**Existing (Read First):**
- `backend/client-core/src/ipc/server.rs` - Current handler pattern (see `handle_discover_server`)
- `backend/client-core/src/ipc/state.rs` - State management pattern
- `backend/client-core/src/field_normalizer.rs` - JSON normalization (just an include!)
- `proto/ipc.proto` - IPC message definitions
- `proto/oc_session.proto` - Session proto types

**To Create:**
- `backend/client-core/src/opencode_client/mod.rs` - HTTP client
- `backend/client-core/src/opencode_client/error.rs` - Error types (optional, can use existing)

**To Modify:**
- `backend/client-core/src/lib.rs` - Add module
- `backend/client-core/src/ipc/state.rs` - Add client field
- `backend/client-core/src/ipc/server.rs` - Implement handlers

---

## Success Criteria

- [ ] `cargo build -p client-core` succeeds
- [ ] `cargo test -p client-core` passes
- [ ] `cargo clippy -p client-core` passes
- [ ] Session handlers return real data (field normalizer working)
- [ ] "No server" case returns proper error

---

## Important Reminders

1. **Field normalizer is critical** - Every JSON response needs `normalize_json()`
2. **Check for server first** - All handlers must check `get_opencode_client().is_some()`
3. **Follow existing patterns** - `handle_discover_server` is your template
4. **reqwest::Client is Clone** - It uses Arc internally, safe to clone into state
5. **No new dependencies needed** - reqwest, serde_json, url already in Cargo.toml

---

**Start with:** Create `opencode_client/mod.rs` with `OpencodeClient::new()` and `list_sessions()`. Test manually that JSON normalization works correctly.
