# Next Session: Fix Proto & Remove Stale Tauri Commands (8A)

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting.**

## Goals

1. Fix broken C# build (stale proto reference)
2. Remove redundant Tauri invoke code

---

## Problem 1: Broken Build

```
$ dotnet build frontend/desktop/opencode/Opencode.csproj

Could not make proto path relative : error : ../../../proto/server.proto: No such file or directory
Build FAILED.
```

**Fix:** `Opencode.csproj` line 27 - replace `server.proto` with actual proto files from `proto/`

---

## Problem 2: Redundant Tauri Commands

Server lifecycle is now in IPC server (`backend/client-core/src/ipc/server.rs`). Tauri invoke path is stale.

**Remove (Rust):**
- `apps/desktop/opencode/src/commands/server.rs`
- References in `commands/mod.rs` and `main.rs`

**Remove (C#):**
- `frontend/desktop/opencode/Services/ServerService.cs`
- `frontend/desktop/opencode/Services/IServerService.cs`
- `frontend/desktop/opencode/Services/TauriCommands.cs`
- `frontend/desktop/opencode/Services/TauriConstants.cs`
- Registration in `Program.cs`
- Usage in `Home.razor`

---

## Proto Files

Location: `proto/`

| File | Key Types |
|------|-----------|
| `ipc.proto` | `IpcClientMessage`, `IpcServerMessage` |
| `oc_session.proto` | `OcSessionInfo`, `OcSessionList` |
| `oc_model.proto` | `OcModelInfo` |
| `oc_provider.proto` | `OcProviderInfo` |
| `oc_auth.proto` | `OcAuthState` |
| + others | |

---

## Success Criteria

- [ ] `dotnet build` succeeds
- [ ] `dotnet publish -c Release` succeeds
- [ ] `cargo build -p opencode` succeeds
- [ ] `commands/server.rs` removed
- [ ] C# Tauri services removed

---

## Note

`Home.razor` will be broken after this session. Session 8B will fix it with IPC client.

---

## Detailed Plan

See `Session_8A_Plan.md`
