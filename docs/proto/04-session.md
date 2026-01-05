# Session Management (`session.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

Track conversation sessions/tabs with model selection and working directory.

---

## Source of Truth

**Note:** No dedicated JSON Schema yet for session types.

**Future work:** Create JSON Schema for session types to align with model/provider pattern.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.session;

import "model.proto";
import "provider.proto";

// Core session identity and metadata (server-managed)
message SessionInfo {
  string id = 1;
  string title = 2;
  string directory = 3;           // Working directory
  optional string version = 4;    // Session version for optimistic updates
  optional SessionTime time = 5;
}

message SessionTime {
  int64 created = 1;   // Unix timestamp (ms)
  int64 updated = 2;
}

// Tab state (client-side UI state + server session)
message TabInfo {
  SessionInfo session = 1;                    // Server-managed session data
  optional ModelSelection selected_model = 2; // Client UI state (from model.proto)
  optional string selected_agent = 3;         // Client UI state
}

message TabList {
  repeated TabInfo tabs = 1;
}

// Request/response messages
message SessionList {
  repeated SessionInfo sessions = 1;
}

message CreateSessionRequest {
  optional string title = 1;
}

message DeleteSessionRequest {
  string session_id = 1;
}

message UpdateDirectoryRequest {
  string session_id = 1;
  string directory = 2;
}

message Empty {}
```

---

## Service Definition

```protobuf
service SessionService {
  rpc ListSessions(Empty) returns (SessionList);
  rpc CreateSession(CreateSessionRequest) returns (SessionInfo);
  rpc DeleteSession(DeleteSessionRequest) returns (Empty);
  rpc UpdateSessionDirectory(UpdateDirectoryRequest) returns (Empty);
}
```

---

## Maps to OpenCode Server

- `SessionService.ListSessions` → `GET /session`
- `SessionService.CreateSession` → `POST /session`
- `SessionService.DeleteSession` → `DELETE /session/{id}`
- `SessionService.UpdateSessionDirectory` → Updates internal state (sent as `x-opencode-directory` header)

---

## Design Notes

**Session vs Tab:**

- `SessionInfo` is server-managed (persisted, has ID)
- `TabInfo` is client UI state (selected model, agent) + server session
- Multiple tabs can reference the same session (future multi-view support)

**Working Directory:**

- Each session has a working directory context
- Sent to server via `x-opencode-directory` header
- Affects file operations, tool permissions, etc.

---

## TODO

- [ ] Create `sessionInfo.schema.json` for session metadata
- [ ] Validate against actual session API responses
- [ ] Document session lifecycle (create, update, delete)
