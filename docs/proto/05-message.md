# Message & Content (`message.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

Handle user/assistant messages with text, reasoning, file attachments, and token tracking.

---

## Source of Truth

**Note:** No dedicated JSON Schema yet for message types.

**Future work:** Create JSON Schema for message types to align with model/provider pattern.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.message;

import "session.proto";
import "tool.proto";

// Outbound: User sends message to session
message SendMessageRequest {
  string session_id = 1;
  repeated MessagePart parts = 2;
  optional ModelSelection model = 3;
  optional string agent = 4;
}

message MessagePart {
  oneof content {
    TextPart text = 1;
    FilePart file = 2;
  }
}

message TextPart {
  string text = 1;
}

message FilePart {
  string mime = 1;
  optional string filename = 2;
  string url = 3;  // Data URI (base64 encoded)
}

// Inbound: Message display state
message DisplayMessage {
  string message_id = 1;
  string role = 2;  // "user" | "assistant" | "system"
  repeated string text_parts = 3;
  repeated string reasoning_parts = 4;
  optional uint64 tokens_input = 5;
  optional uint64 tokens_output = 6;
  optional uint64 tokens_reasoning = 7;
  repeated ToolCallState tool_calls = 8;
}

message MessageList {
  repeated DisplayMessage messages = 1;
}

// Abort streaming response
message AbortSessionRequest {
  string session_id = 1;
}

message GetMessagesRequest {
  string session_id = 1;
}

message Empty {}
```

---

## Service Definition

```protobuf
service MessageService {
  rpc SendMessage(SendMessageRequest) returns (Empty);
  rpc GetMessages(GetMessagesRequest) returns (MessageList);
  rpc AbortSession(AbortSessionRequest) returns (Empty);
}
```

---

## Maps to OpenCode Server

- `MessageService.SendMessage` → `POST /session/{id}/message`
- `MessageService.GetMessages` → `GET /session/{id}/message`
- `MessageService.AbortSession` → `POST /session/{id}/abort`

---

## Design Notes

**Message Parts:**

- Messages can contain multiple parts (text, files)
- Files are sent as data URIs (base64 encoded)
- MIME type determines handling (image, PDF, etc.)

**Display Message:**

- Aggregates text and reasoning parts for UI display
- Tracks token usage (input, output, reasoning)
- References tool calls for inline display

**Streaming:**

- Messages stream via SSE events (see `event.proto`)
- `AbortSession` cancels in-progress streaming

---

## TODO

- [ ] Create `messageInfo.schema.json` for message metadata
- [ ] Create `messagePart.schema.json` for content parts
- [ ] Validate against actual message API responses
- [ ] Document streaming message flow
