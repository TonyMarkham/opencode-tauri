# Session 8A: Fix Proto Compilation & Remove Stale Tauri Commands

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting this session.**

**Status:** Ready to Start  
**Prerequisite:** Session 7 complete  
**Estimated Tokens:** ~40K

---

## Goal

1. Fix the broken C# build (stale proto reference)
2. Remove redundant Tauri invoke code (server lifecycle now handled by IPC)

---

## Problem 1: Broken C# Build

```
$ dotnet build frontend/desktop/opencode/Opencode.csproj

Could not make proto path relative : error : ../../../proto/server.proto: No such file or directory
Build FAILED.
```

**Cause:** `Opencode.csproj` line 27 references `server.proto` which does not exist.

---

## Problem 2: Redundant Tauri Invoke Code

Server lifecycle operations (discover, spawn, health, stop) are now implemented in the IPC server. The Tauri invoke path is stale/redundant.

**IPC server already has:**
- `handle_discover_server` 
- `handle_spawn_server`
- `handle_check_health`
- `handle_stop_server`

(See `backend/client-core/src/ipc/server.rs`, `proto/ipc.proto` lines 31-35)

---

## Files to Modify

### Fix Proto Reference

| File | Line | Change |
|------|------|--------|
| `frontend/desktop/opencode/Opencode.csproj` | 27 | Replace `server.proto` with actual proto files |

### Remove Redundant Tauri Code

**Rust (Tauri app):**

| File | Action |
|------|--------|
| `apps/desktop/opencode/src/commands/server.rs` | Remove entirely |
| `apps/desktop/opencode/src/commands/mod.rs` | Remove `pub mod server;` |
| `apps/desktop/opencode/src/main.rs` | Remove server commands from `invoke_handler` (lines 23-27) |

**C# (Blazor):**

| File | Action |
|------|--------|
| `frontend/desktop/opencode/Services/ServerService.cs` | Remove entirely |
| `frontend/desktop/opencode/Services/IServerService.cs` | Remove entirely |
| `frontend/desktop/opencode/Services/TauriCommands.cs` | Remove entirely |
| `frontend/desktop/opencode/Services/TauriConstants.cs` | Remove entirely |
| `frontend/desktop/opencode/Program.cs` | Remove `IServerService` registration |
| `frontend/desktop/opencode/Pages/Home.razor` | Remove `IServerService` usage (temporary - will be replaced in 8B) |

---

## Current State Reference

### Opencode.csproj (lines 26-28)

```xml
<ItemGroup>
  <Protobuf Include="..\..\..\proto\server.proto" GrpcServices="None" />
</ItemGroup>
```

### Actual Proto Files in `proto/`

| File | Key Types |
|------|-----------|
| `ipc.proto` | `IpcClientMessage`, `IpcServerMessage`, `IpcAuthHandshake`, `IpcErrorResponse` |
| `oc_session.proto` | `OcSessionInfo`, `OcSessionList`, `OcSessionTime` |
| `oc_model.proto` | `OcModelInfo`, `OcModelList` |
| `oc_provider.proto` | `OcProviderInfo`, `OcProviderList` |
| `oc_auth.proto` | `OcAuthState`, `OcAuthInfo` |
| `oc_message.proto` | `OcMessage`, `OcMessageList` |
| `oc_message_part.proto` | `OcMessagePart` |
| `oc_message_error.proto` | `OcMessageError` |
| `oc_tool.proto` | `OcToolCall`, `OcToolResult` |
| `oc_agent.proto` | `OcAgentInfo` |
| `oc_event.proto` | `OcEvent` |

### main.rs invoke_handler (lines 22-28)

```rust
.invoke_handler(tauri::generate_handler![
    commands::server::discover_server,
    commands::server::spawn_server,
    commands::server::check_health,
    commands::server::stop_server,
])
```

### Program.cs service registration

```csharp
builder.Services.AddScoped<IServerService, ServerService>();
```

---

## Deliverables

| # | Deliverable | Success Criteria |
|---|-------------|------------------|
| 1 | Proto compilation fixed | `dotnet build` succeeds |
| 2 | Tauri commands removed | `commands/server.rs` gone, `main.rs` has empty or minimal `invoke_handler` |
| 3 | C# Tauri services removed | `ServerService.cs`, `IServerService.cs`, `TauriCommands.cs`, `TauriConstants.cs` gone |
| 4 | Build succeeds | `cargo build -p opencode` and `dotnet publish -c Release` both pass |

---

## Verification

```bash
# C# build
dotnet build frontend/desktop/opencode/Opencode.csproj

# C# publish
dotnet publish frontend/desktop/opencode/Opencode.csproj -c Release

# Rust build
cargo build -p opencode

# Verify server.rs removed
ls apps/desktop/opencode/src/commands/
# Should NOT contain server.rs

# Verify C# services removed
ls frontend/desktop/opencode/Services/
# Should NOT contain ServerService.cs, IServerService.cs, TauriCommands.cs, TauriConstants.cs
```

---

## Notes

- `Home.razor` will be broken after removing `IServerService` - this is expected. Session 8B will replace it with IPC client.
- `GrpcServices="None"` is correct for proto compilation - only need message types
- Proto package is `opencode` → C# namespace `Opencode`
- Keep `IpcConfig` and the `get_ipc_config` command (needed for 8B) - actually this doesn't exist yet, 8B will create it
