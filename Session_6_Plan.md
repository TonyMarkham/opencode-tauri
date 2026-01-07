# Session 6 Implementation Plan: IPC Server - Auth + Protobuf + Server Management

**Status:** IN PROGRESS - Steps 1-7 Complete  
**Estimated time:** ~3.5 hours  
**Token budget:** 200K tokens

---

## 🎯 PROGRESS SUMMARY

**Completed Prior to Session 6 (Foundation Work):**
- ✅ **Step 1a-1j: All 11 proto files created and compiling** (45 min estimated, DONE)
  - 10 OpenCode canonical proto files (`oc_auth.proto`, `oc_model.proto`, etc.)
  - 1 IPC protocol file (`ipc.proto`)
  - All messages have proper `Ipc*` / `Oc*` prefixes
  - `build.rs` configured to compile all protos
  - Generated code accessible via `client_core::proto::*`
  - Temporary serde added to `IpcServerInfo` for Tauri command compatibility
- ✅ **Models crate refactored**
  - Renamed `models` → `common`
  - Removed `ServerInfo` builder pattern
  - Migrated to `IpcServerInfo` from proto
  - Updated all imports in source and tests
- ✅ **All tests passing** (25 tests: 7 unit + 18 integration) - BEFORE Session 6 work
- ✅ **Build clean** (`cargo build` succeeds, clippy warnings suppressed for generated code)

**Completed in Current Session (Steps 2-7):**
- ✅ **Step 2: File reorganization** (10 min actual)
  - Separated `IpcServerHandle` into `handle.rs` (struct-in-matching-file pattern)
  - Moved server implementation to `server.rs`
  - Added comprehensive module and function documentation
  - Updated exports in `mod.rs`
- ✅ **Step 3: Error types expanded** (15 min actual)
  - Added `Auth`, `ProtobufDecode`, `ProtobufEncode` error variants to `IpcError`
  - Implemented `From<prost::DecodeError>` and `From<prost::EncodeError>`
  - Fixed bug: `Read` error was displaying "Send Error"
  - All error variants now have proper display strings
