# Session 8B: C# IPC Client

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting this session.**

**Status:** Ready to Start  
**Prerequisite:** Session 8A complete (Proto compilation fixed)  
**Estimated Tokens:** ~100K

---

## Goal

Blazor UI displays sessions fetched via WebSocket IPC (not Tauri invoke).

---

## Current State

### What Works (After Session 7)

| Component | Location | Status |
|-----------|----------|--------|
| IPC WebSocket server | `backend/client-core/src/ipc/server.rs` | Running on port 19876 |
| Auth handshake | `server.rs` | Token validated, connection closed if invalid |
| Session handlers | `server.rs` | list/create/delete return real data |
| OpencodeClient | `backend/client-core/src/opencode_client/mod.rs` | HTTP client for OpenCode server |
| Field normalizer | `backend/client-core/src/field_normalizer.rs` | JSON ↔ proto field names |
| IpcConfig | `apps/desktop/opencode/src/ipc_config.rs` | Stores port + auth_token |

### What Doesn't Work

| Gap | Details |
|-----|---------|
| IPC config not exposed | `IpcConfig` is in Tauri state but no command exposes it to Blazor |
| No C# WebSocket client | Blazor has no code to connect to IPC server |
| Blazor uses Tauri invoke | `ServerService.cs` calls Tauri commands via JSInterop |

### Fixed in Session 8A

| Fixed | Details |
|-------|---------|
| Proto compilation | Proto files now compile to C# classes |

---

## Architecture Constraints

| ADR | Constraint |
|-----|------------|
| ADR-0002 | Tauri is thin - only hosts webview |
| ADR-0003 | Blazor ↔ client-core via WebSocket + binary protobuf |
| NO_CUSTOM_JAVASCRIPT_POLICY | No custom JS files - `ClientWebSocket` is native C# |

### Data Flow (Target State)

```
Blazor Component
    ↓ IpcClientService
    ↓ System.Net.WebSockets.ClientWebSocket (native C#)
    ↓ Binary protobuf
WebSocket ws://127.0.0.1:19876
    ↓
client-core IPC Server
    ↓ OpencodeClient
OpenCode Server
```

---

## Existing Code Reference

### Rust Files (Read-Only Context)

| File | Contents |
|------|----------|
| `backend/client-core/src/ipc/server.rs` | WebSocket server, auth validation, message handlers |
| `backend/client-core/src/ipc/state.rs` | `IpcState` with server info and `OpencodeClient` |
| `proto/ipc.proto` | `IpcClientMessage`, `IpcServerMessage`, all payloads |
| `proto/oc_session.proto` | `OcSessionInfo`, `OcSessionList`, etc. |
| `apps/desktop/opencode/src/ipc_config.rs` | `IpcConfig { port: u16, auth_token: String }` |
| `apps/desktop/opencode/src/main.rs` | Lines 54-75: starts IPC server, stores config in Tauri state |
| `apps/desktop/opencode/src/commands/server.rs` | Existing Tauri commands pattern |
| `apps/desktop/opencode/src/commands/mod.rs` | Module declarations |

### C# Files (Modify/Create)

| File | Current State (After 8A) |
|------|--------------------------|
| `frontend/desktop/opencode/Opencode.csproj` | Line 11: `<PublishDir>`. Proto references fixed. |
| `frontend/desktop/opencode/Services/` | Empty (Tauri services removed in 8A) |
| `frontend/desktop/opencode/Pages/Home.razor` | Broken (IServerService removed in 8A) |
| `frontend/desktop/opencode/Program.cs` | DI registration (IServerService removed in 8A) |

### Proto Files

Location: `proto/` directory

| File | Key Types |
|------|-----------|
| `ipc.proto` | `IpcClientMessage`, `IpcServerMessage`, `IpcAuthHandshake`, `IpcAuthHandshakeResponse`, `IpcListSessionsRequest`, `IpcCreateSessionRequest`, `IpcDeleteSessionRequest`, `IpcErrorResponse` |
| `oc_session.proto` | `OcSessionInfo`, `OcSessionList`, `OcSessionTime`, `OcSessionSummary` |
| `oc_model.proto` | `OcModelInfo`, etc. |
| `oc_provider.proto` | `OcProviderInfo`, etc. |
| `oc_auth.proto` | Auth-related types |

Proto package: `opencode` → C# namespace: `Opencode`

### IPC Protocol Details

**Connection:**
- URL: `ws://127.0.0.1:19876`
- Protocol: Binary WebSocket (not text)

**Auth handshake (must be first message):**
- Client sends: `IpcClientMessage { request_id: 0, payload: IpcAuthHandshake { token } }`
- Server responds: `IpcServerMessage { request_id: 0, payload: IpcAuthHandshakeResponse { success, error? } }`
- If `success == false`, server closes connection

**Request/response pattern:**
- Client sets `request_id` (incrementing counter)
- Server echoes `request_id` in response
- Correlation needed for concurrent requests

**Message format:**
- Serialization: `message.ToByteArray()` / `Parser.ParseFrom(bytes)`
- WebSocket message type: Binary

