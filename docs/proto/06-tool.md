# Tool Execution (`tool.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

Track tool execution with comprehensive state (logs, metadata, timing, permissions).

---

## Source of Truth

**Note:** No dedicated JSON Schema yet for tool types.

**Future work:** Create JSON Schema for tool types to align with model/provider pattern.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.tool;

// Tool execution state (comprehensive from opencode-egui audit in submodules/)
message ToolCallState {
  string id = 1;                    // Tool instance ID
  string name = 2;                  // Tool name (bash, read, etc.)
  string status = 3;                // pending, running, success, error, cancelled
  optional string call_id = 4;      // Call ID (for permissions + cancellation)
  string input_json = 5;            // JSON string (tool input)
  optional string output = 6;       // Tool output
  optional string error = 7;        // Error message
  repeated string logs = 8;         // Streaming logs from tool
  string metadata_json = 9;         // JSON string (arbitrary metadata)
  optional int64 started_at = 10;   // Unix timestamp (ms)
  optional int64 finished_at = 11;
}

// Permission request (tool needs user approval)
message PermissionRequest {
  string id = 1;
  string type = 2;                  // filesystem, network, etc.
  repeated string pattern = 3;      // Glob patterns
  string session_id = 4;
  string message_id = 5;
  optional string call_id = 6;
  string title = 7;
  string metadata_json = 8;         // JSON string
  int64 created_at = 9;
}

message PermissionResponse {
  string session_id = 1;
  string permission_id = 2;
  string response = 3;              // "approve" | "reject"
}

message Empty {}
```

---

## Service Definition

```protobuf
service PermissionService {
  rpc RespondToPermission(PermissionResponse) returns (Empty);
}
```

---

## Maps to OpenCode Server

- `PermissionService.RespondToPermission` → `POST /session/{id}/permissions/{id}`

---

## Design Notes

**Tool Call Lifecycle:**

1. `pending` - Tool call created, waiting to execute
2. `running` - Tool is executing, logs streaming
3. `success` - Tool completed successfully
4. `error` - Tool failed with error
5. `cancelled` - Tool was aborted by user

**Permission Flow:**

1. Tool requests permission (e.g., filesystem write)
2. `PermissionRequest` sent to client via SSE event
3. Client displays permission dialog
4. User approves/rejects
5. `PermissionResponse` sent back to server
6. Tool continues or aborts based on response

**JSON Fields:**

- `input_json` and `metadata_json` are JSON strings (not structured)
- Allows flexible tool-specific data without schema changes
- Client parses JSON as needed for display

---

## TODO

- [ ] Create `toolCallState.schema.json` for tool state
- [ ] Create `permissionRequest.schema.json` for permissions
- [ ] Document tool types (bash, read, write, glob, etc.)
- [ ] Validate against actual tool execution events