- ✅ **Step 4: State management migrated** (25 min actual)
  - Created `backend/client-core/src/ipc/state.rs`
  - Implemented `IpcState` with actor pattern (from Tauri's `AppState`)
  - Renamed `AppState` → `IpcState` for clarity
  - Replaced string errors with proper `IpcError` types
  - Added `StateCommand` enum for mutations
  - Lazy actor initialization with `mpsc` channel
- ✅ **Step 5: Test helpers created** (20 min actual)
  - Created `backend/client-core/integration_tests/ipc/helpers.rs`
  - Helper functions: `connect_to_server()`, `send_protobuf()`, `receive_protobuf()`, `authenticate()`, `is_connection_closed()`
  - Test constants: `TEST_AUTH_TOKEN`, `INVALID_AUTH_TOKEN`
  - Fixed initial compilation issue (`.into()` for `Bytes` conversion)
- ✅ **Step 6: Auth state machine implemented** (45 min actual)
  - Created `backend/client-core/src/ipc/connection_state.rs`
  - Implemented `ConnectionState` struct with token validation
  - Updated `start_ipc_server()` to accept `Option<String>` auth token
  - Auto-generates UUID token if not provided
  - First message MUST be `IpcAuthHandshake` with valid token
  - Sends `IpcAuthHandshakeResponse` (success/failure)
  - Rejects non-localhost connections
  - Fail-closed security model (all failures close connection)
  - Added comprehensive security checks and logging
- ✅ **Step 7: Binary protobuf + server handlers** (90 min actual)
  - **Added `IpcErrorCode` enum to proto** (proper error codes, no magic strings)
  - Imported enum variants at top: `InvalidMessage`, `InternalError`, `NotImplemented`, `AuthError`, `NoServer`, `ServerError`
  - Replaced echo logic with protobuf message parsing
  - Added `handle_message()` dispatcher function
  - Implemented 4 server management handlers:
    - `handle_discover_server()` - Calls `discovery::discover()`, updates state
    - `handle_spawn_server()` - Calls `spawn::spawn_and_wait()`, updates state
    - `handle_check_health()` - Gets server from state, calls `check_health()`
    - `handle_stop_server()` - Calls `stop_pid()`, clears state
  - Added `send_protobuf_response()` and `send_error_response()` helpers
  - All handlers send proper protobuf responses
  - Stubs for unimplemented operations (sessions, agents, etc.) return `NOT_IMPLEMENTED` errors
  - Wired up all handler functions in `handle_message()`
  - **Build passes** (only warnings for unused test helpers)

**Remaining for Session 6:**
- ❌ **Step 8: Update tests** (~40 min estimated)
  - Fix 3 broken echo tests (add auth handshake before message)
  - Add 6 auth tests (valid token, invalid token, wrong first message, non-localhost, etc.)
  - Add 4 server management tests (discover, spawn, health, stop)
  - **Current state:** 3 tests failing (expected - they don't authenticate)
- ❌ **Step 9: Update Tauri main.rs** (~5 min estimated)
  - Update `start_ipc_server()` call to pass auth token
  - Deprecate old Tauri commands (discovery/spawn moved to IPC)
- ❌ **Step 10: Cleanup** (~10 min estimated)
  - Run `cargo fmt`
  - Run `cargo clippy` and fix issues
  - Verify logging levels
  - Final test run

**Estimated remaining time:** ~1 hour

---

## 📋 Initialization Prompt for New Session

**Copy this when starting Session 6 in a new conversation:**

```
I need help implementing Session 6 of the OpenCode Tauri-Blazor client.

READ THESE FILES FIRST:
1. /Users/tony/git/opencode-tauri/CRITICAL_OPERATING_CONSTRAINTS.md
2. /Users/tony/git/opencode-tauri/Session_6_Plan.md

KEY REQUIREMENTS:
- Follow ADR-0002 (Thin Tauri Layer) and ADR-0003 (WebSocket + Protobuf IPC)
- All OpenCode models must be 1:1 with JSON Schemas (docs/proto/*.md)
- Teaching mode: Plan each step before implementation, ask permission before writing files
- Production-grade: No shortcuts, no TODOs, comprehensive error handling

Start by confirming you've read both documents, then ask: "Should I explain Step 1, or implement it?"
```

---

## ⚠️ CRITICAL OPERATING CONSTRAINTS

**READ FIRST:** `/Users/tony/git/opencode-tauri/CRITICAL_OPERATING_CONSTRAINTS.md`

This document defines:
- **Teaching Mode** - Plan first, explain, then implement only when asked
- **Production-Grade Requirements** - No shortcuts, comprehensive error handling, full test coverage
- **Planning Process** - Read entire step, identify sub-tasks, consider dependencies, present plan
- **Quality Bar** - Production-ready, not "works on my machine"

**Quick Summary:**
- 🎓 Default = Teach mode (explain, don't write)
- 📋 Before ANY step: Read it ALL, plan, present approach
- 🏭 Production code: No TODOs, no unwraps, no shortcuts
- ✅ Ask permission before writing files

---

## Overview

Transform the working echo server into a production IPC server with:
1. ✅ Auth handshake (first message must be valid token)
2. ✅ Binary protobuf framing (no more text)
3. ✅ Proper naming conventions (`Ipc*`/`Oc*` prefixes)
4. ✅ **Server management migrated from Tauri to client-core** (ADR-0002 compliance)
5. ✅ **Complete OpenCode proto models** (1:1 with JSON Schemas - all 8 domains)

---

## Quick Reference

| Section | Description | Key Info |
|---------|-------------|----------|
| **Overview** | Session goals and scope | 5 deliverables, 3.5 hours |
| **Section 0** | Two-layer proto architecture | `Oc*` vs `Ipc*` separation |
| **Section 1** | Module reorganization | Move code, migrate state |
| **Section 2** | OpenCode proto files | 9 files, ~74 messages |
| **Section 3** | Auth flow state machine | Token validation, security |
| **Section 4** | Error handling | 3 new variants |
| **Section 5** | Logging strategy | Production-grade levels |
| **Section 6** | Test strategy | 13 integration tests |
| **Section 7** | Implementation steps | 10 steps, 45 min → 5 min each |
| **Section 8** | Success criteria | 25+ checkboxes |
| **Section 9** | Out of scope | Sessions 7+ features |
| **Section 10** | Dependencies | Cargo.toml + build.rs |
| **Section 11** | Reference patterns | Existing code examples |
| **Section 12** | Questions resolved | 5 key decisions |
| **Section 13** | Next steps | Session 7 preview |
| **Section 14** | Time breakdown | 3.5 hours with rationale |
| **Summary** | Why the scope change | Before/after comparison |

---

## Critical Architecture Decision: Two-Layer Proto Organization

### Principle: Separation of Concerns

**Layer 1: OpenCode Models (`Oc*` prefix)**
- **Purpose:** 1:1 mapping to OpenCode server's JSON Schema definitions
- **Source of Truth:** `submodules/opencode/schema/*.schema.json` (72+ schemas)
- **Documentation:** `docs/proto/*.md` (8 domains, fully researched)
- **Rule:** NEVER deviate from OpenCode server's data models
- **Examples:** `OcSessionInfo`, `OcAgentInfo`, `OcAuth`, `OcModelInfo`

**Layer 2: IPC Messages (`Ipc*` prefix)**
- **Purpose:** Application-specific IPC protocol (Blazor ↔ client-core)
- **Location:** `proto/ipc.proto`
- **Rule:** Can add application-specific fields for client needs
- **Examples:** `IpcClientMessage`, `IpcAuthHandshake`, `IpcServerMessage`
- **Imports:** References `Oc*` models as needed

### Why This Matters

**Wrong (Current `ipc.proto`):**
```protobuf
// Simplified - doesn't match OpenCode server
message SessionInfo {
  string id = 1;
  optional string title = 2;
  int64 created_at = 3;
  int64 updated_at = 4;
}
```

**Right (Two-layer approach):**
```protobuf
// proto/oc_session.proto - matches sessionInfo.schema.json exactly
message OcSessionInfo {
  string id = 1;
  string project_id = 2;
  string directory = 3;
  optional string parent_id = 4;
  optional OcSessionSummary summary = 5;
  optional OcSessionShare share = 6;
  string title = 7;
  string version = 8;
  OcSessionTime time = 9;
  optional OcPermissionRuleset permission = 10;
  optional OcSessionRevert revert = 11;
}

// proto/ipc.proto - uses the canonical model
import "oc_session.proto";

message IpcServerMessage {
  oneof payload {
    opencode.session.OcSessionList session_list = 20;  // Uses imported type
  }
}
```

### File Organization: One Proto Per Domain

**Rationale:** 72+ OpenCode models in a single file = maintenance nightmare

```
proto/
├── oc_model.proto         # OcModelInfo, OcModelCapabilities, OcModelLimits, etc.
├── oc_provider.proto      # OcProviderInfo, OcProviderList, OcProviderOptions
├── oc_auth.proto          # OcAuth, OcOAuth, OcApiAuth, OcWellKnownAuth
├── oc_session.proto       # OcSessionInfo, OcSessionTime, OcSessionSummary, etc.
├── oc_message.proto       # OcMessage, OcPart variants, OcError types (20+ messages)
├── oc_tool.proto          # OcToolState, OcToolPart, OcPermissionRequest, etc.
├── oc_agent.proto         # OcAgentInfo, OcAgentModel, OcAgentList
├── oc_event.proto         # OcEvent, OcGlobalEvent, event types (13+ messages)
├── oc_server.proto        # OcServerInfo (NEW - for discover/spawn)
└── ipc.proto              # IpcClientMessage, IpcServerMessage (imports oc_*.proto)
```

**Mapping to research:**
```
docs/proto/01-model.md        → proto/oc_model.proto
docs/proto/02-provider.md     → proto/oc_provider.proto
docs/proto/03-auth.md         → proto/oc_auth.proto
docs/proto/04-session.md      → proto/oc_session.proto
docs/proto/05-message.md      → proto/oc_message.proto
docs/proto/06-tool.md         → proto/oc_tool.proto
docs/proto/07-agent.md        → proto/oc_agent.proto
docs/proto/08-event.md        → proto/oc_event.proto
(new)                         → proto/oc_server.proto
```

### Scope Decision: Implement ALL OpenCode Protos in Session 6

**Why now (not incremental):**
1. ✅ Research is complete (all 8 domains documented)
2. ✅ One-time cost (~45 min vs 15 min piecemeal per session)
3. ✅ Future sessions just import, no proto work
4. ✅ Clean architecture from day 1
5. ✅ Validation against JSON Schemas happens once

**Session 6 deliverables:**
- 8 OpenCode proto files (complete, canonical models)
- 1 Server proto file (new - for IPC server info)
- 1 IPC proto file (protocol, imports OpenCode models)
- Verification against `docs/proto/*.md` and JSON Schemas

**Time impact:** +30 min (3 hours → 3.5 hours total)

---

## 1. Module Reorganization

### Current Structure (Wrong)
```
backend/client-core/src/ipc/
├── mod.rs          # Exports from handle.rs
├── handle.rs       # Contains server code (WRONG FILE)
└── server.rs       # EMPTY

apps/desktop/opencode/src/
├── state.rs        # AppState (WRONG LAYER - business logic in Tauri)
└── commands/
    └── server.rs   # Tauri commands (WRONG LAYER - violates ADR-0002)
```

### Target Structure (Correct)
```
backend/client-core/src/ipc/
├── mod.rs          # Exports from server.rs and state.rs
├── server.rs       # WebSocket server implementation
└── state.rs        # IpcState (moved from Tauri layer)

apps/desktop/opencode/src/
├── commands/
│   └── server.rs   # DEPRECATED - Remove after Blazor migration (Session 8)
└── main.rs         # Starts IPC server (thin glue code only)
```

**Why:** 
- Follows established pattern from `discovery/` module
- **Per ADR-0002:** All business logic (state, server management) belongs in client-core
- Tauri layer is ONLY for webview hosting

---

## 2. OpenCode Proto Files (Complete Implementation)

### Files to Create

Each proto file maps 1:1 to research documentation in `docs/proto/`:

| Proto File | Source Doc | Message Count | Status |
|------------|-----------|---------------|--------|
| `oc_model.proto` | `01-model.md` | ~10 messages | From JSON Schema |
| `oc_provider.proto` | `02-provider.md` | ~5 messages | From JSON Schema |
| `oc_auth.proto` | `03-auth.md` | ~4 messages | From JSON Schema |
| `oc_session.proto` | `04-session.md` | ~10 messages | From JSON Schema |
| `oc_message.proto` | `05-message.md` | ~20 messages | From JSON Schema |
| `oc_tool.proto` | `06-tool.md` | ~9 messages | From JSON Schema |
| `oc_agent.proto` | `07-agent.md` | ~2 messages | From JSON Schema |
| `oc_event.proto` | `08-event.md` | ~13 messages | From JSON Schema |
| `oc_server.proto` | (new) | ~1 message | For IPC server info |

**Total: ~74 OpenCode messages** across 9 proto files

### Verification Process

For each proto file, verify against research:

1. **Read source documentation** (`docs/proto/XX-name.md`)
2. **Extract protobuf definitions** from "Messages" section
3. **Cross-reference with JSON Schema tables** in same doc
4. **Copy verbatim** - no modifications, no "simplifications"
5. **Add source comments** - link back to JSON Schema file
6. **Verify field numbering** - matches doc examples

**Example verification (oc_session.proto):**

```protobuf
// Source: submodules/opencode/schema/sessionInfo.schema.json (canonical)
// Documentation: docs/proto/04-session.md lines 48-60
message OcSessionInfo {
  string id = 1;                              // ✓ Matches JSON Schema
  string project_id = 2;                      // ✓ Matches JSON Schema
  string directory = 3;                       // ✓ Matches JSON Schema
  optional string parent_id = 4;              // ✓ Matches JSON Schema (optional)
  optional OcSessionSummary summary = 5;      // ✓ Matches JSON Schema ($ref)
  optional OcSessionShare share = 6;          // ✓ Matches JSON Schema ($ref)
  string title = 7;                           // ✓ Matches JSON Schema
  string version = 8;                         // ✓ Matches JSON Schema
  OcSessionTime time = 9;                     // ✓ Matches JSON Schema ($ref)
  optional OcPermissionRuleset permission = 10; // ✓ Matches JSON Schema ($ref)
  optional OcSessionRevert revert = 11;       // ✓ Matches JSON Schema ($ref)
}
```

### IPC Messages (Separate from OpenCode Models)

**Prefix: `Ipc*`** - These are IPC protocol messages (in `proto/ipc.proto`)

| Message Name | Purpose | Category |
|-------------|---------|----------|
| `IpcClientMessage` | Envelope for all client→server messages | Protocol |
| `IpcServerMessage` | Envelope for all server→client messages | Protocol |
| `IpcAuthHandshake` | Auth token submission | Auth |
| `IpcAuthHandshakeResponse` | Auth result | Auth |
| `IpcDiscoverServerRequest` | Request server discovery | Server Mgmt |
| `IpcDiscoverServerResponse` | Server discovery result | Server Mgmt |
| `IpcSpawnServerRequest` | Request server spawn | Server Mgmt |
| `IpcSpawnServerResponse` | Server spawn result | Server Mgmt |
| `IpcCheckHealthRequest` | Request health check | Server Mgmt |
| `IpcCheckHealthResponse` | Health check result | Server Mgmt |
| `IpcStopServerRequest` | Request server stop | Server Mgmt |
| `IpcStopServerResponse` | Server stop result | Server Mgmt |
| `IpcListSessionsRequest` | Request session list | Sessions |
| `IpcCreateSessionRequest` | Create new session | Sessions |
| `IpcDeleteSessionRequest` | Delete session | Sessions |
| `IpcListAgentsRequest` | Request agent list | Agents |
| `IpcGetProviderStatusRequest` | Request provider status | Providers |
| `IpcSetAuthRequest` | Set provider auth | Auth Ops |
| `IpcGetAuthRequest` | Get provider auth | Auth Ops |
| `IpcErrorResponse` | Error details | Errors |

**Total IPC messages: 20** (protocol layer only)

**Key imports in `ipc.proto`:**
```protobuf
import "oc_session.proto";  // For OcSessionList, OcSessionInfo
import "oc_agent.proto";    // For OcAgentList
import "oc_auth.proto";     // For OcAuth
import "oc_provider.proto"; // For OcProviderStatus
import "oc_server.proto";   // For OcServerInfo
```

### Field Names

**No field renames needed** - Current field names are contextual and unambiguous:
- `token`, `success`, `error` (auth context)
- `session_id`, `title`, `created_at`, `updated_at` (session context)
- `name`, `description`, `mode`, `built_in`, `color` (agent context)
- `provider_id`, `access_token`, `refresh_token`, `expires_at` (auth context)

**When would we rename?** Only if we encounter ambiguous names like:
- `url` → `ipc_url` or `opencode_url`
- `port` → `ipc_port` or `opencode_port`
- `client` → `ipc_client` or `opencode_client`

---

## 3. Auth Flow State Machine

### Architecture Decision

**Per ADR-0002 (Thin Tauri Layer):**
- ❌ Tauri does NOT generate or manage auth tokens
- ❌ Tauri does NOT validate auth
- ✅ client-core generates auth token on server start
- ✅ client-core validates auth handshake
- ✅ Blazor connects directly to client-core WebSocket (no Tauri intermediary)

**Why:** Auth is business logic, not webview hosting. It belongs in client-core.

### Token Generation

```rust
// backend/client-core/src/ipc/server.rs

pub async fn start_ipc_server(ipc_port: u16) -> Result<Handle, IpcError> {
    // Generate random auth token (per-instance)
    let auth_token = uuid::Uuid::new_v4().to_string();
    
    // TODO Session 8: Expose token to Blazor via config file or IPC query
    // For now: Log it so we can use it in tests
    log::info!("IPC server auth token: {}", auth_token);
    
    let address = format!("127.0.0.1:{}", ipc_port);
    let listener = TcpListener::bind(&address).await?;
    
    // Pass token to connection handler
    tokio::spawn(accept_loop(listener, auth_token));
    
    Ok(Handle {})
}
```

**Security note:** In Session 8, Blazor will query this token via a one-time mechanism (file, env var, or initial IPC call). For Session 6, we'll hardcode it in tests.

### Connection State Machine

```
┌─────────────────────────────────────────────────────────────┐
│ 1. App Startup (client-core)                               │
│    - Generate random auth token: uuid::Uuid::new_v4()      │
│    - Bind WebSocket server to 127.0.0.1:ipc_port           │
│    - Store token for validation                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Client Connects (WebSocket handshake)                   │
│    - TCP connection accepted                                │
│    - Check: addr.ip().is_loopback() (reject if not)       │
│    - WebSocket upgrade successful                           │
│    - Connection state: UNAUTHENTICATED                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. First Message MUST Be Auth                              │
│    - Receive binary WebSocket frame                         │
│    - Decode IpcClientMessage from bytes                    │
│    - Check oneof: MUST be auth_handshake                   │
│    - If NOT auth_handshake → log error, close connection   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Validate Token                                          │
│    - Extract token from IpcAuthHandshake                   │
│    - Compare with server's stored token                    │
│                                                             │
│    VALID TOKEN:                                             │
│    - log::info!("Client authenticated")                    │
│    - Send IpcAuthHandshakeResponse { success: true }       │
│    - Set state: AUTHENTICATED                              │
│    - Continue to main message loop                         │
│                                                             │
│    INVALID TOKEN:                                           │
│    - log::warn!("Auth failed: invalid token")              │
│    - Send IpcAuthHandshakeResponse {                       │
│        success: false,                                     │
│        error: "Invalid authentication token"               │
│      }                                                      │
│    - Close connection                                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Main Message Loop (only if authenticated)              │
│    - Receive binary frames                                 │
│    - Decode IpcClientMessage                               │
│    - log::debug!("Received: {:?}", message)                │
│    - Dispatch based on oneof payload                       │
│    - Send IpcServerMessage responses                       │
│    - Loop until disconnect                                  │
└─────────────────────────────────────────────────────────────┘
```

### Connection State

```rust
struct ConnectionState {
    authenticated: bool,
    expected_token: String,
}

impl ConnectionState {
    fn new(token: String) -> Self {
        Self {
            authenticated: false,
            expected_token: token,
        }
    }
    
    fn validate_token(&mut self, token: &str) -> bool {
        if token == self.expected_token {
            self.authenticated = true;
            true
        } else {
            false
        }
    }
    
    fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}
```

---

## 4. Error Handling

**Production-grade principle:** Every error must be actionable. Log context, propagate properly, never swallow.

### New Error Variants

```rust
// backend/client-core/src/error/ipc.rs

#[derive(Debug, ThisError)]
pub enum IpcError {
    // Existing variants
    #[error("Handshake Error: {message} {location}")]
    Handshake {
        message: String,
        location: ErrorLocation,
    },

    #[error("Send Error: {message} {location}")]
    Send {
        message: String,
        location: ErrorLocation,
    },

    #[error("Read Error: {message} {location}")]
    Read {
        message: String,
        location: ErrorLocation,
    },

    #[error("IO Error: {message} {location}")]
    Io {
        message: String,
        location: ErrorLocation,
    },
    
    // New variants for Session 6
    #[error("Auth Error: {message} {location}")]
    Auth {
        message: String,
        location: ErrorLocation,
    },
    
    #[error("Protobuf Decode Error: {message} {location}")]
    ProtobufDecode {
        message: String,
        location: ErrorLocation,
    },
    
    #[error("Protobuf Encode Error: {message} {location}")]
    ProtobufEncode {
        message: String,
        location: ErrorLocation,
    },
}

// Conversion from prost errors
impl From<prost::DecodeError> for IpcError {
    #[track_caller]
    fn from(error: prost::DecodeError) -> Self {
        IpcError::ProtobufDecode {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}

impl From<prost::EncodeError> for IpcError {
    #[track_caller]
    fn from(error: prost::EncodeError) -> Self {
        IpcError::ProtobufEncode {
            message: error.to_string(),
            location: ErrorLocation::from(Location::caller()),
        }
    }
}
```

### Error Scenarios

| Scenario | Detection | Response | Logging | Test Required |
|----------|-----------|----------|---------|---------------|
| **Non-loopback connection** | `!addr.ip().is_loopback()` | Drop connection immediately | `warn!("Rejected non-loopback connection from {}", addr)` | ✅ Yes |
| **Invalid token** | `token != expected_token` | Send error response, close | `warn!("Auth failed: invalid token from {}", addr)` | ✅ Yes |
| **Wrong first message** | First payload not `auth_handshake` | Close connection | `warn!("Auth failed: first message not auth handshake")` | ✅ Yes |
| **Message before auth** | `!authenticated` and not auth message | Close connection | `warn!("Rejected unauthenticated message")` | ✅ Yes |
| **Malformed protobuf** | `IpcClientMessage::decode()` fails | Close connection | `error!("Protobuf decode failed: {}", err)` | ✅ Yes |
| **WebSocket error** | `read.next()` returns `Err` | Close connection | `error!("WebSocket error: {}", err)` | Covered by existing tests |

---

## 5. Production-Grade Logging Strategy

### Log Levels

```rust
use log::{trace, debug, info, warn, error};

// TRACE: Not used (too verbose for production)

// DEBUG: Message-level details (helpful for development, disabled in prod)
debug!("Received IpcClientMessage: request_id={}", request_id);
debug!("Sending IpcServerMessage: request_id={}", request_id);

// INFO: Connection lifecycle and important events
info!("IPC server listening on {}", address);
info!("Client connected from {}", addr);
info!("Client authenticated successfully");
info!("Client disconnected: {}", addr);

// WARN: Recoverable errors and security events
warn!("Rejected non-loopback connection from {}", addr);
warn!("Auth failed: invalid token from {}", addr);
warn!("Auth failed: first message was not auth handshake");
warn!("Rejected unauthenticated message from {}", addr);

// ERROR: Unrecoverable errors
error!("WebSocket handshake failed: {}", err);
error!("Protobuf decode failed: {}", err);
error!("Failed to send message: {}", err);
```

### Structured Logging (Future Enhancement)

For Session 6, use simple `log` macros. In future sessions, consider:
- `tracing` crate for structured logging
- Span/context tracking across async boundaries
- Request ID correlation

---

## 6. Test Strategy

### Test Organization

```
backend/client-core/integration_tests/ipc/
├── mod.rs           # Test module declarations
├── ipc.rs           # Integration tests (currently 3 echo tests)
└── helpers.rs       # NEW: Test helpers and utilities
```

### Test Helpers

```rust
// integration_tests/ipc/helpers.rs

use client_core::proto::{IpcClientMessage, IpcServerMessage, IpcAuthHandshake};
use prost::Message as ProstMessage;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

/// Test helper: Connect to IPC server and return WebSocket stream
pub async fn connect_to_server(ipc_port: u16) -> WebSocketStream<tokio::net::TcpStream> {
    let url = format!("ws://127.0.0.1:{}", ipc_port);
    let (ws_stream, _) = connect_async(&url)
        .await
        .expect("Failed to connect to WebSocket server");
    ws_stream
}

/// Test helper: Send protobuf message over WebSocket
pub async fn send_protobuf<T: ProstMessage>(
    ws: &mut WebSocketStream<tokio::net::TcpStream>,
    message: &T,
) {
    let mut buf = Vec::new();
    message.encode(&mut buf).expect("Failed to encode protobuf");
    ws.send(Message::Binary(buf))
        .await
        .expect("Failed to send message");
}

/// Test helper: Receive and decode protobuf message
pub async fn receive_protobuf<T: ProstMessage + Default>(
    ws: &mut WebSocketStream<tokio::net::TcpStream>,
) -> T {
    let msg = ws.next().await
        .expect("No message received")
        .expect("Error receiving message");
    
    let bytes = msg.into_data();
    T::decode(&bytes[..]).expect("Failed to decode protobuf")
}

/// Test helper: Send auth handshake and return response
pub async fn authenticate(
    ws: &mut WebSocketStream<tokio::net::TcpStream>,
    token: &str,
) -> IpcAuthHandshakeResponse {
    let auth_msg = IpcClientMessage {
        request_id: 1,
        payload: Some(ipc_client_message::Payload::AuthHandshake(
            IpcAuthHandshake {
                token: token.to_string(),
            }
        )),
    };
    
    send_protobuf(ws, &auth_msg).await;
    
    let response: IpcServerMessage = receive_protobuf(ws).await;
    match response.payload {
        Some(ipc_server_message::Payload::AuthHandshakeResponse(resp)) => resp,
        _ => panic!("Expected AuthHandshakeResponse"),
    }
}

/// Test helper: Check if WebSocket connection is closed
pub async fn is_connection_closed(
    ws: &mut WebSocketStream<tokio::net::TcpStream>,
) -> bool {
    // Try to read with short timeout
    tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        ws.next()
    ).await.is_err() || matches!(
        ws.next().await,
        None | Some(Ok(Message::Close(_)))
    )
}

// Test constants
pub const TEST_AUTH_TOKEN: &str = "test-token-12345";
pub const INVALID_AUTH_TOKEN: &str = "wrong-token";
```

### Test Coverage

#### Existing Tests (Update for Protobuf)

```rust
// Update existing echo tests to use protobuf instead of text

#[tokio::test]
async fn given_authenticated_when_client_sends_message_then_receives_echo() {
    // GIVEN: IPC server with auth
    let ipc_port = 19876;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // Authenticate first
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success);
    
    // WHEN: Send test message (for Session 6, echo is fine)
    let test_msg = IpcClientMessage {
        request_id: 2,
        payload: Some(/* test payload */),
    };
    send_protobuf(&mut ws, &test_msg).await;
    
    // THEN: Receive response
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
}
```

#### New Auth Tests

```rust
/// Test: Valid auth token succeeds
#[tokio::test]
async fn given_valid_token_when_auth_handshake_then_success() {
    // GIVEN: IPC server with known token
    let ipc_port = 19880;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // WHEN: Send valid auth handshake
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // THEN: Success
    assert!(auth_response.success, "Auth should succeed with valid token");
    assert!(auth_response.error.is_none(), "No error should be present");
}

/// Test: Invalid auth token rejected
#[tokio::test]
async fn given_invalid_token_when_auth_handshake_then_rejected() {
    // GIVEN: IPC server with known token
    let ipc_port = 19881;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // WHEN: Send invalid auth handshake
    let auth_response = authenticate(&mut ws, INVALID_AUTH_TOKEN).await;
    
    // THEN: Failure
    assert!(!auth_response.success, "Auth should fail with invalid token");
    assert!(auth_response.error.is_some(), "Error message should be present");
    
    // AND: Connection should close
    assert!(is_connection_closed(&mut ws).await, "Connection should be closed");
}

/// Test: Wrong first message type rejected
#[tokio::test]
async fn given_non_auth_first_message_when_connect_then_rejected() {
    // GIVEN: IPC server running
    let ipc_port = 19882;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // WHEN: First message is NOT auth (e.g., ListSessionsRequest)
    let wrong_msg = IpcClientMessage {
        request_id: 1,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {}
        )),
    };
    send_protobuf(&mut ws, &wrong_msg).await;
    
    // THEN: Connection closed (no response expected)
    assert!(
        is_connection_closed(&mut ws).await,
        "Connection should close when first message is not auth"
    );
}

/// Test: Message sent before auth is rejected
#[tokio::test]
async fn given_unauthenticated_when_send_non_auth_message_then_rejected() {
    // GIVEN: Connected but not authenticated
    let ipc_port = 19883;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // WHEN: Send non-auth message without authenticating first
    let msg = IpcClientMessage {
        request_id: 1,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {}
        )),
    };
    send_protobuf(&mut ws, &msg).await;
    
    // THEN: Connection closed
    assert!(
        is_connection_closed(&mut ws).await,
        "Connection should close for unauthenticated non-auth message"
    );
}

/// Test: After successful auth, normal messages accepted
#[tokio::test]
async fn given_authenticated_when_send_message_then_accepted() {
    // GIVEN: Authenticated connection
    let ipc_port = 19884;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // Authenticate
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success, "Auth should succeed");
    
    // WHEN: Send normal message
    let msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::ListSessions(
            IpcListSessionsRequest {}
        )),
    };
    send_protobuf(&mut ws, &msg).await;
    
    // THEN: Receive response (for Session 6, even if it's an error, connection stays open)
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2, "Should receive response with matching request_id");
}

/// Test: Malformed protobuf closes connection
#[tokio::test]
async fn given_authenticated_when_send_malformed_protobuf_then_connection_closed() {
    // GIVEN: Authenticated connection
    let ipc_port = 19885;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    
    // Authenticate
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success);
    
    // WHEN: Send garbage bytes (malformed protobuf)
    let garbage = vec![0xFF, 0xFF, 0xFF, 0xFF];
    ws.send(Message::Binary(garbage))
        .await
        .expect("Failed to send garbage");
    
    // THEN: Connection closes (server detects decode error)
    assert!(
        is_connection_closed(&mut ws).await,
        "Connection should close on malformed protobuf"
    );
}
```

#### New Server Management Tests

```rust
/// Test: Discover server through IPC
#[tokio::test]
async fn given_authenticated_when_discover_server_then_returns_info() {
    // GIVEN: Authenticated connection + running OpenCode server
    let ipc_port = 19886;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    let auth_response = authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    assert!(auth_response.success);
    
    // WHEN: Request server discovery
    let discover_msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::DiscoverServer(
            IpcDiscoverServerRequest {}
        )),
    };
    send_protobuf(&mut ws, &discover_msg).await;
    
    // THEN: Receive server info (or None if no server running)
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    // Note: Actual server info depends on test environment
}

/// Test: Spawn server through IPC
#[tokio::test]
async fn given_authenticated_when_spawn_server_then_returns_info() {
    // GIVEN: Authenticated connection
    let ipc_port = 19887;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // WHEN: Request server spawn
    let spawn_msg = IpcClientMessage {
        request_id: 2,
        payload: Some(ipc_client_message::Payload::SpawnServer(
            IpcSpawnServerRequest {}
        )),
    };
    send_protobuf(&mut ws, &spawn_msg).await;
    
    // THEN: Receive spawned server info
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 2);
    
    match response.payload {
        Some(ipc_server_message::Payload::SpawnServerResponse(resp)) => {
            assert!(resp.server.is_some(), "Should return server info");
        }
        _ => panic!("Expected SpawnServerResponse"),
    }
}

/// Test: Health check through IPC
#[tokio::test]
async fn given_authenticated_when_check_health_then_returns_status() {
    // GIVEN: Authenticated connection + server in state
    let ipc_port = 19888;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // Discover or spawn server first
    // ... (abbreviated for brevity)
    
    // WHEN: Request health check
    let health_msg = IpcClientMessage {
        request_id: 3,
        payload: Some(ipc_client_message::Payload::CheckHealth(
            IpcCheckHealthRequest {}
        )),
    };
    send_protobuf(&mut ws, &health_msg).await;
    
    // THEN: Receive health status
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 3);
}

/// Test: Stop server through IPC
#[tokio::test]
async fn given_authenticated_when_stop_server_then_succeeds() {
    // GIVEN: Authenticated connection + running server
    let ipc_port = 19889;
    let _handle = start_ipc_server_with_auth(ipc_port, TEST_AUTH_TOKEN)
        .await
        .expect("Failed to start IPC server");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut ws = connect_to_server(ipc_port).await;
    authenticate(&mut ws, TEST_AUTH_TOKEN).await;
    
    // Spawn server first
    // ... (abbreviated for brevity)
    
    // WHEN: Request server stop
    let stop_msg = IpcClientMessage {
        request_id: 4,
        payload: Some(ipc_client_message::Payload::StopServer(
            IpcStopServerRequest {}
        )),
    };
    send_protobuf(&mut ws, &stop_msg).await;
    
    // THEN: Receive success response
    let response: IpcServerMessage = receive_protobuf(&mut ws).await;
    assert_eq!(response.request_id, 4);
}
```

### Test Summary

| Test | Purpose | Bug Caught |
|------|---------|------------|
| **Auth Tests (6)** |
| `given_valid_token_when_auth_handshake_then_success` | Happy path auth | Auth flow broken |
| `given_invalid_token_when_auth_handshake_then_rejected` | Security: reject bad tokens | Auth bypass |
| `given_non_auth_first_message_when_connect_then_rejected` | Security: enforce auth-first | Auth bypass |
| `given_unauthenticated_when_send_non_auth_message_then_rejected` | Security: no messages before auth | Auth bypass |
| `given_authenticated_when_send_message_then_accepted` | Auth allows normal flow | Auth blocks valid traffic |
| `given_malformed_protobuf_then_connection_closed` | Robustness: handle bad data | Crash on invalid input |
| **Server Management Tests (4)** |
| `given_authenticated_when_discover_server_then_returns_info` | Server discovery via IPC | Discovery broken |
| `given_authenticated_when_spawn_server_then_returns_info` | Server spawn via IPC | Spawn broken |
| `given_authenticated_when_check_health_then_returns_status` | Health check via IPC | Health check broken |
| `given_authenticated_when_stop_server_then_succeeds` | Stop server via IPC | Stop broken |

**Total tests:** 6 auth tests + 4 server mgmt tests + 3 updated echo tests = **13 integration tests**

---

## 7. Implementation Steps

### ⚠️ BEFORE YOU START

**Remember the constraints:**
1. **Teaching mode:** Do NOT write files unless user explicitly says "implement" or "write the code"
2. **Production-grade:** No shortcuts, no TODOs, no "we'll fix later"
3. **Step-by-step:** Present one step at a time, get user confirmation
4. **Explain first:** Show code snippets, explain why, then ask permission

**How to proceed:**
- Read this plan thoroughly first
- Ask user: "Should I explain Step 1, or implement it?"
- If explain: Show snippet, explain trade-offs, ask for confirmation
- If implement: Only then use write/edit tools
- After each step: Verify, test, show user the results

---

### Step 1: Create All OpenCode Proto Files (45 min)

**Goal:** Implement complete, canonical OpenCode data models (1:1 with JSON Schemas)

**Teaching approach:**
- Show example from one proto file (e.g., `oc_session.proto`)
- Explain how to verify against JSON Schema
- Ask user if they want you to implement all 9 files
- If yes, proceed with write operations

**Sub-steps:**

#### 1a. Create `proto/oc_model.proto` (5 min)
**Source:** `docs/proto/01-model.md`

**Actions:**
1. Read entire `docs/proto/01-model.md`
2. Copy protobuf definitions from "Messages" section (lines ~40-200)
3. Verify against JSON Schema cross-reference table
4. Add source comments linking to JSON Schema files
5. No modifications - copy verbatim

**Expected messages:**
- `OcModelInfo`
- `OcModelCapabilities`
- `OcModelLimits`
- `OcModelCost`
- `OcModelSelection`
- And supporting types (~10 total)

#### 1b. Create `proto/oc_provider.proto` (5 min)
**Source:** `docs/proto/02-provider.md`

**Actions:**
1. Read entire `docs/proto/02-provider.md`
2. Copy protobuf definitions (lines ~29-89)
3. Verify against JSON Schema cross-reference
4. Note: Imports `oc_model.proto` for ModelInfo

**Expected messages:**
- `OcProviderInfo`
- `OcProviderOptions`
- `OcProviderSource` (enum)
- `OcProviderList`

#### 1c. Create `proto/oc_auth.proto` (5 min)
**Source:** `docs/proto/03-auth.md`

**Actions:**
1. Read entire `docs/proto/03-auth.md`
2. Copy protobuf definitions (lines ~31-71)
3. Verify discriminated union structure (oneof with type discriminators)

**Expected messages:**
- `OcAuth` (discriminated union)
- `OcOAuth`
- `OcApiAuth`
- `OcWellKnownAuth`

#### 1d. Create `proto/oc_session.proto` (5 min)
**Source:** `docs/proto/04-session.md`

**Actions:**
1. Read entire `docs/proto/04-session.md`
2. Copy protobuf definitions (lines ~39-150)
3. Verify nested types (SessionTime, SessionSummary, FileDiff, etc.)

**Expected messages:**
- `OcSessionInfo`
- `OcSessionTime`
- `OcSessionSummary`
- `OcSessionShare`
- `OcSessionRevert`
- `OcFileDiff`
- `OcPermissionRule`
- `OcPermissionRuleset`
- `OcPermissionAction` (enum)
- `OcSessionList`

#### 1e. Create `proto/oc_message.proto` (5 min)
**Source:** `docs/proto/05-message.md`

**Actions:**
1. Read entire `docs/proto/05-message.md`
2. Copy protobuf definitions (large file - 20+ messages)
3. Note: Will be used in Session 13+ (chat)

**Expected messages:**
- `OcMessage` (discriminated union)
- `OcUserMessage`
- `OcAssistantMessage`
- `OcPart` variants (TextPart, ReasoningPart, etc.)
- `OcMessageError` types
- 20+ messages total

#### 1f. Create `proto/oc_tool.proto` (5 min)
**Source:** `docs/proto/06-tool.md`

**Actions:**
1. Read entire `docs/proto/06-tool.md`
2. Copy protobuf definitions
3. Note: Used in Session 20+ (tools)

**Expected messages:**
- `OcToolState` (discriminated union)
- `OcToolStatePending`
- `OcToolStateRunning`
- `OcToolStateCompleted`
- `OcToolStateError`
- `OcToolPart`
- `OcPermissionRequest`
- `OcPermissionReply`

#### 1g. Create `proto/oc_agent.proto` (5 min)
**Source:** `docs/proto/07-agent.md`

**Actions:**
1. Read entire `docs/proto/07-agent.md`
2. Copy protobuf definitions (lines ~43-79)
3. Verify field count: 13 fields in AgentInfo

**Expected messages:**
- `OcAgentInfo`
- `OcAgentModel`
- `OcAgentList`

#### 1h. Create `proto/oc_event.proto` (5 min)
**Source:** `docs/proto/08-event.md`

**Actions:**
1. Read entire `docs/proto/08-event.md`
2. Copy protobuf definitions (13+ event types)
3. Note: Used in Session 15+ (SSE streaming)

**Expected messages:**
- `OcEvent` (discriminated union)
- `OcGlobalEvent`
- Event types (MessageUpdated, SessionCreated, PermissionAsked, etc.)
- 13+ messages total

#### 1i. Create `proto/oc_server.proto` (5 min)
**NEW** - Not in research (client-core specific)

**Actions:**
1. Define `OcServerInfo` for IPC server discovery/spawn
2. This is NOT an OpenCode server model (it's about the TypeScript OpenCode server process)
3. Fields: `pid`, `port`, `base_url`, `owned`

**Expected messages:**
```protobuf
syntax = "proto3";
package opencode.server;

// Information about a discovered or spawned OpenCode server process
// Note: This models the TypeScript OpenCode server (not protobuf/gRPC)
message OcServerInfo {
  int32 pid = 1;           // Process ID
  int32 port = 2;          // HTTP port (e.g., 4008)
  string base_url = 3;     // e.g., "http://127.0.0.1:4008"
  bool owned = 4;          // Whether this process was spawned by us
}
```

#### 1j. Update `proto/ipc.proto` (5 min)

**Actions:**
1. Remove old simplified models (SessionInfo, AgentInfo, AuthInfo, etc.)
2. Add imports for OpenCode protos:
   ```protobuf
   import "oc_session.proto";
   import "oc_agent.proto";
   import "oc_auth.proto";
   import "oc_provider.proto";
   import "oc_server.proto";
   ```
3. Update `IpcServerMessage` to use fully-qualified types:
   ```protobuf
   oneof payload {
     opencode.session.OcSessionList session_list = 20;
     opencode.agent.OcAgentList agent_list = 30;
     // etc.
   }
   ```
4. Add server management request/response messages
5. Rename all messages with `Ipc*` prefix

**New proto definitions:**

```protobuf
// Server Management (ADD TO proto/ipc.proto)
message IpcDiscoverServerRequest {}

message IpcDiscoverServerResponse {
  optional OcServerInfo server = 1;
}

message IpcSpawnServerRequest {}

message IpcSpawnServerResponse {
  OcServerInfo server = 1;
}

message IpcCheckHealthRequest {}

message IpcCheckHealthResponse {
  bool healthy = 1;
}

message IpcStopServerRequest {}

message IpcStopServerResponse {}

// OpenCode Server Info (data model)
message OcServerInfo {
  int32 pid = 1;
  int32 port = 2;
  string base_url = 3;
  bool owned = 4;
}
```

**Update envelopes:**

```protobuf
message IpcClientMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Auth
    IpcAuthHandshake auth_handshake = 10;
    
    // Server Management (NEW)
    IpcDiscoverServerRequest discover_server = 15;
    IpcSpawnServerRequest spawn_server = 16;
    IpcCheckHealthRequest check_health = 17;
    IpcStopServerRequest stop_server = 18;
    
    // Sessions
    IpcListSessionsRequest list_sessions = 20;
    IpcCreateSessionRequest create_session = 21;
    IpcDeleteSessionRequest delete_session = 22;
    
    // ... rest unchanged
  }
}

message IpcServerMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Auth
    IpcAuthHandshakeResponse auth_handshake_response = 10;
    
    // Server Management (NEW)
    IpcDiscoverServerResponse discover_server_response = 15;
    IpcSpawnServerResponse spawn_server_response = 16;
    IpcCheckHealthResponse check_health_response = 17;
    IpcStopServerResponse stop_server_response = 18;
    
    // Sessions
    OcSessionList session_list = 20;
    OcSessionInfo session_info = 21;
    
    // ... rest unchanged
  }
}
```

**Verification:**
```bash
cargo build -p client-core
# Should succeed with new generated code in OUT_DIR/
# Generated files: opencode.model.rs, opencode.session.rs, etc.
```

**Expected:** 
- 9 new proto files created
- Build generates separate Rust modules per proto
- Tests will fail to compile (they reference old names), but build succeeds
- ~74 OpenCode messages + 20 IPC messages = 94 total protobuf messages

---

### Step 2: Move Code to Correct File (5 min)

**Actions:**
1. Copy all code from `backend/client-core/src/ipc/handle.rs` to `server.rs`
2. Update `mod.rs`:
   ```rust
   mod server;
   
   pub use server::{Handle, start_ipc_server};
   ```
3. Delete `handle.rs`

**Verification:**
```bash
cargo build -p client-core
# Should succeed
```

---

### Step 3: Update Error Types (5 min)

**File:** `backend/client-core/src/error/ipc.rs`

**Actions:**
1. Add `Auth`, `ProtobufDecode`, `ProtobufEncode` variants
2. Add `From<prost::DecodeError>` impl
3. Add `From<prost::EncodeError>` impl
4. Fix line 22 typo: `"Send Error"` → `"Read Error"` for `Read` variant

**Verification:**
```bash
cargo build -p client-core
# Should succeed
```

---

### Step 4: Move State Management to client-core (20 min)

**Source:** `apps/desktop/opencode/src/state.rs`  
**Destination:** `backend/client-core/src/ipc/state.rs`

**Actions:**
1. Copy `state.rs` from Tauri layer to client-core IPC module
2. Rename `AppState` → `IpcState` (more specific name)
3. Keep the actor pattern (proven to work)
4. Update `backend/client-core/src/ipc/mod.rs`:
   ```rust
   mod handle;
   mod server;
   mod state;  // NEW
   
   pub use server::{Handle, start_ipc_server};
   pub use state::IpcState;  // NEW
   ```
5. Update imports in `server.rs` to use `IpcState`

**Rationale:**
- State management is business logic (where is the server, what's its PID)
- Per ADR-0002: Business logic belongs in client-core, not Tauri
- Actor pattern prevents race conditions across concurrent IPC requests
- Tauri layer should NOT own or manage application state

**Verification:**
```bash
cargo build -p client-core
# Should succeed
```

---

### Step 5: Create Test Helpers (15 min)

**File:** `backend/client-core/integration_tests/ipc/helpers.rs` (NEW)

**Actions:**
1. Create helper functions from section 6
2. Add test constants
3. Export from `integration_tests/ipc/mod.rs`

**Verification:**
```bash
cargo test -p client-core --test integration --no-run
# Should compile
```

---

### Step 6: Implement Auth State Machine (30 min)

**File:** `backend/client-core/src/ipc/server.rs`

**Actions:**
1. Add `ConnectionState` struct
2. Update `start_ipc_server()` signature: add `auth_token: &str` parameter
3. Generate UUID token if not provided (for backward compat)
4. Implement first-message validation
5. Implement token validation
6. Send `IpcAuthHandshakeResponse`
7. Add logging at all state transitions

**Key code sections:**

```rust
// Token generation
pub async fn start_ipc_server(ipc_port: u16, auth_token: Option<String>) -> Result<Handle, IpcError> {
    let auth_token = auth_token.unwrap_or_else(|| {
        let token = uuid::Uuid::new_v4().to_string();
        log::info!("Generated IPC auth token: {}", token);
        token
    });
    
    // ... rest of setup
}

// Connection state
struct ConnectionState {
    authenticated: bool,
    expected_token: String,
}

// First message validation
async fn handle_connection(stream: TcpStream, auth_token: String) -> Result<(), IpcError> {
    let addr = stream.peer_addr()?;
    
    // Security: Reject non-loopback
    if !addr.ip().is_loopback() {
        log::warn!("Rejected non-loopback connection from {}", addr);
        return Ok(());
    }
    
    log::info!("Client connected from {}", addr);
    
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    
    let mut state = ConnectionState::new(auth_token);
    
    // First message MUST be auth
    if let Some(Ok(Message::Binary(data))) = read.next().await {
        let client_msg = IpcClientMessage::decode(&data[..])?;
        
        match client_msg.payload {
            Some(ipc_client_message::Payload::AuthHandshake(auth)) => {
                if state.validate_token(&auth.token) {
                    log::info!("Client authenticated successfully");
                    send_auth_response(&mut write, true, None).await?;
                } else {
                    log::warn!("Auth failed: invalid token");
                    send_auth_response(&mut write, false, Some("Invalid token")).await?;
                    return Ok(());
                }
            }
            _ => {
                log::warn!("Auth failed: first message was not auth handshake");
                return Ok(());
            }
        }
    } else {
        log::warn!("Auth failed: no message received or not binary");
        return Ok(());
    }
    
    // Main message loop (authenticated)
    while let Some(msg) = read.next().await {
        // ... handle messages
    }
    
    log::info!("Client disconnected: {}", addr);
    Ok(())
}
```

**Verification:**
```bash
cargo build -p client-core
# Should succeed
```

---

### Step 7: Switch to Binary Protobuf + Add Server Handlers (40 min)

**File:** `backend/client-core/src/ipc/server.rs`

**Actions:**
1. Replace text echo logic with protobuf decode/encode
2. Handle `IpcClientMessage` envelope
3. Respond with `IpcServerMessage` envelope
4. **NEW:** Implement server management handlers:
   - `handle_discover_server()` - calls `client_core::discovery::process::discover()`
   - `handle_spawn_server()` - calls `client_core::discovery::spawn::spawn_and_wait()`
   - `handle_check_health()` - calls `client_core::discovery::process::check_health()`
   - `handle_stop_server()` - calls `client_core::discovery::process::stop_pid()`
5. Wire `IpcState` into handlers (update state on discover/spawn, read state for health/stop)
6. For Session 7: Stub handlers for sessions (return empty list or NOT_IMPLEMENTED error)

**Key code:**

```rust
// Main message loop
while let Some(msg) = read.next().await {
    match msg {
        Ok(Message::Binary(data)) => {
            match IpcClientMessage::decode(&data[..]) {
                Ok(client_msg) => {
                    log::debug!("Received message: request_id={}", client_msg.request_id);
                    
                    // For Session 6: Simple echo or stub response
                    let response = handle_message(client_msg).await;
                    
                    let mut buf = Vec::new();
                    response.encode(&mut buf)?;
                    
                    write.send(Message::Binary(buf)).await?;
                }
                Err(e) => {
                    log::error!("Protobuf decode failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(Message::Close(_)) => {
            log::debug!("Client sent close frame");
            break;
        }
        Ok(_) => {
            log::warn!("Received non-binary message, ignoring");
        }
        Err(e) => {
            log::error!("WebSocket error: {}", e);
            return Err(IpcError::Read {
                message: e.to_string(),
                location: ErrorLocation::from(Location::caller()),
            });
        }
    }
}

// Message handler with server management + session stubs
async fn handle_message(msg: IpcClientMessage, state: &IpcState) -> IpcServerMessage {
    let response_payload = match msg.payload {
        // Server Management (NEW - Session 6)
        Some(ipc_client_message::Payload::DiscoverServer(_)) => {
            handle_discover_server(state).await
        }
        Some(ipc_client_message::Payload::SpawnServer(_)) => {
            handle_spawn_server(state).await
        }
        Some(ipc_client_message::Payload::CheckHealth(_)) => {
            handle_check_health(state).await
        }
        Some(ipc_client_message::Payload::StopServer(_)) => {
            handle_stop_server(state).await
        }
        
        // Sessions (Session 7 - stubs for now)
        Some(ipc_client_message::Payload::ListSessions(_)) => {
            Some(ipc_server_message::Payload::SessionList(OcSessionList {
                sessions: vec![],
            }))
        }
        
        // Default: Not implemented
        _ => {
            Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "Handler not implemented yet".to_string(),
            }))
        }
    };
    
    IpcServerMessage {
        request_id: msg.request_id,
        payload: response_payload,
    }
}

// Server management handlers
async fn handle_discover_server(state: &IpcState) -> Option<ipc_server_message::Payload> {
    match client_core::discovery::process::discover() {
        Ok(Some(server_info)) => {
            // Update state
            let _ = state.update(StateCommand::SetServer(server_info.clone())).await;
            
            Some(ipc_server_message::Payload::DiscoverServerResponse(
                IpcDiscoverServerResponse {
                    server: Some(convert_server_info_to_proto(server_info)),
                }
            ))
        }
        Ok(None) => {
            Some(ipc_server_message::Payload::DiscoverServerResponse(
                IpcDiscoverServerResponse { server: None }
            ))
        }
        Err(e) => {
            log::error!("Server discovery failed: {}", e);
            Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                code: "DISCOVERY_FAILED".to_string(),
                message: e.to_string(),
            }))
        }
    }
}

async fn handle_spawn_server(state: &IpcState) -> Option<ipc_server_message::Payload> {
    match client_core::discovery::spawn::spawn_and_wait().await {
        Ok(server_info) => {
            // Update state
            let _ = state.update(StateCommand::SetServer(server_info.clone())).await;
            
            Some(ipc_server_message::Payload::SpawnServerResponse(
                IpcSpawnServerResponse {
                    server: Some(convert_server_info_to_proto(server_info)),
                }
            ))
        }
        Err(e) => {
            log::error!("Server spawn failed: {}", e);
            Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                code: "SPAWN_FAILED".to_string(),
                message: e.to_string(),
            }))
        }
    }
}

async fn handle_check_health(state: &IpcState) -> Option<ipc_server_message::Payload> {
    match state.get_server().await {
        Some(server_info) => {
            let healthy = client_core::discovery::process::check_health(&server_info.base_url).await;
            Some(ipc_server_message::Payload::CheckHealthResponse(
                IpcCheckHealthResponse { healthy }
            ))
        }
        None => {
            Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                code: "NO_SERVER".to_string(),
                message: "No server connected".to_string(),
            }))
        }
    }
}

async fn handle_stop_server(state: &IpcState) -> Option<ipc_server_message::Payload> {
    match state.get_server().await {
        Some(server_info) => {
            let stopped = client_core::discovery::process::stop_pid(server_info.pid);
            if stopped {
                let _ = state.update(StateCommand::ClearServer).await;
                Some(ipc_server_message::Payload::StopServerResponse(
                    IpcStopServerResponse {}
                ))
            } else {
                Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                    code: "STOP_FAILED".to_string(),
                    message: format!("Failed to stop server PID {}", server_info.pid),
                }))
            }
        }
        None => {
            Some(ipc_server_message::Payload::Error(IpcErrorResponse {
                code: "NO_SERVER".to_string(),
                message: "No server connected".to_string(),
            }))
        }
    }
}

// Convert models::ServerInfo to OcServerInfo proto
fn convert_server_info_to_proto(info: models::ServerInfo) -> OcServerInfo {
    OcServerInfo {
        pid: info.pid,
        port: info.port as i32,
        base_url: info.base_url,
        owned: info.owned,
    }
}
```

**Verification:**
```bash
cargo build -p client-core
# Should succeed
```

---

### Step 8: Update Tests (40 min)

**Files:**
- `backend/client-core/integration_tests/ipc/ipc.rs`
- `backend/client-core/integration_tests/ipc/helpers.rs`

**Actions:**
1. Add `mod helpers;` and `use helpers::*;` to `ipc.rs`
2. Update 3 existing echo tests to use protobuf
3. Add 6 new auth tests from section 6
4. **NEW:** Add 4 server management tests
5. Update `start_ipc_server()` calls to include auth token

**Verification:**
```bash
cargo test -p client-core --test integration
# All 13 tests should pass (3 echo + 6 auth + 4 server mgmt)
```

---

### Step 9: Update Tauri Main (5 min)

**File:** `apps/desktop/opencode/src/main.rs`

**Actions:**
1. Start IPC server in `.setup()` handler
2. Comment out Tauri command registration with deprecation note
3. Add TODO for Session 8 (remove Tauri commands entirely)

**Code:**

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // DEPRECATED - These commands are replaced by IPC server (ADR-0002)
            // Remove in Session 8 after Blazor migrates to WebSocket
            // commands::server::discover_server,
            // commands::server::spawn_server,
            // commands::server::check_health,
            // commands::server::stop_server,
        ])
        .setup(|app| {
            // Get app data directory for logs
            let log_dir = app.path().app_log_dir()
                .map_err(|e| OpencodeError::Opencode {
                    message: format!("Failed to get log directory: {}", e),
                    location: ErrorLocation::from(Location::caller()),
                })?;

            // Ensure log directory exists
            create_dir_all(&log_dir).map_err(|e| OpencodeError::Opencode {
                message: format!("Failed to create log directory: {}", e),
                location: ErrorLocation::from(Location::caller()),
            })?;

            // Initialize logger
            LoggerInitialize(&log_dir)?;
            info!("OpenCode Tauri application starting");
            info!("Log directory: {}", log_dir.display());

            // Start IPC server (NEW - Session 6)
            let ipc_port = 19876; // TODO: Make configurable via env/config
            tokio::spawn(async move {
                info!("Starting IPC server on port {}", ipc_port);
                if let Err(e) = client_core::ipc::start_ipc_server(ipc_port, None).await {
                    error!("IPC server failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Verification:**
```bash
cargo build -p opencode
# Should succeed with warnings about unused Tauri commands (expected)
```

---

### Step 10: Cleanup (10 min)

**Actions:**
1. Remove any commented-out code
2. Run `cargo fmt`
3. Run `cargo clippy -- -D warnings`
4. Verify all logging statements are appropriate levels
5. Double-check naming conventions

**Verification:**
```bash
cargo fmt
cargo clippy -p client-core -- -D warnings
cargo test -p client-core --test integration
# All checks pass, all tests pass
```

---

## 8. Success Criteria

**Note:** These criteria define "done" for Session 6. All items must be verified before proceeding to Session 7.

### Proto Files ✅ COMPLETE
- [x] **11 OpenCode proto files created** (10 `oc_*.proto` + 1 `ipc.proto`) - **DONE**
- [x] **All OpenCode messages verified** against `docs/proto/*.md` and JSON Schemas - **DONE**
- [x] **~94 total messages** (74 OpenCode + 20 IPC) - **DONE**
- [x] **Proper `Ipc*` / `Oc*` prefixes** applied - **DONE**
- [x] No ambiguous field names (verified: none exist) - **DONE**
- [x] `cargo build -p client-core` succeeds (11 proto files compile) - **DONE**
- [x] **`build.rs` configured** with all proto files + serde for IpcServerInfo - **DONE**
- [x] **Models crate refactored** (`models` → `common`, `ServerInfo` → `IpcServerInfo`) - **DONE**
- [x] **All tests passing** (25 tests: 7 unit + 18 integration) - **DONE**

### Code Structure
- [ ] Module structure cleaned up (`server.rs` has code, `handle.rs` deleted, `state.rs` moved from Tauri)
- [ ] State management moved to client-core (`IpcState`)
- [ ] IPC server starts in Tauri main.rs
- [ ] Tauri commands deprecated with comments

### IPC Functionality
- [ ] Auth handshake accepts valid token
- [ ] Auth handshake rejects invalid token
- [ ] Auth handshake rejects wrong first message type
- [ ] Binary protobuf messages parse correctly
- [ ] **Server discovery works through IPC** (NEW)
- [ ] **Server spawn works through IPC** (NEW)
- [ ] **Health check works through IPC** (NEW)
- [ ] **Stop server works through IPC** (NEW)

### Testing & Quality
- [ ] `cargo test -p client-core --test integration` passes (**13 tests** total: 3 echo + 6 auth + 4 server mgmt)
- [ ] `cargo clippy` passes with no warnings
- [ ] Production-grade logging at all state transitions

### Documentation
- [ ] Source comments in all proto files link to JSON Schema sources
- [ ] Cross-reference tables verified (field count, type matching)

---

## 9. Out of Scope for Session 6

**NOT implementing:**
- OpenCode session handlers (Session 7 - list/create/delete sessions)
- C# IPC client (Session 8 - Blazor WebSocket client)
- Complete Blazor migration from Tauri invoke to WebSocket (Session 8)
- Token exchange mechanism for Blazor (Session 8 - how Blazor gets the auth token)
- Request multiplexing with correlation (Session 7+ - concurrent requests)
- Streaming chat messages (Session 7+ - SSE → IPC forwarding)
- Removing deprecated Tauri commands (Session 8)

**Focus:** Auth handshake + protobuf framing + server management migration from Tauri to client-core.

---

## 10. Dependencies & Build Configuration

### Cargo.toml Additions

Verify these are already present in `backend/client-core/Cargo.toml`:

```toml
[dependencies]
# WebSocket
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"

# Protobuf
prost = "0.12"

# Logging
log = "0.4"

# Error handling
thiserror = "1.0"

# UUID for token generation
uuid = { version = "1.0", features = ["v4"] }

# Models (existing)
models = { path = "../../models" }

[build-dependencies]
prost-build = "0.12"
```

### build.rs Configuration

Update `backend/client-core/build.rs` to compile all proto files:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile all OpenCode proto files
    prost_build::compile_protos(
        &[
            "../../proto/oc_model.proto",
            "../../proto/oc_provider.proto",
            "../../proto/oc_auth.proto",
            "../../proto/oc_session.proto",
            "../../proto/oc_message.proto",
            "../../proto/oc_tool.proto",
            "../../proto/oc_agent.proto",
            "../../proto/oc_event.proto",
            "../../proto/oc_server.proto",
            "../../proto/ipc.proto",
        ],
        &["../../proto/"],
    )?;
    Ok(())
}
```

### Generated Code Organization

After `cargo build`, protobuf generates Rust code in `OUT_DIR`:

```
target/debug/build/client-core-XXXXX/out/
├── opencode.model.rs         # From oc_model.proto
├── opencode.provider.rs      # From oc_provider.proto
├── opencode.auth.rs          # From oc_auth.proto
├── opencode.session.rs       # From oc_session.proto
├── opencode.message.rs       # From oc_message.proto
├── opencode.tool.rs          # From oc_tool.proto
├── opencode.agent.rs         # From oc_agent.proto
├── opencode.event.rs         # From oc_event.proto
├── opencode.server.rs        # From oc_server.proto
└── opencode.ipc.rs           # From ipc.proto
```

### Import in lib.rs

Update `backend/client-core/src/lib.rs`:

```rust
pub mod discovery;
pub mod error;
pub mod ipc;

// Protobuf generated code
pub mod proto {
    // OpenCode models
    pub mod model {
        include!(concat!(env!("OUT_DIR"), "/opencode.model.rs"));
    }
    pub mod provider {
        include!(concat!(env!("OUT_DIR"), "/opencode.provider.rs"));
    }
    pub mod auth {
        include!(concat!(env!("OUT_DIR"), "/opencode.auth.rs"));
    }
    pub mod session {
        include!(concat!(env!("OUT_DIR"), "/opencode.session.rs"));
    }
    pub mod message {
        include!(concat!(env!("OUT_DIR"), "/opencode.message.rs"));
    }
    pub mod tool {
        include!(concat!(env!("OUT_DIR"), "/opencode.tool.rs"));
    }
    pub mod agent {
        include!(concat!(env!("OUT_DIR"), "/opencode.agent.rs"));
    }
    pub mod event {
        include!(concat!(env!("OUT_DIR"), "/opencode.event.rs"));
    }
    pub mod server {
        include!(concat!(env!("OUT_DIR"), "/opencode.server.rs"));
    }
    
    // IPC protocol
    pub mod ipc {
        include!(concat!(env!("OUT_DIR"), "/opencode.ipc.rs"));
    }
}

#[cfg(test)]
mod tests;

pub const OPENCODE_BINARY: &str = "opencode";
pub const OPENCODE_SERVER_HOSTNAME: &str = "127.0.0.1";
pub const OPENCODE_SERVER_BASE_URL: &str =
    const_format::concatcp!("http://", OPENCODE_SERVER_HOSTNAME);
```

**Verification:**
```bash
cargo tree -p client-core | grep -E "(prost|uuid|tokio-tungstenite)"
# Should show all dependencies

cargo build -p client-core
# Should compile all 10 proto files and generate Rust modules
```

---

## 11. Reference Patterns

### Discovery Module (Established Pattern)

```
backend/client-core/src/discovery/
├── mod.rs          # Public exports
├── process.rs      # Process discovery implementation
└── spawn.rs        # Server spawning implementation
```

**Apply to IPC:**
```
backend/client-core/src/ipc/
├── mod.rs          # Public exports
└── server.rs       # WebSocket server implementation
```

### Error Module (Established Pattern)

```rust
// models/src/error/mod.rs
pub mod error_location;
pub mod model_error;

// backend/client-core/src/error/mod.rs
pub mod discovery;
pub mod ipc;
```

**Apply to IPC errors:** Follow same pattern with `ErrorLocation` and `#[track_caller]`.

---

## 12. Questions Resolved

### Q1: Token Generation Location
**A:** Per ADR-0002, client-core generates the token. Tauri does NOT participate in auth logic.

### Q2: Test Helpers
**A:** Use production-grade helpers with constants. Consider mocking frameworks where appropriate (Session 7+).

### Q3: Logging Level
**A:** Production-grade logging strategy:
- INFO: Connection lifecycle
- WARN: Security events (rejected auth, non-loopback connections)
- ERROR: Unrecoverable errors
- DEBUG: Message-level details (disabled in prod)

### Q4: Server Management Migration (NEW)
**Q:** Should server management (discover/spawn/health/stop) be migrated from Tauri to client-core in Session 6?

**A:** YES - Include in Session 6. Rationale:
- Current architecture violates ADR-0002 (Blazor → Tauri → client-core is wrong)
- Target architecture per ADR-0003 is Blazor → WebSocket → client-core (direct)
- Adding 1 hour to Session 6 (→ 3 hours) is better than:
  - Creating technical debt
  - Requiring Blazor to use Tauri invoke temporarily
  - Updating Blazor twice (once for Tauri, once for WebSocket)
- Session 6 becomes complete ADR-0002 compliance checkpoint

### Q5: Implement All OpenCode Protos Now or Incremental? (NEW)
**Q:** Should we implement all 8 OpenCode proto domains in Session 6, or add them incrementally per session?

**A:** ALL IN SESSION 6. Rationale:
- Research is complete (72+ JSON Schemas documented in `docs/proto/`)
- One-time cost: +30 min (45 min vs 15 min piecemeal across 6 sessions)
- Future sessions: Zero proto work (just import what's needed)
- Architecture principle: OpenCode models (`Oc*`) must be 1:1 with server schemas
- Maintenance: 9 modular proto files (not 1 monolithic file)
- Validation: All JSON Schema cross-references verified once
- Clean separation: `Oc*` = server models, `Ipc*` = IPC protocol

---

## 13. Next Steps (Session 7)

After Session 6 completes:

1. **Session handlers:** Implement real handlers for:
   - `IpcListSessionsRequest` → HTTP call to OpenCode server
   - `IpcCreateSessionRequest` → HTTP POST to OpenCode server
   - `IpcDeleteSessionRequest` → HTTP DELETE to OpenCode server

2. **HTTP client:** Create `OpencodeClient` in client-core for OpenCode server communication

3. **SSE event streaming:** Subscribe to `GET {opencode_url}/global/event` and forward to IPC

4. **Request multiplexing:** Handle multiple concurrent requests with `request_id` tracking

---

## Estimated Time Breakdown

| Step | Task | Original | With Server Mgmt | With Full Protos |
|------|------|----------|------------------|------------------|
| 1 | Proto work | 10 min | 15 min | **45 min** |
| 2 | Move code to server.rs | 5 min | 5 min | 5 min |
| 3 | Update error types | 5 min | 5 min | 5 min |
| 4 | Move state to client-core | 0 min | 20 min | 20 min |
| 5 | Create test helpers | 15 min | 15 min | 15 min |
| 6 | Implement auth state machine | 30 min | 30 min | 30 min |
| 7 | Protobuf + server handlers | 20 min | 40 min | 40 min |
| 8 | Update tests (13 total) | 30 min | 40 min | 40 min |
| 9 | Update Tauri main | 0 min | 5 min | 5 min |
| 10 | Cleanup | 10 min | 10 min | 10 min |
| **Total** | | **~2 hours** | **~3 hours** | **~3.5 hours** |

**Buffer:** ~15-30 min for unexpected issues

**Why 3.5 hours?** 

1. **Server management migration** (+1 hour):
   - +State migration from Tauri to client-core  
   - +4 server management handlers
   - +4 integration tests
   - +Tauri main.rs updates

2. **Complete OpenCode proto implementation** (+30 min):
   - Create 9 proto files (was: rename 1 file)
   - Implement ~74 OpenCode messages (was: 32 messages)
   - Verify against JSON Schemas (8 domains)
   - Future-proof: Sessions 7+ have zero proto work

---

## Summary: What Makes Session 6 Different

### Original Scope (2 hours)
- Rename 23 proto messages
- Add auth handshake
- Switch to binary protobuf
- 9 integration tests

### Final Scope (3.5 hours)
- **Create 9 OpenCode proto files** (~74 messages, 1:1 with JSON Schemas)
- **Create 1 IPC proto file** (20 protocol messages)
- Migrate server management from Tauri to client-core (ADR-0002 compliance)
- Add auth handshake + state machine
- Switch to binary protobuf
- 13 integration tests (auth + server management)

### Why the Extra Work is Worth It

**Architectural Integrity:**
- ✅ Two-layer proto: `Oc*` (canonical) + `Ipc*` (protocol)
- ✅ ADR-0002 compliance: Tauri = webview host only
- ✅ ADR-0003 compliance: Blazor → WebSocket → client-core (direct)

**Future-Proofing:**
- ✅ Sessions 7-45: Zero proto work (just import)
- ✅ No breaking changes needed later
- ✅ No "simplified then fixed" technical debt

**Code Quality:**
- ✅ 9 modular proto files (not 1 monolithic)
- ✅ JSON Schema verification (72+ schemas)
- ✅ Production-grade from day 1

### Total Messages Implemented

| Category | Count | Location |
|----------|-------|----------|
| OpenCode Models | ~74 | 9 proto files (`oc_*.proto`) |
| IPC Protocol | 20 | 1 proto file (`ipc.proto`) |
| **Total** | **~94** | **10 proto files** |

---

## Ready to Proceed

This plan is comprehensive and ready for implementation. All architectural decisions documented, patterns established, tests defined.

**Next command:** "Please proceed with Step 1" (or "Please implement all steps")
