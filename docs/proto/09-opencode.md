# Main Service Definition (`opencode.proto`)

**Status:** ⏳ Work in Progress  
**Last Updated:** 2026-01-04

---

## Purpose

Aggregate all sub-services into a single gRPC service definition. This is the entry point for the Blazor frontend to communicate with the Rust backend.

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

## TODO

- [ ] Implement all RPC methods in Rust backend
- [ ] Generate C# client from proto files
- [ ] Test end-to-end with Blazor frontend
- [ ] Document error handling patterns
