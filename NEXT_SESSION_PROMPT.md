# Next Session: Implement C# IPC Client (8B)

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting.**

## MANDATORY: Code Reading Rules

**BEFORE suggesting ANY code change:**
1. **Read the actual file first** using the Read tool
2. **Understand existing patterns** (error handling, logging, DI registration)
3. **Suggest ONE minimal change** that preserves existing patterns
4. **Don't brainstorm alternatives** unless explicitly asked

**If you haven't read the file, say "Let me read X first" - don't guess.**

---

## Goals

1. Expose IPC config (port + auth token) to Blazor
2. Create C# WebSocket client that connects to IPC server
3. Replace stubbed `Home.razor` with session list via IPC

---

## Current State (After Session 8A)

### ✅ What Works

- IPC WebSocket server running on port 19876 (`backend/client-core/src/ipc/server.rs`)
- Auth handshake validates token, closes connection if invalid
- Session handlers (list/create/delete) return real data
- IpcConfig stored in Tauri state (`apps/desktop/opencode/src/main.rs` line 68)
- Proto files compile to C# classes

### ❌ What's Missing

- IpcConfig not exposed to Blazor (needs Tauri command)
- No C# WebSocket client
- `Home.razor` is stubbed (Session 8A removed old Tauri invoke code)

---

## Architecture (ADR-0003)

```
Blazor Component (Home.razor)
    ↓
IpcClientService (C#)
    ↓
System.Net.WebSockets.ClientWebSocket (native C#, no JS)
    ↓
Binary protobuf over WebSocket
    ↓
IPC Server (backend/client-core/src/ipc/server.rs)
    ↓
OpencodeClient → OpenCode Server
```

---

## IPC Protocol

**Connection:**
- URL: `ws://127.0.0.1:19876`
- Binary WebSocket (not text)

**Auth (first message):**
```
Client → IpcClientMessage { request_id: 0, payload: IpcAuthHandshake { token } }
Server → IpcServerMessage { request_id: 0, payload: IpcAuthHandshakeResponse { success } }
```
If `success == false`, server closes connection.

**Request/Response:**
- Client sets incrementing `request_id`
- Server echoes `request_id` in response
- Use request_id to correlate responses (async)

**Serialization:**
- C#: `message.ToByteArray()` / `Parser.ParseFrom(bytes)`
- WebSocket message type: Binary

---

## Proto Types (Available in C#)

After Session 8A, these compile from `proto/*.proto`:

| Proto File | Key C# Types |
|------------|--------------|
| `ipc.proto` | `IpcClientMessage`, `IpcServerMessage`, `IpcAuthHandshake`, `IpcListSessionsRequest`, etc. |
| `oc_session.proto` | `OcSessionInfo`, `OcSessionList` |

Proto package `opencode` → C# namespace `Opencode`

---

## Deliverables

| # | Task | Success Criteria |
|---|------|------------------|
| 1 | Expose IpcConfig via Tauri command | Blazor can get port + token |
| 2 | C# WebSocket client service | Connects, authenticates, sends/receives protobuf |
| 3 | Update Home.razor | Displays session list from IPC |

---

## Files to Modify/Create

### Rust Side

| File | Action |
|------|--------|
| `apps/desktop/opencode/src/commands/ipc.rs` | **Create** - Add `get_ipc_config` command |
| `apps/desktop/opencode/src/commands/mod.rs` | **Create if missing** or modify - Add `pub mod ipc;` |
| `apps/desktop/opencode/src/main.rs` | **Modify** - Add command to `invoke_handler` |

### C# Side

| File | Action |
|------|--------|
| `frontend/desktop/opencode/Services/IpcClientService.cs` | **Create** - WebSocket client, auth, request/response |
| `frontend/desktop/opencode/Services/IIpcClientService.cs` | **Create** - Interface |
| `frontend/desktop/opencode/Program.cs` | **Modify** - Register `IIpcClientService` |
| `frontend/desktop/opencode/Pages/Home.razor` | **Modify** - Replace stub with session list |

---

## Success Criteria

Run `cargo tauri dev` and verify:

- [ ] App starts without errors
- [ ] IPC connection establishes (check logs)
- [ ] Auth succeeds
- [ ] Session list displays in Home.razor
- [ ] No custom JavaScript files (per NO_CUSTOM_JAVASCRIPT_POLICY)

---

## Detailed Plan

See `Session_8B_Plan.md`
