# Main Service Definition (`opencode.proto`)

**Status:** ✅ Complete  
**Last Updated:** 2026-01-05

---

## Purpose

Aggregate all sub-services into a single gRPC service definition. This is the entry point for the Blazor frontend to communicate with the Rust backend.

---

## Source of Truth

This proto file aggregates all other proto definitions. The source of truth for each imported proto is its corresponding JSON Schema files:

| Import | Proto Doc | JSON Schema Files | Status |
|--------|-----------|-------------------|--------|
| `model.proto` | [01-model.md](./01-model.md) | `modelInfo.schema.json`, `modelCost.schema.json`, `modelCapabilities.schema.json`, etc. | ✅ Complete |
| `provider.proto` | [02-provider.md](./02-provider.md) | `providerInfo.schema.json`, `providerOptions.schema.json`, etc. | ✅ Complete |
| `auth.proto` | [03-auth.md](./03-auth.md) | `auth.schema.json`, `apiAuth.schema.json`, `oauth.schema.json`, etc. | ✅ Complete |
| `session.proto` | [04-session.md](./04-session.md) | `sessionInfo.schema.json`, `sessionTime.schema.json`, `sessionSummary.schema.json`, etc. | ✅ Complete |
| `message.proto` | [05-message.md](./05-message.md) | `message.schema.json`, `userMessage.schema.json`, `assistantMessage.schema.json`, `*Part.schema.json`, etc. | ✅ Complete |
| `tool.proto` | [06-tool.md](./06-tool.md) | `toolState.schema.json`, `toolPart.schema.json`, `permissionRequest.schema.json`, etc. | ✅ Complete |
| `agent.proto` | [07-agent.md](./07-agent.md) | `agentInfo.schema.json`, `agentModel.schema.json` | ✅ Complete |
| `event.proto` | [08-event.md](./08-event.md) | `event.schema.json`, `globalEvent.schema.json`, `*Event.schema.json`, `sessionStatus.schema.json` | ✅ Complete |

**All imported proto files have corresponding JSON Schema definitions.**

---

## Complete Service Interface

```protobuf
syntax = "proto3";
package opencode;

import "model.proto";
import "provider.proto";
import "auth.proto";
import "session.proto";
import "message.proto";
import "tool.proto";
import "agent.proto";
import "event.proto";

// Main OpenCode gRPC service (aggregates all sub-services)
service OpenCodeService {
  // Session Management
  rpc ListSessions(Empty) returns (SessionList);
  rpc CreateSession(CreateSessionRequest) returns (SessionInfo);
  rpc DeleteSession(DeleteSessionRequest) returns (Empty);
  rpc UpdateSessionDirectory(UpdateDirectoryRequest) returns (Empty);

  // Provider Management
  rpc GetProviders(Empty) returns (ProviderList);

  // Message Operations
  rpc SendMessage(SendMessageRequest) returns (Empty);
  rpc GetMessages(GetMessagesRequest) returns (MessageList);
  rpc AbortSession(AbortSessionRequest) returns (Empty);

  // Tool Permissions
  rpc RespondToPermission(PermissionResponse) returns (Empty);

  // Agent Management
  rpc ListAgents(Empty) returns (AgentList);

  // Authentication
  rpc GetAuthStatus(Empty) returns (AuthStatus);
  rpc GetProviderAuth(ProviderAuthRequest) returns (ProviderAuthInfo);
  rpc GetProviderStatus(Empty) returns (ProviderStatus);
  rpc SwitchProviderAuth(SwitchProviderAuthRequest) returns (Empty);

  // Event Streaming
  rpc SubscribeGlobalEvents(Empty) returns (stream GlobalEvent);
  rpc SubscribeEvents(Empty) returns (stream Event);
}
```

---

## RPC Method Summary

