# Tool Execution (`tool.proto`)

**Status:** ✅ Complete (JSON Schema created)  
**Last Updated:** 2026-01-05

---

## Purpose

Track tool execution with comprehensive state (logs, metadata, timing, permissions).

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/toolState.schema.json` - Discriminated union of tool states
- `submodules/opencode/schema/toolStatePending.schema.json` - Pending tool state
- `submodules/opencode/schema/toolStateRunning.schema.json` - Running tool state  
- `submodules/opencode/schema/toolStateCompleted.schema.json` - Completed tool state
- `submodules/opencode/schema/toolStateError.schema.json` - Error tool state
- `submodules/opencode/schema/toolPart.schema.json` - Tool call message part
- `submodules/opencode/schema/permissionRequest.schema.json` - Permission request
- `submodules/opencode/schema/permissionReply.schema.json` - Permission reply (enum)
- `submodules/opencode/schema/permissionToolContext.schema.json` - Tool context for permissions

**Previously derived from (now superseded):**

- `packages/opencode/src/session/message-v2.ts` @ `21dc3c2` lines 214-291 (tool states, tool part)
- `packages/opencode/src/permission/next.ts` @ `21dc3c2` lines 52-74 (permission request/reply)

---

## Generated Validators

The following validators are auto-generated from JSON Schema:

```
submodules/opencode/packages/opencode/generated/validators/
├── toolState.ts
├── toolStatePending.ts
├── toolStateRunning.ts
├── toolStateCompleted.ts
├── toolStateError.ts
├── toolPart.ts
├── permissionRequest.ts
├── permissionReply.ts
└── permissionToolContext.ts
```

Import with `@generated/validators/<name>`.

---

## Messages

### Tool State (Discriminated Union)

```protobuf
syntax = "proto3";
package opencode.tool;

// Tool state discriminated union
// Source: toolState.schema.json
// Status: pending | running | completed | error
oneof tool_state {
  ToolStatePending pending = 1;
  ToolStateRunning running = 2;
  ToolStateCompleted completed = 3;
  ToolStateError error = 4;
}

// Pending tool state
// Source: toolStatePending.schema.json
message ToolStatePending {
  string status = 1;           // const: "pending"
  google.protobuf.Struct input = 2;  // Tool input parameters
  string raw = 3;              // Raw input string
}

// Running tool state
// Source: toolStateRunning.schema.json
message ToolStateRunning {
  string status = 1;           // const: "running"
  google.protobuf.Struct input = 2;  // Tool input parameters
  optional string title = 3;    // Display title
  optional google.protobuf.Struct metadata = 4;  // Additional metadata
  ToolTime time = 5;           // Timing (start only)
}

// Completed tool state
// Source: toolStateCompleted.schema.json
message ToolStateCompleted {
  string status = 1;           // const: "completed"
  google.protobuf.Struct input = 2;  // Tool input parameters
  string output = 3;           // Tool output result
  string title = 4;            // Display title
  google.protobuf.Struct metadata = 5;  // Additional metadata
  ToolTimeWithEnd time = 6;    // Timing (start + end)
  repeated FilePart attachments = 7;  // File attachments
}

// Error tool state
// Source: toolStateError.schema.json
message ToolStateError {
  string status = 1;           // const: "error"
  google.protobuf.Struct input = 2;  // Tool input parameters
  string error = 3;            // Error message
  optional google.protobuf.Struct metadata = 4;  // Additional metadata
  ToolTimeWithEnd time = 5;    // Timing (start + end)
}

// Timing information
message ToolTime {
  double start = 1;            // Start timestamp (ms)
}

message ToolTimeWithEnd {
  double start = 1;            // Start timestamp (ms)
  double end = 2;              // End timestamp (ms)
  optional double compacted = 3;  // Compaction timestamp (ms)
}
```

### Tool Part

```protobuf
// Tool call message part
// Source: toolPart.schema.json
message ToolPart {
  string id = 1;               // Part identifier
  string session_id = 2;       // Session identifier
  string message_id = 3;       // Message identifier
  string type = 4;             // const: "tool"
  string call_id = 5;          // Tool call identifier
  string tool = 6;             // Tool name (bash, read, write, glob, etc.)
  oneof state {                // Current tool execution state
    ToolStatePending pending = 7;
    ToolStateRunning running = 8;
    ToolStateCompleted completed = 9;
    ToolStateError error = 10;
  }
  optional google.protobuf.Struct metadata = 11;  // Additional metadata
}
```

### Permission Request

```protobuf
// Permission request from tool execution
// Source: permissionRequest.schema.json
message PermissionRequest {
  string id = 1;               // Permission ID (prefixed with "per")
  string session_id = 2;       // Session ID (prefixed with "ses")
  string permission = 3;       // Permission type being requested
  repeated string patterns = 4;  // Patterns to match (e.g., file globs)
  google.protobuf.Struct metadata = 5;  // Additional metadata
  repeated string always = 6;  // Patterns to remember if user approves "always"
  optional PermissionToolContext tool = 7;  // Tool context
}

