# Session 6 Continuation: IPC Tests & Cleanup

## Session Context

**Previous session completed:** Steps 1-7 of Session 6 (Proto foundation + IPC server implementation)

**This session:** Complete Steps 8-10 (Tests + Tauri integration + Cleanup)

---

## 📋 Read These Files First

1. `/Users/tony/git/opencode-tauri/CRITICAL_OPERATING_CONSTRAINTS.md` - Operating mode and quality standards
2. `/Users/tony/git/opencode-tauri/Session_6_Plan.md` - Full session plan with progress
3. `/Users/tony/git/opencode-tauri/docs/adr/0002-thin-tauri-layer-principle.md` - Architecture principle
4. `/Users/tony/git/opencode-tauri/docs/adr/0003-websocket-protobuf-ipc.md` - IPC protocol design

---

## ✅ What's Already Done (Steps 1-7)

### Core Infrastructure
- ✅ All 11 proto files created and compiling
- ✅ `IpcErrorCode` enum added (no magic strings)
- ✅ Error types expanded (`Auth`, `ProtobufDecode`, `ProtobufEncode`)
- ✅ File structure cleaned (`handle.rs` for struct, `server.rs` for implementation)
- ✅ Comprehensive documentation added to all modules and functions

### Auth & Security
- ✅ Auth state machine implemented (`ConnectionState`)
- ✅ Token generation (UUID) on server start
- ✅ First-message validation (must be `IpcAuthHandshake`)
- ✅ Non-localhost rejection
- ✅ Fail-closed security model

### Server Management
- ✅ State management migrated to client-core (`IpcState` actor pattern)
- ✅ Binary protobuf message handling
- ✅ 4 server handlers implemented and wired up:
  - `handle_discover_server()` - Discovery + state update
  - `handle_spawn_server()` - Spawn + state update
  - `handle_check_health()` - Health check from state
  - `handle_stop_server()` - Stop + state clear
- ✅ Test helpers created (`helpers.rs`)

### Current Status
- ✅ **Build passes:** `cargo build -p client-core` succeeds
- ⚠️ **3 tests failing:** Echo tests don't authenticate (expected - will fix in Step 8)
- ✅ **Code quality:** Production-grade, no TODOs, comprehensive error handling

---

## 🎯 What Needs to Be Done (Steps 8-10)

### Step 8: Update Tests (~40 min)

**Current test state:**
- 3 IPC echo tests **FAILING** (don't send auth handshake)
- Test helpers exist but unused

**What needs to happen:**

1. **Update 3 existing echo tests** (`integration_tests/ipc/ipc.rs`):
   - Add auth handshake before sending messages
   - Use helpers from `helpers.rs`
   - Tests should authenticate, then echo messages

2. **Add 6 new auth tests**:
   - Valid token → auth succeeds
   - Invalid token → auth fails, connection closes
   - Wrong first message (not auth) → connection closes
   - Missing auth token → connection closes
   - Non-localhost connection → rejected silently
   - Auth handshake after first message → error response

3. **Add 4 server management tests**:
   - Discover server (no server running → `None`)
   - Spawn server → receives `IpcServerInfo`
   - Check health (no server → error)
   - Stop server (no server → error)

**Test patterns:**
```rust
// Auth then send messages
let mut ws = connect_to_server(port).await;
authenticate(&mut ws, "test-token").await;
// Now send actual messages
```

### Step 9: Update Tauri main.rs (~5 min)

**File:** `apps/desktop/opencode/src/main.rs`

**Changes needed:**
1. Start IPC server in `.setup()` handler
2. Pass auth token (generate or read from config)
3. Log the port and token for Blazor to connect

**Example:**
```rust
tauri::Builder::default()
    .setup(|app| {
        let auth_token = uuid::Uuid::new_v4().to_string();
        tokio::spawn(async move {
            client_core::ipc::start_ipc_server(19876, Some(auth_token))
                .await
                .expect("Failed to start IPC server");
        });
        Ok(())
    })
    .run(...)
```

### Step 10: Cleanup (~10 min)

1. **Format code:**
   ```bash
   cargo fmt --all
   ```

2. **Fix clippy warnings:**
   ```bash
   cargo clippy --all-targets --all-features
   ```

3. **Final test run:**
   ```bash
   cargo test -p client-core
   ```

4. **Verify logging:**
   - Check that `info!`, `warn!`, `error!` are used appropriately
   - No `println!` or `dbg!` macros

---

## 🚨 Critical Constraints (Reminder)

### Teaching Mode
- **Default:** Teach first, explain approach, THEN implement when asked
- **Never:** Write code without explaining the plan first
- **Always:** Ask "Should I explain this step, or implement it?"

### Code Quality
- **No magic strings:** Use enums/constants (already done for error codes)
- **No TODOs:** If something can't be done, document why and what's needed
- **Comprehensive errors:** Every error has context and location
- **Production-grade:** This is shipping code, not a prototype

### File Organization
- **Struct in matching file:** `IpcServerHandle` in `handle.rs` (already done)
- **No "dump everything in one file":** Keep modules focused and clear
- **Documentation required:** Module docs (`//!`) and function docs (`///`)

### Architecture
- **ADR-0002:** All business logic in `client-core`, NOT in Tauri
- **ADR-0003:** WebSocket + binary protobuf for IPC
- **Security first:** Localhost-only, auth required, fail-closed

---

## 🔧 Verification Before Starting

Run these commands to verify current state:

```bash
cd /Users/tony/git/opencode-tauri

# Proto files exist
ls proto/oc_*.proto proto/ipc.proto

# Build succeeds
cargo build -p client-core

# Check test status (3 should fail)
cargo test -p client-core --test integration_tests

# Verify handlers exist
rg "async fn handle_discover_server" backend/client-core/src/ipc/server.rs
rg "async fn handle_spawn_server" backend/client-core/src/ipc/server.rs
rg "async fn handle_check_health" backend/client-core/src/ipc/server.rs
rg "async fn handle_stop_server" backend/client-core/src/ipc/server.rs
```

**Expected results:**
- ✅ Proto files present (11 files)
- ✅ Build succeeds
- ⚠️ 3 tests failing (echo tests)
- ✅ All 4 handlers found

---

## 🎯 Session Goal

By end of this session:
1. ✅ All tests passing (13 IPC tests total: 3 echo + 6 auth + 4 server mgmt)
2. ✅ Tauri main.rs starts IPC server
3. ✅ Code formatted and clippy-clean
4. ✅ Production-ready IPC server with auth, protobuf, and server management

**Estimated time:** ~1 hour

---

## 💬 How to Start

When you begin, say:

> "I've read the context files. Current state verified: Steps 1-7 complete, 3 tests failing as expected. Ready to start Step 8 (Update Tests). Should I explain the test strategy first, or start implementing?"

Then wait for my decision on teaching mode vs. direct implementation.
