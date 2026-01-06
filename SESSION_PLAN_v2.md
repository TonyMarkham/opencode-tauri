# Session Plan v2: OpenCode Tauri-Blazor Client

**Budget:** 100K tokens per session (hard limit)  
**Philosophy:** Every session ends with something you can demo  
**Last Updated:** 2026-01-05

---

## What's Built (Sessions 1-4)

| Component | Status | Location |
|-----------|--------|----------|
| Rust client-core (discovery, spawn, health) | DONE | `backend/client-core/` |
| Tauri app shell + commands | DONE | `apps/desktop/opencode/` |
| Blazor scaffold + server UI | DONE | `frontend/desktop/opencode/` |
| Proto schemas (ipc.proto, server.proto) | DONE | `proto/` |
| Proto documentation (72+ JSON schemas) | DONE | `docs/proto/` |
| Protobuf codegen (Rust side) | DONE | `client-core/build.rs` |

**Current State:** App launches, discovers/spawns OpenCode server, shows status in UI.

---

## What's Next

### Session 5: "Send Message, See Response" (~100K)

**Demo at end:** Type message, click send, see Claude's response appear.

**Build:**
1. WebSocket server in client-core (Rust)
2. WebSocket client service (C# - no JS)
3. Connect on app startup
4. Send message → receive response (non-streaming first)
5. Basic chat UI (input box + message list)

**Don't build:**
- Token streaming (Session 6)
- Tool calls (Session 7)
- Multi-tab (Session 8)

**Success:** Send "Hello" → See "Hello! How can I help you today?" (or similar)

---

### Session 6: "Streaming Tokens" (~100K)

**Demo at end:** See response appear word-by-word, not all at once.

**Build:**
1. SSE subscription to OpenCode server (Rust)
2. SSE → WebSocket bridge (push tokens to Blazor)
3. Streaming proto messages (`ChatToken`, `ChatCompleted`)
4. UI updates as tokens arrive
5. Cancel button (abort mid-stream)

**Success:** Response types out visibly, can cancel mid-response.

---

### Session 7: "Tool Calls + Permissions" (~100K)

**Demo at end:** Ask "list files in current directory", see tool execute, approve permission.

**Build:**
1. Tool call proto messages
2. Tool call UI component (collapsible block)
3. Permission dialog (inline)
4. Permission response to server
5. Tool output display

**Success:** Tool executes, permission dialog appears, can approve/reject.

---

### Session 8: "Multi-Tab + Agent Selection" (~100K)

**Demo at end:** Open multiple chat tabs, select different agents per tab.

**Build:**
1. Tab bar component
2. Tab state management (multiple sessions)
3. Agent picker (sidebar or dropdown)
4. Per-tab session ID tracking

**Success:** Can have 3 tabs open with different conversations.

---

### Session 9: "Model Selection + Polish" (~100K)

**Demo at end:** Switch models mid-conversation, see provider status.

**Build:**
1. Model picker dropdown
2. Provider status display
3. Auth mode toggle (OAuth vs API key)
4. Settings persistence
5. Markdown rendering (Markdig)

**Success:** Full chat experience with model switching.

---

### Session 10: "Production Ready" (~100K)

**Demo at end:** Ship it.

**Build:**
1. Error handling polish
2. Loading states everywhere
3. Cross-platform testing
4. Build system (justfile)
5. Documentation

**Success:** Someone else can clone, build, and use it.

---

## Architecture Reminders

**IPC:** WebSocket + Protobuf (ADR-0003)
```
Blazor (C#) → ClientWebSocket → ws://127.0.0.1:PORT → tokio-tungstenite (Rust) → HTTP → OpenCode Server
```

**Key Principles:**
- Blazor is dumb glass (renders, never decides)
- All logic in client-core (not Tauri)
- Zero custom JavaScript
- Binary protocol only (protobuf)

**Existing Proto Messages** (`proto/ipc.proto`):
- `ClientMessage` / `ServerMessage` envelopes
- Sessions: `ListSessions`, `CreateSession`, `DeleteSession`
- Agents: `ListAgents`
- Providers: `GetProviderStatus`
- Auth: `SetAuth`, `GetAuth`

---

## Files Reference

**Rust:**
- `backend/client-core/src/lib.rs` - Public API
- `backend/client-core/src/discovery/` - Server discovery
- `backend/client-core/src/ws.rs` - WebSocket (TO BUILD)
- `backend/client-core/build.rs` - Protobuf codegen

**C#:**
- `frontend/desktop/opencode/Services/` - Service layer
- `frontend/desktop/opencode/Pages/Home.razor` - Server status UI

**Proto:**
- `proto/ipc.proto` - IPC messages
- `proto/server.proto` - ServerInfo

---

## Token Budget History

| Session | Estimate | Actual | Notes |
|---------|----------|--------|-------|
| 1 | 60K | 60K | Rust core |
| 2 | 80K | 120K | Tauri backend (state refactor) |
| 3 | 60K | 90K | Blazor scaffold |
| 4 | 140K | 60K | Became docs only |
| 5 | 100K | ? | NEXT |