// Tool context for permission request
// Source: permissionToolContext.schema.json
message PermissionToolContext {
  string message_id = 1;       // Message ID containing the tool call
  string call_id = 2;          // Tool call identifier
}
```

### Permission Reply

```protobuf
// User reply to permission request
// Source: permissionReply.schema.json
enum PermissionReply {
  PERMISSION_REPLY_UNSPECIFIED = 0;
  PERMISSION_REPLY_ONCE = 1;    // Approve once
  PERMISSION_REPLY_ALWAYS = 2;  // Approve always (remember patterns)
  PERMISSION_REPLY_REJECT = 3;  // Reject
}
```

---

## Service Definition

```protobuf
service PermissionService {
  rpc RespondToPermission(PermissionResponse) returns (Empty);
}

message PermissionResponse {
  string session_id = 1;
  string permission_id = 2;
  PermissionReply reply = 3;
}

message Empty {}
```

---

## JSON Schema Cross-Reference

### Tool State Fields

| JSON Schema Property | Type | Required | Protobuf Field | Type | Notes |
|---------------------|------|----------|----------------|------|-------|
| `status` | string (const) | ✓ | `string status` | string | Discriminator: "pending", "running", "completed", "error" |
| `input` | object | ✓ | `Struct input` | Struct | additionalProperties: true |
| `raw` | string | ✓ (pending) | `string raw` | string | Only in pending state |
| `output` | string | ✓ (completed) | `string output` | string | Only in completed state |
| `error` | string | ✓ (error) | `string error` | string | Only in error state |
| `title` | string | optional/✓ | `string title` | string | Optional in running, required in completed |
| `metadata` | object | optional | `Struct metadata` | Struct | additionalProperties: true |
| `time` | object | ✓ | `ToolTime/ToolTimeWithEnd` | message | Contains start, end, compacted |
| `attachments` | array | optional | `repeated FilePart` | repeated | Only in completed state |

### Tool Part Fields

| JSON Schema Property | Type | Required | Protobuf Field | Type | Notes |
|---------------------|------|----------|----------------|------|-------|
| `id` | string | ✓ | `string id` | string | |
| `sessionID` | string | ✓ | `string session_id` | string | camelCase → snake_case |
| `messageID` | string | ✓ | `string message_id` | string | camelCase → snake_case |
| `type` | string (const "tool") | ✓ | `string type` | string | Discriminator |
| `callID` | string | ✓ | `string call_id` | string | camelCase → snake_case |
| `tool` | string | ✓ | `string tool` | string | Tool name |
| `state` | ToolState | ✓ | `oneof state` | oneof | Discriminated union |
| `metadata` | object | optional | `Struct metadata` | Struct | |

### Permission Request Fields

| JSON Schema Property | Type | Required | Protobuf Field | Type | Notes |
|---------------------|------|----------|----------------|------|-------|
| `id` | string (pattern: ^per) | ✓ | `string id` | string | Prefixed with "per" |
| `sessionID` | string (pattern: ^ses) | ✓ | `string session_id` | string | Prefixed with "ses" |
| `permission` | string | ✓ | `string permission` | string | |
| `patterns` | string[] | ✓ | `repeated string patterns` | repeated | |
| `metadata` | object | ✓ | `Struct metadata` | Struct | |
| `always` | string[] | ✓ | `repeated string always` | repeated | |
| `tool` | PermissionToolContext | optional | `PermissionToolContext tool` | message | |

### Permission Reply Values

| JSON Schema enum | Protobuf enum |
|-----------------|---------------|
| `"once"` | `PERMISSION_REPLY_ONCE` |
| `"always"` | `PERMISSION_REPLY_ALWAYS` |
| `"reject"` | `PERMISSION_REPLY_REJECT` |

---

## Maps to OpenCode Server

- `PermissionService.RespondToPermission` → `POST /session/{id}/permissions/{id}`

---

## Design Notes

**Tool Call Lifecycle:**

1. `pending` - Tool call created, waiting to execute
2. `running` - Tool is executing, logs streaming
3. `completed` - Tool completed successfully
4. `error` - Tool failed with error

**Permission Flow:**

1. Tool requests permission (e.g., filesystem write)
2. `PermissionRequest` sent to client via SSE event
3. Client displays permission dialog
4. User selects once/always/reject
5. `PermissionResponse` sent back to server
6. Tool continues or aborts based on response

**Discriminated Union Pattern:**

- Tool state uses `status` field as discriminator
- JSON Schema: `oneOf` with `const` status values
- Generated Zod: `z.discriminatedUnion("status", [...])`
- Protobuf: `oneof` with type-specific messages

**Identifier Prefixes:**

- Permission IDs: `per` prefix (e.g., `per01J...`)
- Session IDs: `ses` prefix (e.g., `ses01J...`)
- Validated by JSON Schema `pattern: "^per"` and `pattern: "^ses"`

---

## Refactored TypeScript Source

The following files now import from generated validators:

```typescript
// packages/opencode/src/session/message-v2.ts
import { toolStateSchema, type ToolState } from "@generated/validators/toolState"
import { toolPartSchema, type ToolPart } from "@generated/validators/toolPart"
// ...etc

// packages/opencode/src/permission/next.ts
import { permissionRequestSchema, type PermissionRequest } from "@generated/validators/permissionRequest"
import { permissionReplySchema, type PermissionReply } from "@generated/validators/permissionReply"
// ...etc
```

---

## Verification

- ✅ All 52 JSON Schema files valid (`bun run generate:schemas`)
- ✅ All generated validators match original Zod (field-by-field verified)
- ✅ TypeScript typecheck passes (`bun run typecheck`)
- ✅ All 544 tests pass (`bun test`)
