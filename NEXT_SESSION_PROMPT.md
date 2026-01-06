# Session 5: WebSocket Server

**Goal:** WebSocket server in client-core that handles session operations.

**Demo:** Connect with `wscat`, list sessions, create session.

---

## What Exists

**Rust (`backend/client-core/`):**
- `src/lib.rs` - Exports `discovery`, `error`, `ws`, `proto`
- `src/discovery/` - Server discovery, spawn, health
- `src/error/ws.rs` - `WsError` type (stub)
- `Cargo.toml` - Has `tokio-tungstenite`, `prost`, `futures-util`
- `build.rs` - Protobuf codegen configured

**Proto (`proto/ipc.proto`):**
- `ClientMessage` / `ServerMessage` envelopes
- `AuthHandshake` / `AuthHandshakeResponse`
- `ListSessionsRequest` / `SessionList` / `SessionInfo`
- `CreateSessionRequest` / `DeleteSessionRequest`
- `ErrorResponse`

---

## What to Build

### 1. WebSocket Server Module

Create `backend/client-core/src/ws/mod.rs` and `server.rs`:

```
ws/
├── mod.rs      # Public API: start_server()
└── server.rs   # Server implementation
```

**Public API:**
```rust
pub async fn start_server(port: u16, auth_token: String, opencode_url: String) -> Result<WsHandle, WsError>

pub struct WsHandle {
    // Shutdown handle
}
```

**Server behavior:**
- Bind to `127.0.0.1:{port}` only
- Accept connections
- First message must be `AuthHandshake` with matching token
- Reject bad auth, close connection
- Route `ClientMessage` to handlers
- Send `ServerMessage` responses

### 2. Message Handlers

Handlers that bridge to OpenCode HTTP API:

```rust
async fn handle_list_sessions(opencode_url: &str) -> Result<SessionList, WsError>
// GET {opencode_url}/session

async fn handle_create_session(opencode_url: &str, req: CreateSessionRequest) -> Result<SessionInfo, WsError>
// POST {opencode_url}/session

async fn handle_delete_session(opencode_url: &str, req: DeleteSessionRequest) -> Result<(), WsError>
// DELETE {opencode_url}/session/{id}
```

### 3. Error Handling

Expand `src/error/ws.rs`:

```rust
pub enum WsError {
    Bind { port: u16, source: std::io::Error },
    Auth { message: String },
    Http { status: u16, message: String },
    Protocol { message: String },
    // etc.
}
```

### 4. Tauri Integration

Add to `apps/desktop/opencode/src/commands/`:

```rust
#[tauri::command]
pub fn get_ws_config(state: State<AppState>) -> Result<WsConfig, String>

pub struct WsConfig {
    pub port: u16,
    pub auth_token: String,
}
```

Start WebSocket server in Tauri setup hook.

---

## OpenCode API Reference

**List sessions:**
```
GET /session
Response: { "sessions": [{ "id": "...", "title": "...", "created": ..., "updated": ... }] }
```

**Create session:**
```
POST /session
Body: {}
Response: { "id": "...", "title": null, "created": ..., "updated": ... }
```

**Delete session:**
```
DELETE /session/{id}
Response: 204 No Content
```

---

## Testing

Test with `wscat`:

```bash
# Connect
wscat -c ws://127.0.0.1:PORT

# Send auth (as binary protobuf - or add text debug mode)
```

Or create a simple test in `backend/client-core/integration_tests/`.

---

## Success Criteria

- [ ] WebSocket server starts on app launch
- [ ] Auth handshake works (accept valid, reject invalid)
- [ ] `ListSessions` returns session list from OpenCode
- [ ] `CreateSession` creates session, returns info
- [ ] `DeleteSession` deletes session
- [ ] Tauri command returns WS config (port + token)
- [ ] Server binds only to localhost

---

## Not Building (Later Sessions)

- C# WebSocket client (Session 6)
- Chat UI (Session 7)
- SendMessage (Session 7)
- SSE bridge (Session 8)
- Streaming (Session 8)
- Tool calls (Session 9)
