# Protobuf Schema Documentation

**Version:** 1.3.0  
**Last Updated:** 2026-01-05

---

## Overview

This directory contains the **protobuf schema definitions** for the Tauri-Blazor desktop client's gRPC communication layer. The schemas define the data contract between:

- **Blazor Frontend (C#)** ↔ **Rust Backend (client-core)** ↔ **OpenCode Server (HTTP/SSE)**

## Source of Truth

**As of 2026-01-04, the canonical source of truth for Model and Provider schemas is:**

```
submodules/opencode/schema/*.schema.json
```

These JSON Schema files are the authoritative specification. The protobuf definitions are derived from them.

| JSON Schema | Protobuf | Status |
|-------------|----------|--------|
| `modelInfo.schema.json` | `model.proto` | ✅ Complete |
| `providerInfo.schema.json` | `provider.proto` | ✅ Complete |
| `auth.schema.json` | `auth.proto` | ✅ Complete |
| `sessionInfo.schema.json` | `session.proto` | ✅ Complete |
| `*Part.schema.json` + `*Message.schema.json` + `*Error.schema.json` (20 files) | `message.proto` | ✅ Complete |
| `toolState.schema.json` + 8 related | `tool.proto` | ✅ Complete |
| (none yet) | `agent.proto` | ⏳ WIP |
| (none yet) | `event.proto` | ⏳ WIP |

**GitHub Issue:** [anomalyco/opencode#6879](https://github.com/anomalyco/opencode/issues/6879)  
**PoC Branch:** [TonyMarkham/opencode@feature/json-schema-poc](https://github.com/TonyMarkham/opencode/tree/feature/json-schema-poc)

---

## File Organization

```
proto/
├── model.proto         - 1. Model metadata, capabilities, options, cost, limits
├── provider.proto      - 2. Provider management, SDK options, source enum
├── auth.proto          - 3. Authentication per provider (API key vs OAuth)
├── session.proto       - 4. Sessions, tabs, working directory
├── message.proto       - 5. User/assistant messages, attachments
├── tool.proto          - 6. Tool execution state, permissions
├── agent.proto         - 7. Agent listing and metadata
├── event.proto         - 8. SSE event streaming (gRPC translation)
└── opencode.proto      - 9. Main service definition (aggregates all services)
```

## Documentation Files

| File | Description | Status |
|------|-------------|--------|
| [01-model.md](./01-model.md) | Model metadata, capabilities, cost, limits | ✅ Complete |
| [02-provider.md](./02-provider.md) | Provider management, SDK options | ✅ Complete |
| [03-auth.md](./03-auth.md) | Authentication per provider | ✅ Complete |
| [04-session.md](./04-session.md) | Session/tab management | ✅ Complete |
| [05-message.md](./05-message.md) | User/assistant messages | ✅ Complete |
| [06-tool.md](./06-tool.md) | Tool execution state | ✅ Complete |
| [07-agent.md](./07-agent.md) | Agent listing | ⏳ WIP |
| [08-event.md](./08-event.md) | SSE event streaming | ⏳ WIP |
| [09-opencode.md](./09-opencode.md) | Main service aggregator | ⏳ WIP |

## Adding New Schemas

See **[SCHEMA_DEVELOPMENT_PROCESS.md](./SCHEMA_DEVELOPMENT_PROCESS.md)** for the step-by-step process to:

1. Identify source TypeScript/Zod definitions
2. Create JSON Schema files
3. Validate and generate validators
4. Update protobuf documentation
5. Cross-reference JSON ↔ Proto fields

---

## Build Order / Dependency Graph

```
model.proto (foundation - no dependencies)
    │
    ▼
provider.proto (imports model.proto)
    │
    ▼
auth.proto (references provider IDs)
    │
    ▼
session.proto (imports model.proto, provider.proto)
    │
    ├──▶ message.proto (imports session.proto)
    │        │
    │        ▼
    │    tool.proto (imported by message.proto)
    │
    └──▶ agent.proto (standalone, referenced by session)

event.proto (imports tool.proto for ToolCallState)
    │
    ▼
opencode.proto (aggregates all services)
```

**Recommended build order:**
1. `model.proto` - Foundation, no imports
2. `provider.proto` - Imports model
3. `auth.proto` - References provider IDs
4. `session.proto` - Imports model, provider
5. `tool.proto` - Standalone tool state
6. `message.proto` - Imports session, tool
7. `agent.proto` - Standalone
8. `event.proto` - Imports tool
9. `opencode.proto` - Aggregates all

---

## Design Principles

1. **Type Safety Over Flexibility** - Use proper protobuf types (enums, messages) instead of generic maps where possible
2. **Composition Over Duplication** - `TabInfo` contains `SessionInfo`, `ModelSelection` contains `ProviderInfo`
3. **Separation of Concerns** - Stable data (provider metadata) vs volatile data (auth status)
4. **Per-Provider Auth** - Each provider has its own auth mode, not global
5. **UI-Centric Models** - Reduce lookups, cache intelligently, support rich UI features
6. **JSON Schema Alignment** - Protobuf definitions should match JSON Schema structure where applicable

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

---

## Version History

### 1.4.0 (2026-01-05)

- **Message schemas completed** (20 new JSON Schema files)
- Added JSON Schema for Part types: `textPart`, `reasoningPart`, `snapshotPart`, `patchPart`, `agentPart`, `compactionPart`, `subtaskPart`, `stepStartPart`, `stepFinishPart`, `retryPart`, `part`
- Added JSON Schema for Message types: `userMessage`, `assistantMessage`, `message`
- Added JSON Schema for Error types: `apiError`, `providerAuthError`, `unknownError`, `outputLengthError`, `abortedError`, `messageError`
- Refactored `message-v2.ts` to use generated validators
- Updated `05-message.md` with comprehensive cross-reference tables (17 schemas verified)

### 1.3.0 (2026-01-05)

- **Tool schemas completed** (`toolState.schema.json` and 8 related schemas)
- Added JSON Schema for: `ToolState`, `ToolStatePending`, `ToolStateRunning`, `ToolStateCompleted`, `ToolStateError`, `ToolPart`, `PermissionRequest`, `PermissionReply`, `PermissionToolContext`
- Refactored `message-v2.ts` and `permission/next.ts` to use generated validators
- Updated `tool.proto` with comprehensive cross-reference tables

### 1.2.0 (2026-01-05)

- **Session schemas completed** (`sessionInfo.schema.json` and 9 related schemas)
- Added JSON Schema for: `SessionInfo`, `SessionTime`, `SessionSummary`, `SessionShare`, `SessionRevert`, `FileDiff`, `PermissionRule`, `PermissionRuleset`, `PermissionAction`, `SessionList`
- Updated `session.proto` to match JSON Schema structure
- Added comprehensive cross-reference tables in `04-session.md`

### 1.1.0 (2026-01-04)

- **Source of truth changed to JSON Schema files** (`schema/*.schema.json`)
- Split monolithic `PROTO_SCHEMA.md` into individual files per proto
- Updated `ModelInfo` and `ProviderInfo` to match JSON Schema
- Reordered sections: auth.proto now follows provider.proto

### 1.0.0 (2026-01-04)

- Initial schema definition
- 9 proto files with logical domain grouping
- Full service interface (40+ messages, 6 services, 15+ RPC methods)
