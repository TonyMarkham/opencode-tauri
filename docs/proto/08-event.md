# Event Streaming (`event.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

Translate OpenCode server SSE events into gRPC streams for real-time updates.

---

## Source of Truth

**Note:** No dedicated JSON Schema yet for event types.

**Future work:** Create JSON Schema for event types to align with model/provider pattern.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.event;

import "tool.proto";

// Global event stream (SSE → gRPC translation)
message GlobalEvent {
  string directory = 1;
  oneof event {
    MessageUpdated message_updated = 2;
    MessagePartUpdated message_part_updated = 3;
    PermissionCreated permission_created = 4;
  }
}

// Event: Message metadata updated
message MessageUpdated {
  string message_id = 1;
  string role = 2;
  optional string finish = 3;       // null | "stop" | "error"
  int64 created_at = 4;
  int64 updated_at = 5;
  optional TokenUsage tokens = 6;
}

message TokenUsage {
  uint64 input = 1;
  uint64 output = 2;
  uint64 reasoning = 3;
}

// Event: Streaming content chunk
message MessagePartUpdated {
  string message_id = 1;
  oneof part {
    TextPart text = 2;
    ReasoningPart reasoning = 3;
    ToolPart tool = 4;
  }
}

message TextPart {
  string text = 1;  // Accumulated text (not delta)
}

message ReasoningPart {
  string text = 1;  // Accumulated reasoning
}

message ToolPart {
  ToolCallState state = 1;
}

// Event: Permission request created
message PermissionCreated {
  PermissionRequest permission = 1;
}

message Empty {}
```

---

## Service Definition

```protobuf
service EventService {
  rpc SubscribeGlobalEvents(Empty) returns (stream GlobalEvent);
}
```

---

## Maps to OpenCode Server

- `EventService.SubscribeGlobalEvents` → `GET /global/event` (SSE stream)
  - Backend subscribes to SSE, translates events to gRPC stream
  - Events: `message.updated`, `message.part.updated`, `permission.created`

---

## Design Notes

**SSE to gRPC Translation:**

The OpenCode server uses Server-Sent Events (SSE) for real-time streaming. The Rust backend (client-core) subscribes to SSE and translates events to gRPC streams for the Blazor frontend.

```
OpenCode Server (SSE) → Rust Backend → gRPC Stream → Blazor Frontend
```

**Event Types:**

| SSE Event | gRPC Message | Description |
|-----------|--------------|-------------|
| `message.updated` | `MessageUpdated` | Message metadata changed (tokens, finish status) |
| `message.part.updated` | `MessagePartUpdated` | Streaming content chunk (text, reasoning, tool) |
| `permission.created` | `PermissionCreated` | Tool needs user permission |

**Accumulated vs Delta:**

- Text and reasoning parts are **accumulated** (full content so far)
- NOT deltas (unlike some streaming APIs)
- Simplifies client rendering (just replace, don't append)

**Directory Context:**

- Each event includes the working directory
- Allows filtering events by directory (multi-session support)

---

## TODO

- [ ] Create `globalEvent.schema.json` for event envelope
- [ ] Document SSE event format from server
- [ ] Validate against actual SSE events
- [ ] Document reconnection/retry logic