| Category | Method | Request | Response | Server Endpoint |
|----------|--------|---------|----------|-----------------|
| Session | `ListSessions` | `Empty` | `SessionList` | `GET /session` |
| Session | `CreateSession` | `CreateSessionRequest` | `SessionInfo` | `POST /session` |
| Session | `DeleteSession` | `DeleteSessionRequest` | `Empty` | `DELETE /session/{id}` |
| Session | `UpdateSessionDirectory` | `UpdateDirectoryRequest` | `Empty` | Header: `x-opencode-directory` |
| Provider | `GetProviders` | `Empty` | `ProviderList` | `GET /config/providers` |
| Message | `SendMessage` | `SendMessageRequest` | `Empty` | `POST /session/{id}/message` |
| Message | `GetMessages` | `GetMessagesRequest` | `MessageList` | `GET /session/{id}/message` |
| Message | `AbortSession` | `AbortSessionRequest` | `Empty` | `POST /session/{id}/abort` |
| Tool | `RespondToPermission` | `PermissionResponse` | `Empty` | `POST /session/{id}/permissions/{id}` |
| Agent | `ListAgents` | `Empty` | `AgentList` | `GET /agent` |
| Auth | `GetAuthStatus` | `Empty` | `AuthStatus` | `~/.local/share/opencode/auth.json` |
| Auth | `GetProviderAuth` | `ProviderAuthRequest` | `ProviderAuthInfo` | `~/.local/share/opencode/auth.json` |
| Auth | `GetProviderStatus` | `Empty` | `ProviderStatus` | `GET /provider` |
| Auth | `SwitchProviderAuth` | `SwitchProviderAuthRequest` | `Empty` | Internal state |
| Event | `SubscribeGlobalEvents` | `Empty` | `stream GlobalEvent` | `GET /global/event` (SSE) |
| Event | `SubscribeEvents` | `Empty` | `stream Event` | `GET /event` (SSE) |

---

## Design Notes

**Why a single aggregated service?**

- Simpler client code (one channel, one client)
- Easier to manage connection lifecycle
- gRPC-Web compatibility (Blazor WebAssembly)

**Alternative: Separate services**

Could split into `SessionService`, `ProviderService`, etc. but adds complexity for the Blazor client.

---

## Implementation Notes

### Rust (client-core)

```toml
[dependencies]
tonic = "0.10"
prost = "0.12"

[build-dependencies]
tonic-build = "0.10"
```

```rust
// build.rs
fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["proto/opencode.proto"],
            &["proto/"],
        )
        .unwrap();
}
```

### C# (Blazor)

```xml
<ItemGroup>
  <PackageReference Include="Grpc.Net.Client" Version="2.60.0" />
  <PackageReference Include="Google.Protobuf" Version="3.25.1" />
  <PackageReference Include="Grpc.Tools" Version="2.60.0" PrivateAssets="All" />
</ItemGroup>

<ItemGroup>
  <Protobuf Include="../../../proto/**/*.proto" GrpcServices="Client" />
</ItemGroup>
```

```csharp
var channel = GrpcChannel.ForAddress("http://localhost:50051");
var client = new OpenCodeService.OpenCodeServiceClient(channel);

var sessions = await client.ListSessionsAsync(new Empty());
```

---

## Verification

All imported proto files are now complete with JSON Schema definitions:

- ✅ `model.proto` - 8 JSON Schema files
- ✅ `provider.proto` - 6 JSON Schema files  
- ✅ `auth.proto` - 4 JSON Schema files
- ✅ `session.proto` - 10 JSON Schema files
- ✅ `message.proto` - 20 JSON Schema files
- ✅ `tool.proto` - 9 JSON Schema files
- ✅ `agent.proto` - 2 JSON Schema files
- ✅ `event.proto` - 13 JSON Schema files

**Total: 72+ JSON Schema files defining the complete data contract.**

---

## TODO

- [x] Define all RPC methods
- [x] Map RPC methods to OpenCode server endpoints
- [x] Document all imported proto files with JSON Schema sources
- [ ] Implement all RPC methods in Rust backend
- [ ] Generate C# client from proto files
- [ ] Test end-to-end with Blazor frontend
- [ ] Document error handling patterns
