# Session Management (`session.proto`)

**Status:** ✅ Complete  
**Last Updated:** 2026-01-05

---

## Purpose

Track conversation sessions/tabs with model selection and working directory.

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/sessionInfo.schema.json` - Core session metadata
- `submodules/opencode/schema/sessionTime.schema.json` - Session timestamps
- `submodules/opencode/schema/sessionSummary.schema.json` - Session summary statistics
- `submodules/opencode/schema/sessionShare.schema.json` - Share information
- `submodules/opencode/schema/sessionRevert.schema.json` - Revert state
- `submodules/opencode/schema/sessionList.schema.json` - API response type
- `submodules/opencode/schema/fileDiff.schema.json` - File diff metadata
- `submodules/opencode/schema/permissionRule.schema.json` - Permission rule
- `submodules/opencode/schema/permissionRuleset.schema.json` - Permission ruleset
- `submodules/opencode/schema/permissionAction.schema.json` - Permission action enum

**Previously derived from (now superseded):**

- `packages/opencode/src/session/index.ts` @ `351ddee` lines 39-79 (Session.Info)
- `packages/opencode/src/snapshot/index.ts` @ `0e08655` lines 147-157 (FileDiff)
- `packages/opencode/src/permission/next.ts` @ `47c670a` lines 15-33 (Permission types)

---

## Messages

```protobuf
syntax = "proto3";
package opencode.session;

import "model.proto";
import "provider.proto";

// Core session identity and metadata (server-managed)
// Source: submodules/opencode/schema/sessionInfo.schema.json (canonical)
message SessionInfo {
  string id = 1;
  string project_id = 2;
  string directory = 3;
  optional string parent_id = 4;
  optional SessionSummary summary = 5;
  optional SessionShare share = 6;
  string title = 7;
  string version = 8;
  SessionTime time = 9;
  optional PermissionRuleset permission = 10;
  optional SessionRevert revert = 11;
}

// Session timestamp metadata
// Source: submodules/opencode/schema/sessionTime.schema.json (canonical)
message SessionTime {
  int64 created = 1;   // Unix timestamp (ms)
  int64 updated = 2;
  optional int64 compacting = 3;
  optional int64 archived = 4;
}

// Session summary statistics
// Source: submodules/opencode/schema/sessionSummary.schema.json (canonical)
message SessionSummary {
  int32 additions = 1;
  int32 deletions = 2;
  int32 files = 3;
  repeated FileDiff diffs = 4;
}

// File diff information
// Source: submodules/opencode/schema/fileDiff.schema.json (canonical)
message FileDiff {
  string file = 1;
  string before = 2;
  string after = 3;
  int32 additions = 4;
  int32 deletions = 5;
}

// Session share information
// Source: submodules/opencode/schema/sessionShare.schema.json (canonical)
message SessionShare {
  string url = 1;
}

// Session revert state
// Source: submodules/opencode/schema/sessionRevert.schema.json (canonical)
message SessionRevert {
  string message_id = 1;
  optional string part_id = 2;
  optional string snapshot = 3;
  optional string diff = 4;
}

// Permission action enum
// Source: submodules/opencode/schema/permissionAction.schema.json (canonical)
enum PermissionAction {
  PERMISSION_ACTION_UNSPECIFIED = 0;
  ALLOW = 1;
  DENY = 2;
  ASK = 3;
}

// Permission rule
// Source: submodules/opencode/schema/permissionRule.schema.json (canonical)
message PermissionRule {
  string permission = 1;
  string pattern = 2;
  PermissionAction action = 3;
}

// Permission ruleset
// Source: submodules/opencode/schema/permissionRuleset.schema.json (canonical)
message PermissionRuleset {
  repeated PermissionRule rules = 1;
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

## JSON Schema Cross-Reference

### SessionInfo Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `id` | `id` | string | Required; pattern `^ses_` (session ID prefix) |
| `project_id` | `projectID` | string | Required; naming: snake_case vs camelCase |
| `directory` | `directory` | string | Required in both |
| `parent_id` | `parentID` | string | Optional; pattern `^ses_` (session ID prefix) |
| `summary` | `summary` | SessionSummary | Optional in both |
| `share` | `share` | SessionShare | Optional in both |
| `title` | `title` | string | Required in both |
| `version` | `version` | string | Required in both |
| `time` | `time` | SessionTime | Required in both |
| `permission` | `permission` | PermissionRuleset | Optional in both |
| `revert` | `revert` | SessionRevert | Optional in both |

### SessionTime Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `created` | `created` | number (int64) | Unix timestamp in milliseconds |
| `updated` | `updated` | number (int64) | Unix timestamp in milliseconds |
| `compacting` | `compacting` | number (int64) | Optional in both |
| `archived` | `archived` | number (int64) | Optional in both |

### SessionSummary Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `additions` | `additions` | number (int32) | Required in both |
| `deletions` | `deletions` | number (int32) | Required in both |
| `files` | `files` | number (int32) | Required in both |
| `diffs` | `diffs` | array[FileDiff] | Optional in both |

### FileDiff Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `file` | `file` | string | Required in both |
| `before` | `before` | string | Required in both |
| `after` | `after` | string | Required in both |
| `additions` | `additions` | number (int32) | Required in both |
| `deletions` | `deletions` | number (int32) | Required in both |

### SessionShare Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `url` | `url` | string | Required in both |

### SessionRevert Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `message_id` | `messageID` | string | Required; naming: snake_case vs camelCase |
| `part_id` | `partID` | string | Optional; naming: snake_case vs camelCase |
| `snapshot` | `snapshot` | string | Optional in both |
| `diff` | `diff` | string | Optional in both |

### PermissionRule Message

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `permission` | `permission` | string | Required in both |
| `pattern` | `pattern` | string | Required in both |
| `action` | `action` | PermissionAction | Required in both |

### PermissionAction Enum

| Protobuf Value | JSON Schema Value | Notes |
|----------------|------------------|-------|
| `ALLOW` | `"allow"` | String enum in JSON |
| `DENY` | `"deny"` | String enum in JSON |
| `ASK` | `"ask"` | String enum in JSON |

### PermissionRuleset Message

Note: PermissionRuleset in JSON Schema is an array of PermissionRule. In protobuf, this is represented as:

```protobuf
message PermissionRuleset {
  repeated PermissionRule rules = 1;
}
```

**Why wrapped in a message?** Protobuf doesn't support top-level repeated fields in message definitions, so we wrap the array in a message. The JSON Schema correctly represents it as `type: "array"` with `items: { $ref: "permissionRule.schema.json" }`.

---

## TODO

- [x] Create `sessionInfo.schema.json` for session metadata
- [x] Create related session schemas (time, summary, share, revert, fileDiff, permission)
- [x] Validate schemas with `bun run generate:schemas`
- [x] Add JSON Schema cross-reference table
- [x] Refactor TypeScript source to use generated validators
- [x] Typecheck and tests pass after refactoring
- [ ] Validate against actual session API responses
- [ ] Document session lifecycle (create, update, delete)