### Blazor WebSocket API

`System.Net.WebSockets.ClientWebSocket` is a native C# class:
- Works in Blazor WASM
- No JavaScript required
- Methods: `ConnectAsync`, `SendAsync`, `ReceiveAsync`, `CloseAsync`

### Opencode.csproj Config (After Session 8A)

```xml
<!-- Publish output goes to Tauri frontend directory -->
<PublishDir>../../../apps/desktop/opencode/frontend/</PublishDir>

<!-- Protobuf packages -->
<PackageReference Include="Google.Protobuf" Version="3.33.2" />
<PackageReference Include="Grpc.Tools" Version="2.76.0">
  <PrivateAssets>all</PrivateAssets>
  <IncludeAssets>runtime; build; native; contentfiles; analyzers; buildtransitive</IncludeAssets>
</PackageReference>

<!-- Proto references (fixed in Session 8A) -->
<ItemGroup>
  <Protobuf Include="..\..\..\proto\*.proto" GrpcServices="None" />
</ItemGroup>
```

Notes:
- `dotnet publish` outputs to `apps/desktop/opencode/frontend/`
- Tauri serves from `./frontend/wwwroot` (see `tauri.conf.json`)
- Proto types available: `IpcClientMessage`, `OcSessionInfo`, etc.

---

## Deliverables

| # | Deliverable | Success Criteria |
|---|-------------|------------------|
| 1 | IPC config accessible from Blazor | Blazor can obtain port and auth_token |
| 2 | WebSocket client service | Connects, authenticates, sends/receives protobuf |
| 3 | Session operations in UI | List, create, delete sessions work |

---

## Session Boundaries

### In Scope
- IPC connection from Blazor to client-core
- Auth handshake
- Session list/create/delete via IPC
- Basic UI to display sessions

### Out of Scope (Future Sessions)
- Streaming (SSE events, LLM tokens)
- Chat messages
- Agent operations
- Tool calls
- Migrating server discovery/spawn to IPC

### Removed in Session 8A

| Component | Status |
|-----------|--------|
| `ServerService.cs` | Removed |
| `IServerService.cs` | Removed |
| `commands/server.rs` | Removed |
| `TauriCommands.cs`, `TauriConstants.cs` | Removed |

Session 8A cleaned up the stale Tauri invoke path. Session 8B builds the IPC client to replace it.

---

## Testing

### Manual Verification
1. App starts → IPC connection established
2. Auth succeeds → no errors
3. Sessions display → list from OpenCode server
4. Create session → appears in list
5. Delete session → removed from list

### Verification Commands
```bash
# C# publish (outputs to apps/desktop/opencode/frontend/)
dotnet publish frontend/desktop/opencode/Opencode.csproj -c Release

# Tauri dev (builds Rust + serves Blazor)
cargo tauri dev

# No custom JavaScript
find frontend -name "*.js" | grep -v _framework | grep -v node_modules
# Should return nothing
```

---

## Files Summary

### Rust Side

| File | Notes |
|------|-------|
| `apps/desktop/opencode/src/ipc_config.rs` | Already has `port()` and `auth_token()` getters |
| `apps/desktop/opencode/src/commands/mod.rs` | Add new module if creating new command file |
| `apps/desktop/opencode/src/main.rs` | `IpcConfig` stored at line 75: `app.manage(IpcConfig::new(...))` |

### C# Side

| File | Notes |
|------|-------|
| `frontend/desktop/opencode/Opencode.csproj` | Proto compilation fixed in Session 8A |
| `frontend/desktop/opencode/Services/` | New service files go here |
| `frontend/desktop/opencode/Pages/Home.razor` | Currently uses `IServerService` |
| `frontend/desktop/opencode/Program.cs` | DI registration |

---

## Reference: Existing Patterns

### Tauri Command Pattern (from `commands/server.rs`)

```rust
#[TauriCommand]
pub async fn discover_server(
    state: State<'_, AppState>,
) -> Result<Option<IpcServerInfo>, OpencodeError> {
    // implementation
}
```

### Blazor Service Pattern (from `ServerService.cs`)

```csharp
public class ServerService : IServerService
{
    private readonly IJSRuntime _jsRuntime;
    
    public async Task<ServerInfo?> DiscoverServerAsync()
    {
        var result = await _jsRuntime.InvokeAsync<JsonElement>(
            TauriConstants.TauriInvoke,
            TauriCommands.DiscoverServer);
        // ...
    }
}
```

### DI Registration Pattern (from `Program.cs`)

```csharp
builder.Services.AddScoped<IServerService, ServerService>();
```

---

## Important Context

1. **One Tauri invoke is acceptable** - ADR-0003 allows getting IPC config via Tauri invoke at startup
2. **Auth is required** - Server closes connection if first message is not valid auth
3. **Binary protocol** - All messages are binary protobuf, not JSON or text
4. **request_id correlation** - Server echoes request_id for matching responses
5. **ClientWebSocket is native C#** - No JavaScript needed per ADR-0003
