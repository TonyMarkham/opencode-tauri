# Session Plan: Tauri + Blazor Desktop Client (ADR-0001)

## Goal

Build a new Tauri + Blazor WebAssembly desktop client as an alternative to egui, sharing Rust backend code and following the zero-custom-JavaScript policy.

---

## Session 1: Shared Rust Core & Project Scaffold

### Step 1: Extract Shared Client Core

- ✅ Create workspace at repository root
- ✅ Build production-grade `backend/client-core/` crate from scratch
- ✅ Build production-grade `models/` crate for shared utilities
- ⏭️ Extract from egui (DEFERRED - built fresh code instead, egui now in submodules/opencode-egui/)
- ⏭️ Update egui to use shared crate (DEFERRED - egui unchanged)

### Step 2: Create Tauri-Blazor Directory Structure

- ✅ Create workspace layout at repository root
- ✅ Create workspace Cargo.toml with proper dependencies
- ✅ Create README.md documenting structure
- ⏭️ Set up Tauri project skeleton (`apps/desktop/opencode/`) (DEFERRED to Session 2)

**Status:** ✅ Complete

**Actual Tokens:** ~60K

**Deliverables:**

- ✅ Working `backend/client-core` with discovery + spawn logic
- ✅ Working `models` crate with ErrorLocation utilities
- ✅ egui client unchanged (no breaking changes, now in submodules/opencode-egui/)
- ⏭️ Tauri project structure (deferred to Session 2)

---

## Session 2: Tauri Backend & Server Commands

### Step 1: Implement Tauri State Management

- ✅ Create `apps/desktop/opencode/src/state.rs` for shared app state
- ✅ Refactored from simple Mutex to actor pattern for race-free state management
- ✅ Set up state initialization in main.rs

### Step 2: Implement Server Discovery Commands

- ✅ Create `apps/desktop/opencode/src/commands/server.rs`
- ✅ Implement `discover_server()`, `spawn_server()`, `check_health()`, `stop_server()`
- ✅ Wire commands into Tauri builder
- ✅ Add production-grade logging to all commands

### Step 3: Test Tauri Commands

- ✅ Build Tauri app with minimal HTML frontend
- ✅ Test commands from browser console
- ✅ Verify server discovery/spawn works
- ✅ Test full lifecycle: discover/spawn → health → stop

**Status:** ✅ Complete

**Actual Tokens:** ~120K

**Deliverables:**

- ✅ Tauri commands working for server operations
- ✅ Can discover/spawn OpenCode server from Tauri
- ✅ State management functional (actor pattern, race-free)
- ✅ Production-grade logging throughout
- ✅ Clippy clean with `-D warnings`

---

## Session 3: Blazor Frontend Scaffold & Server Integration

### Step 1: Initialize Blazor WASM Project

- ✅ Create `frontend/` directory at `frontend/desktop/opencode/`
- ✅ Configure Opencode.csproj (.NET 10.0, Radzen 8.4.2, Markdig 0.44.0)
- ✅ Set up Program.cs with Radzen dependency injection
- ✅ Configure publish to `apps/desktop/opencode/frontend/`

### Step 2: Create Server Service Layer

- ✅ Implement `IServerService` interface
- ✅ Implement `ServerService` using `IJSRuntime` (NO custom JS)
- ✅ Create `TauriCommands` constants for type-safe command invocation
- ✅ Create `TauriConstants` for JSInterop configuration
- ✅ Create exception types for error handling (ServerDiscoveryException, ServerSpawnException, etc.)
- ✅ Use snake_case JSON deserialization matching Rust API

### Step 3: Build Basic UI

- ✅ Create Home.razor with server status display (Radzen Card, Alert, Stack)
- ✅ Add discover/spawn/health/stop buttons with Radzen components
- ✅ Test full flow: Blazor → C# → IJSRuntime → Tauri → Rust
- ✅ Wire up NotificationService for success/error messages
- ✅ Implement loading states with IsBusy button prop

**Status:** ✅ Complete

**Actual Tokens:** ~90K

**Deliverables:**

- ✅ Blazor WASM project compiles and publishes
- ✅ Tauri loads Blazor UI successfully
- ✅ Server discovery/spawn working from UI
- ✅ All 4 server commands callable from Blazor
- ✅ Error handling with user-friendly notifications
- ✅ Zero custom JavaScript files (all IJSRuntime)

---

## Session 4: Data Models & gRPC Service Infrastructure

**Goal:** Define comprehensive protobuf schema and gRPC service interface based on OpenCode server JSON Schema definitions. Group data models logically by domain: Sessions, Messages, Tools, Agents, Auth, and Events.

---

### SCOPE CHANGE: Documentation-First Approach ✅

**What actually happened (2026-01-04 to 2026-01-05):**

Instead of writing protobuf directly, we took a more rigorous approach:

1. **Created 72+ JSON Schema files** from OpenCode server TypeScript/Zod types
2. **Generated Zod validators** from schemas (single source of truth)
3. **Refactored OpenCode server** to use generated validators (544 tests pass)
4. **Documented 9 protobuf domains** with cross-reference tables (4,400+ lines)

**Why this was better:**

- JSON Schema is the canonical source (matches OpenCode server exactly)
- Automated validation prevents drift between TypeScript ↔ JSON Schema ↔ Protobuf
- Generated code eliminates manual transcription errors
- Documentation is now traceable to specific schema files

**Result:** Session 4 became a documentation + schema generation phase instead of coding phase.

---

### Design Rationale

**Why This Session Matters:**

This session defines the **complete data contract** between Blazor UI (C#) and Rust backend (client-core). Getting this right is critical because:

1. Changes after implementation are expensive (protobuf breaking changes)
2. Poor data models lead to UI complexity (excessive lookups, stale data)
3. Missing fields discovered later require rework across the stack

**Key Design Decisions:**

1. **JSON Schema as Source of Truth**
   - All protobuf definitions derive from `submodules/opencode/schema/*.schema.json`
   - Generator validates schemas and produces Zod validators
   - TypeScript source refactored to use generated validators
   - **Why:** Single source prevents drift, enables automated validation

2. **Composition Over Duplication**
   - `TabInfo` contains `SessionInfo` (not duplicate fields)
   - `ModelSelection` contains `ProviderInfo` (not just provider_id string)
   - **Why:** Single source of truth, no sync issues between duplicated fields

3. **Stable vs. Volatile Data Separation**
   - `ProviderInfo` (name, source) included in `ModelSelection` - stable, safe to cache
   - `ProviderAuthInfo` (type, expires) queried separately - volatile, must be fresh
   - **Why:** Auth can change (OAuth ↔ API key switch), provider metadata doesn't

4. **Per-Provider Auth (Not Global)**
   - Each provider has its own auth mode (`anthropic`: OAuth, `openai`: API key)
   - Auth tracked per provider_id, not globally
   - **Why:** Server's `auth.json` structure has per-provider auth entries

5. **UI-Centric Data Models**
   - `ModelSelection` includes `ProviderInfo` for clean UI display
   - `ProviderInfo` includes curated models list (not all discoverable models)
   - No extra lookups needed for common UI tasks (model picker, tab header)
   - **Why:** Frontend perspective - reduce boilerplate, improve code clarity

6. **Curated Models Over Discovery**
   - `ProviderInfo.models` contains only configured/curated models (from server)
   - Model discovery UI (add arbitrary models) deferred to Session 5
   - **Why:** Curated list is what user actually wants in model picker (not giant unfiltered list)
   - **Maps to:** OpenCode server `GET /config/providers` endpoint

7. **Comprehensive Tool State**
   - Tool execution includes logs, metadata, timing (learned from opencode-egui audit in submodules/)
   - Supports rich UI (progress indicators, log viewers, timing charts)
   - **Why:** Users need visibility into what tools are doing

8. **Event Streaming Translation**
   - SSE events from OpenCode server translated to gRPC streams
   - Blazor consumes unified event stream (no SSE client needed)
   - **Why:** Consistent API surface, easier testing, better type safety

9. **Typed Model Options (Not Generic Maps)**
   - `ModelOptions` uses proper protobuf types with provider-specific submessages
   - Each provider (OpenAI, Google, Anthropic) has its own strongly-typed options message
   - Universal options (temperature, maxOutputTokens) in separate message
   - **Why:** Type safety in C#, IDE autocomplete, schema clarity, compile-time validation
   - **Trade-off:** More maintenance when providers add options vs. flexible `map<string, string>`
   - **Decision:** Type safety wins - desktop client should validate before sending to server

**Scope Boundaries:**

- ✅ **In Scope:** JSON Schema definitions (72+ files)
- ✅ **In Scope:** Protobuf documentation (9 files, 4,400+ lines)
- ✅ **In Scope:** Cross-reference tables (JSON Schema ↔ Protobuf)
- ❌ **Out of Scope:** Actual protobuf `.proto` files (Session 4.5)
- ❌ **Out of Scope:** gRPC service implementation (Session 4.5)
- ❌ **Out of Scope:** OpenCode server HTTP calls (Session 4.5)
- ❌ **Out of Scope:** UI implementation (Session 4.5)

---

### Documentation Completed

**See [docs/proto/](./docs/proto/) for complete documentation**

**9 Proto Documentation Files (4,400+ lines):**

1. `01-model.md` - Model metadata, capabilities, cost, limits (265 lines)
2. `02-provider.md` - Provider management, SDK options (146 lines)
3. `03-auth.md` - Authentication per provider (177 lines)
4. `04-session.md` - Session/tab management (300 lines)
5. `05-message.md` - User/assistant messages (780 lines) ⭐ largest
6. `06-tool.md` - Tool execution state (317 lines)
7. `07-agent.md` - Agent listing (207 lines)
8. `08-event.md` - SSE event streaming (479 lines)
9. `09-opencode.md` - Main service aggregator (196 lines)

**Supporting Documentation:**

- `README.md` - Overview, build order, version history (231 lines)
- `SCHEMA_DEVELOPMENT_PROCESS.md` - Workflow guide (1,031 lines)
- `NEXT_SESSION_PROMPT.md` - Session 4.5 prompt (305 lines)

**JSON Schema Files Created:**

| Domain | Schema Files | Status |
|--------|-------------|--------|
| Model | 8 files | ✅ Complete |
| Provider | 6 files | ✅ Complete |
| Auth | 4 files | ✅ Complete |
| Session | 10 files | ✅ Complete |
| Message | 20 files | ✅ Complete |
| Tool | 9 files | ✅ Complete |
| Agent | 2 files | ✅ Complete |
| Event | 13 files | ✅ Complete |
| **Total** | **72+ files** | ✅ Complete |

---

**Status:** ✅ Complete (Documentation Phase)

**Actual Tokens:** ~60K (documentation + schema validation)

**Deliverables:**

- ✅ 9 protobuf documentation files (4,400+ lines)
- ✅ 72+ JSON Schema files (canonical source of truth)
- ✅ Cross-reference tables for all types (JSON ↔ Proto)
- ✅ Schema generator validated all schemas
- ✅ OpenCode server refactored to use generated validators
- ✅ All 544 OpenCode server tests pass
- ✅ Build succeeds for 11 platforms
- ⏭️ Actual `.proto` files (deferred to Session 4.5)
- ⏭️ gRPC service implementation (deferred to Session 4.5)

---

## Session 4.5: gRPC Communication Layer (Backend Only)

**Goal:** Complete working gRPC layer with all services implemented. **NO GUI** - testable via grpcurl or simple C# console app.

**Philosophy:** Session 4.5 is purely about the **communication layer**. Building actual GUI elements is Session 5 (completely separate).

---

### Step 1: Create Protobuf Files from Documentation

**Input:** Documentation in `docs/proto/*.md` (4,400+ lines, all complete)

**Output:** Actual `.proto` files in `proto/` directory

**Tasks:**

1. Create `proto/model.proto` from `docs/proto/01-model.md`
2. Create `proto/provider.proto` from `docs/proto/02-provider.md`
3. Create `proto/auth.proto` from `docs/proto/03-auth.md`
4. Create `proto/session.proto` from `docs/proto/04-session.md`
5. Create `proto/message.proto` from `docs/proto/05-message.md`
6. Create `proto/tool.proto` from `docs/proto/06-tool.md`
7. Create `proto/agent.proto` from `docs/proto/07-agent.md`
8. Create `proto/event.proto` from `docs/proto/08-event.md`
9. Create `proto/opencode.proto` from `docs/proto/09-opencode.md`

**Technical Details:**

- Follow dependency order: `model` → `provider` → `auth` → `session` → `tool` → `message` → `agent` → `event` → `opencode`
- Use `import` statements for dependencies
- Add service definitions to `opencode.proto`
- Verify with `protoc` compiler

**Estimated:** ~20K tokens (mostly copy-paste from docs with syntax adjustments)

---

### Step 2: Generate Code (Rust + C#)

**Rust (client-core):**

1. Update `backend/client-core/build.rs` with `tonic-build` configuration
2. Add dependencies: `tonic`, `prost`, `tokio`
3. Run `cargo build` to generate Rust code from protos
4. Verify generated code compiles

**C# (Blazor):**

1. Update `frontend/desktop/opencode/Opencode.csproj` with `Grpc.Tools`
2. Add `<Protobuf>` items for all `.proto` files
3. Run `dotnet build` to generate C# client code
4. Verify generated code compiles

**Estimated:** ~10K tokens (build configuration + verification)

---

### Step 3: Implement gRPC Services in client-core

**Service Implementation (Rust):**

Create service handlers that call OpenCode server HTTP REST API:

1. **SessionService** - `ListSessions`, `CreateSession`, `DeleteSession`, `UpdateSessionDirectory`
2. **ProviderService** - `GetProviders` (GET /config/providers)
3. **MessageService** - `SendMessage`, `GetMessages`, `AbortSession`
4. **PermissionService** - `RespondToPermission`
5. **AgentService** - `ListAgents` (GET /agent)
6. **AuthService** - `GetAuthStatus`, `GetProviderAuth`, `GetProviderStatus`, `SwitchProviderAuth`
7. **EventService** - `SubscribeGlobalEvents` (SSE → gRPC stream translation)

**Implementation Strategy:**

- Start with stubs (return "Not implemented")
- Implement session management first (simplest)
- Add HTTP client for OpenCode server calls
- Implement SSE → gRPC streaming for events (most complex)

**Estimated:** ~70K tokens (core service implementation - this is the hard part)

**Implementation details:**
- HTTP client setup (reqwest or hyper)
- OpenCode server communication (REST API calls)
- SSE event parsing (eventsource or custom parser)
- SSE → gRPC stream translation (most complex)
- Error mapping (HTTP/SSE errors → gRPC Status)
- State management (active sessions, pending requests)

---

### Step 4: Expose gRPC Server Startup from client-core

**Architecture Principle:** Tauri layer is **only** for hosting the webview. All application logic lives in client-core.

**Tasks:**

1. Export `start_grpc_server()` function from `backend/client-core/src/grpc/mod.rs`
2. Update `apps/desktop/opencode/src/main.rs` to call `client_core::grpc::start_grpc_server()`
3. Spawn in tokio task (non-blocking)
4. Handle graceful shutdown via Tauri's drop handler

**Implementation in client-core:**

```rust
// backend/client-core/src/grpc/mod.rs
pub async fn start_grpc_server(addr: impl Into<SocketAddr>) -> Result<(), Error> {
    let server = OpenCodeGrpcServer::new();
    Server::builder()
        .add_service(OpenCodeServiceServer::new(server))
        .serve(addr.into())
        .await?;
    Ok(())
}
```

**Tauri integration (minimal glue code):**

```rust
// apps/desktop/opencode/src/main.rs
use client_core::grpc;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Start gRPC server in background
            tokio::spawn(async {
                if let Err(e) = grpc::start_grpc_server("127.0.0.1:50051").await {
                    error!("gRPC server failed: {}", e);
                }
            });
            Ok(())
        })
        .run(...)
}
```

**Why this is better:**
- Tauri is just a thin wrapper (hosts webview, spawns gRPC server, done)
- client-core is self-contained and testable without Tauri
- Follows the pattern from Sessions 1 & 2 (Tauri **uses** client-core, doesn't contain logic)
- Reusable for future clients (CLI, different GUI framework, etc.)

**Estimated:** ~5K tokens (down from 10K - it's now trivial glue code)

---

### Step 5: Test gRPC Services (No GUI Required)

**Goal:** Verify all services work without building any UI.

**Testing Approaches:**

1. **grpcurl (command-line testing):**
   ```bash
   grpcurl -plaintext localhost:50051 list
   grpcurl -plaintext localhost:50051 opencode.OpenCodeService/ListSessions
   grpcurl -plaintext localhost:50051 opencode.OpenCodeService/GetProviders
   ```

2. **Simple C# console app:**
   ```csharp
   var channel = GrpcChannel.ForAddress("http://localhost:50051");
   var client = new OpenCodeServiceClient(channel);
   
   var sessions = await client.ListSessionsAsync(new Empty());
   Console.WriteLine($"Sessions: {sessions.Sessions.Count}");
   
   var providers = await client.GetProvidersAsync(new Empty());
   Console.WriteLine($"Providers: {providers.Providers.Count}");
   ```

3. **Rust integration tests:**
   - Test each service handler directly
   - Mock OpenCode server responses
   - Verify error handling

**Deliverable:** All 7 services respond correctly. Can create sessions, send messages, stream events - all testable without GUI.

**Estimated:** ~10K tokens (testing + documentation)

---

**Status:** ⏳ Pending

**Estimated Tokens:** ~120K (backend/communication layer only - no GUI)

**Deliverables:**

- ✅ 9 `.proto` files created from documentation
- ✅ Rust code generated via `tonic-build`
- ✅ C# client code generated via `Grpc.Tools`
- ✅ All 7 gRPC services implemented in client-core:
  - SessionService (create, list, delete sessions)
  - ProviderService (list providers with models)
  - MessageService (send, receive, abort messages)
  - PermissionService (respond to permission requests)
  - AgentService (list agents)
  - AuthService (manage auth per provider)
  - EventService (SSE → gRPC streaming)
- ✅ HTTP client for OpenCode server REST API
- ✅ SSE event parsing and gRPC stream translation
- ✅ client-core exports `start_grpc_server()` function
- ✅ Tauri calls `client_core::grpc::start_grpc_server()` (5 lines of glue)
- ✅ gRPC server runs on localhost:50051
- ✅ **Tested via grpcurl or simple C# console app (NO GUI)**
- ✅ All error handling across HTTP/gRPC boundary
- ✅ Production-grade logging throughout

**Out of Scope (moved to Session 5):**
- ❌ Blazor UI components (no Razor files)
- ❌ Tab/session management UI
- ❌ Chat interface
- ❌ Markdown rendering
- ❌ Visual auth/settings UI

---

## Session 5: Blazor UI (Frontend Only)

**Goal:** Build complete Blazor interface that consumes the gRPC services from Session 4.5.

**Prerequisites:** Session 4.5 complete (working gRPC server)

---

### Step 1: Blazor gRPC Client Services

**C# Service Layer:**

Create thin wrappers around gRPC client:

1. `frontend/Services/ISessionService.cs` + `SessionService.cs`
   - ListSessions, CreateSession, DeleteSession
   - Wraps `OpenCodeServiceClient.ListSessionsAsync()` etc.

2. `frontend/Services/IProviderService.cs` + `ProviderService.cs`
   - GetProviders (returns ProviderList with models)

3. `frontend/Services/IMessageService.cs` + `MessageService.cs`
   - SendMessage, GetMessages, AbortSession

4. `frontend/Services/IAgentService.cs` + `AgentService.cs`
   - ListAgents

5. `frontend/Services/IAuthService.cs` + `AuthService.cs`
   - GetAuthStatus, GetProviderAuth, SwitchProviderAuth

6. `frontend/Services/IEventService.cs` + `EventService.cs`
   - SubscribeGlobalEvents (gRPC stream → C# event handlers)

**All use `Grpc.Net.Client` to call localhost:50051.**

**Estimated:** ~25K tokens

---

### Step 2: Session/Tab Management UI

**Razor Components:**

1. `frontend/Pages/Sessions.razor` - Session list sidebar
2. `frontend/Components/TabBar.razor` - Tab navigation
3. `frontend/Components/ModelPicker.razor` - Model/provider dropdown
4. `frontend/Components/AgentPicker.razor` - Agent selection dropdown
5. `frontend/Components/DirectoryInput.razor` - Working directory input

**Functionality:**

- Create new session button
- Delete session button
- Switch between tabs (active tab highlighting)
- Select model per session (calls ProviderService)
- Select agent per session (calls AgentService)
- Set working directory (stored per session)

**Estimated:** ~30K tokens

---

### Step 3: Chat UI with Message Streaming

**Razor Components:**

1. `frontend/Pages/Chat.razor` - Main chat page
2. `frontend/Components/MessageList.razor` - Scrollable message list
3. `frontend/Components/MessageInput.razor` - Text input + send button
4. `frontend/Components/MessageRenderer.razor` - Renders message parts
5. `frontend/Components/ToolCallView.razor` - Tool execution display
6. `frontend/Services/MarkdownService.cs` - Markdig wrapper

**Functionality:**

- Display user/assistant messages
- Stream assistant responses in real-time (EventService → UI updates)
- Render 11 different message part types:
  - TextPart → Markdown rendering
  - ReasoningPart → Collapsible section
  - ToolPart → Tool execution state (pending/running/completed/error)
  - FilePart → File attachments
  - StepStartPart/StepFinishPart → Step markers
  - etc.
- Handle errors (MessageError union types)
- Loading states
- Abort button (calls MessageService.AbortSession)

**Estimated:** ~50K tokens (most complex UI component)

---

### Step 4: Provider Status & Auth UI

**Razor Components:**

1. `frontend/Pages/Settings.razor` - Settings page
2. `frontend/Components/ProviderStatus.razor` - Connected providers list
3. `frontend/Components/AuthModeToggle.razor` - OAuth vs API key switch

**Functionality:**

- Display all providers (from ProviderService)
- Show auth status per provider (OAuth token expiry, API key present)
- Switch auth modes (calls AuthService.SwitchProviderAuth)
- Handle auth errors (ProviderAuthError display)

**Estimated:** ~20K tokens

---

**Status:** ⏳ Pending

**Estimated Tokens:** ~125K (UI only - backend from Session 4.5)

**Deliverables:**

- ✅ Blazor gRPC client services (6 services)
- ✅ Session/tab management UI (create, switch, delete)
- ✅ Model picker UI (dropdown with providers + models)
- ✅ Agent picker UI (dropdown with agents)
- ✅ Chat interface with streaming messages
- ✅ Markdown rendering (Markdig)
- ✅ Tool call visualization (11 part types)
- ✅ Provider status UI
- ✅ Auth mode switching UI
- ✅ All error states and loading indicators
- ✅ Production-ready Blazor app

---

## Session 6: Polish, Testing & Documentation

**Goal:** Make it production-ready and ship it.

### Step 1: Error Handling & Edge Cases

- Create `Settings.razor` page
- Add model selection UI
- Add provider configuration
- Add theme/UI preferences

### Step 3: Persistence

- Implement settings save/load
- Store auth tokens securely
- Session restoration

**Status:** ⏳ Pending

**Estimated Tokens:** ~100K

**Deliverables:**

- ✅ OAuth authentication working
- ✅ Settings panel functional
- ✅ Preferences persisted across restarts

---

## Session 6: Polish, Testing & Documentation

### Step 1: Cross-Platform Testing

- Test on macOS, Windows, Linux
- Fix platform-specific issues
- Optimize performance

### Step 2: Build System Integration

- Create justfile for build orchestration
- Integrate with monorepo tooling (turbo.json)
- Set up packaging/bundling

### Step 3: Documentation

- Update README with usage instructions
- Document differences from opencode-egui client (submodules/opencode-egui/)
- Create "Choosing a Desktop Client" guide

**Status:** ⏳ Pending

**Estimated Tokens:** ~80K

**Deliverables:**

- ✅ Client works on all platforms
- ✅ Build process streamlined
- ✅ Documentation complete

---

## Success Criteria

### Session 1

- [x] `backend/client-core` builds successfully with clippy `-D warnings`
- [x] Production-grade error handling with ErrorLocation tracking
- [x] Discovery module complete (discover, stop_pid, check_health)
- [x] Spawn module complete (spawn_and_wait with exponential backoff)
- [x] Zero magic numbers, all DRY, full rustdoc
- [x] egui client unchanged (no breaking changes)
- [ ] Tauri project structure created (deferred to Session 2)

### Session 2

- [x] Tauri commands for server operations work
- [x] Can discover running OpenCode server
- [x] Can spawn new OpenCode server
- [x] Actor-based state management (race-free)
- [x] Production-grade logging
- [x] Clippy clean with `-D warnings`

### Session 3

- [x] Blazor WASM compiles and loads in Tauri
- [x] Server discovery UI works
- [x] No custom JavaScript files created
- [x] All 4 Tauri commands callable from UI
- [x] Error notifications working
- [x] Loading states visual feedback

### Session 4 (Documentation Phase)

**Data Modeling (JSON Schema):**

- [x] Model schemas complete (8 files: modelInfo, modelCapabilities, modelCost, modelLimits, modelStatus, modelAPI, ioCapabilities, cacheCost)
- [x] Provider schemas complete (6 files: providerInfo, providerSource, providerOptions, providerList)
- [x] Auth schemas complete (4 files: auth, oauth, apiAuth, wellKnownAuth)
- [x] Session schemas complete (10 files: sessionInfo, sessionTime, sessionSummary, sessionShare, sessionRevert, fileDiff, permissionRule, permissionRuleset, permissionAction, sessionList)
- [x] Message schemas complete (20 files: message, userMessage, assistantMessage, 10 part types, 6 error types)
- [x] Tool schemas complete (9 files: toolState, 4 state variants, toolPart, permissionRequest, permissionReply, permissionToolContext)
- [x] Agent schemas complete (2 files: agentInfo, agentModel)
- [x] Event schemas complete (13 files: event, globalEvent, sessionStatus, 4 message events, 4 session events, 2 permission events)

**Documentation:**

- [x] `docs/proto/01-model.md` - Model metadata (265 lines)
- [x] `docs/proto/02-provider.md` - Provider management (146 lines)
- [x] `docs/proto/03-auth.md` - Authentication (177 lines)
- [x] `docs/proto/04-session.md` - Session management (300 lines)
- [x] `docs/proto/05-message.md` - Messages (780 lines)
- [x] `docs/proto/06-tool.md` - Tool execution (317 lines)
- [x] `docs/proto/07-agent.md` - Agents (207 lines)
- [x] `docs/proto/08-event.md` - Event streaming (479 lines)
- [x] `docs/proto/09-opencode.md` - Service aggregator (196 lines)
- [x] `docs/proto/README.md` - Overview and index (231 lines)
- [x] `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` - Workflow guide (1,031 lines)

**Validation:**

- [x] All 72+ JSON schemas validated with `bun run generate:schemas`
- [x] Generated Zod validators match original TypeScript
- [x] OpenCode server refactored to use generated validators
- [x] All 544 OpenCode server tests pass
- [x] Build succeeds for 11 platforms

### Session 4.5 (Implementation Phase)

**Protobuf Files:**

- [ ] `proto/model.proto` created from documentation
- [ ] `proto/provider.proto` created from documentation
- [ ] `proto/auth.proto` created from documentation
- [ ] `proto/session.proto` created from documentation
- [ ] `proto/message.proto` created from documentation
- [ ] `proto/tool.proto` created from documentation
- [ ] `proto/agent.proto` created from documentation
- [ ] `proto/event.proto` created from documentation
- [ ] `proto/opencode.proto` created from documentation (aggregator)

**Code Generation:**

- [ ] Rust code generated via `tonic-build` (prost + tonic)
- [ ] C# client code generated via `Grpc.Tools`
- [ ] Generated code compiles in both languages

**gRPC Services (Rust):**

- [ ] SessionService implemented (ListSessions, CreateSession, DeleteSession, UpdateSessionDirectory)
- [ ] ProviderService implemented (GetProviders)
- [ ] MessageService implemented (SendMessage, GetMessages, AbortSession)
- [ ] PermissionService implemented (RespondToPermission)
- [ ] AgentService implemented (ListAgents)
- [ ] AuthService implemented (GetAuthStatus, GetProviderAuth, GetProviderStatus, SwitchProviderAuth)
- [ ] EventService implemented (SubscribeGlobalEvents - SSE → gRPC stream)
- [ ] HTTP client for OpenCode server REST API
- [ ] SSE event parsing and gRPC streaming

**Tauri Integration:**

- [ ] gRPC server initialized on `localhost:50051`
- [ ] Graceful shutdown on app exit
- [ ] Production-grade logging throughout

**Blazor Client Services:**

- [ ] ISessionService + SessionService (gRPC client wrapper)
- [ ] IProviderService + ProviderService
- [ ] IMessageService + MessageService
- [ ] IAgentService + AgentService
- [ ] IAuthService + AuthService
- [ ] IEventService + EventService
- [ ] MarkdownService (Markdig wrapper)

**UI Components:**

- [ ] Sessions.razor - Session list sidebar
- [ ] TabBar.razor - Tab navigation
- [ ] ModelPicker.razor - Model/provider selection
- [ ] AgentPicker.razor - Agent selection
- [ ] DirectoryInput.razor - Working directory input
- [ ] Chat.razor - Main chat interface
- [ ] MessageList.razor - Message display
- [ ] MessageInput.razor - User input box
- [ ] MessageRenderer.razor - Markdown rendering
- [ ] Settings.razor - Settings page
- [ ] ProviderStatus.razor - Provider connection status
- [ ] AuthModeToggle.razor - OAuth vs API key toggle

**Functionality:**

- [ ] Tab/session CRUD operations working
- [ ] Working directory tracked per tab (x-opencode-directory header)
- [ ] AbortSession working (POST /session/{id}/abort)
- [ ] Agent selection per tab
- [ ] Model/provider selection per tab
- [ ] Message streaming via gRPC (SSE → gRPC translation)
- [ ] Markdown rendering (text, code, reasoning, tool calls)
- [ ] Token counts displayed
- [ ] Provider status displayed
- [ ] Auth mode switching functional
- [ ] Error handling and loading states

### Session 5

- [ ] OAuth login successful
- [ ] Settings persist across restarts
- [ ] Multiple providers supported

### Session 6

- [ ] Builds on all platforms
- [ ] Performance acceptable (<2s startup)
- [ ] Documentation published

---

## Notes & Decisions

### Session 4 (2026-01-04 to 2026-01-05) ✅

**Accomplishments:**

- ✅ Created 72+ JSON Schema files from OpenCode server TypeScript/Zod types
- ✅ Organized schemas by domain: Model (8), Provider (6), Auth (4), Session (10), Message (20), Tool (9), Agent (2), Event (13)
- ✅ Wrote comprehensive documentation (4,400+ lines across 9 proto docs)
- ✅ Created cross-reference tables for all types (JSON Schema ↔ Protobuf)
- ✅ Generated Zod validators from JSON schemas
- ✅ Refactored OpenCode server to use generated validators
- ✅ Validated all schemas with `bun run generate:schemas`
- ✅ All 544 OpenCode server tests pass after refactoring
- ✅ Build succeeds for 11 platforms

**Files Created:**

- `submodules/opencode/schema/*.schema.json` - 72+ JSON Schema files
- `docs/proto/01-model.md` through `docs/proto/09-opencode.md` - 9 documentation files
- `docs/proto/README.md` - Overview and version history
- `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` - Workflow documentation
- `docs/proto/SESSION_4_SCHEMA_PROMPT_ARCHIVE.md` - Session 4 schema creation prompt (archived)

**Technical Decisions:**

- **JSON Schema as source of truth** - Chose JSON Schema over protobuf-first to match OpenCode server exactly
- **Generated validators** - Used custom generator (`script/generate-from-schemas.ts`) to produce Zod validators
- **Discriminated unions** - Used `oneOf` with `type`/`name`/`role` discriminators throughout
- **Comprehensive error types** - Created NamedError pattern with 6 error variants
- **72+ schemas** - Far more than initially estimated due to thorough coverage of all OpenCode types

**Key Learnings:**

- JSON Schema → Zod → TypeScript is more reliable than manual protobuf translation
- OpenCode server has ~72 distinct types requiring schemas (not 40+)
- Message system is most complex domain (20 schemas: 11 parts, 6 errors, 3 message types)
- Cross-reference tables essential for protobuf → JSON Schema traceability

**Scope Change:**

- Original plan: Write `.proto` files directly
- Actual approach: Create JSON schemas first, document protobuf second
- Rationale: Single source of truth, automated validation, better alignment with OpenCode server

**Deferred to Session 4.5:**

- Actual `.proto` file creation (will copy from documentation)
- Code generation (Rust via tonic-build, C# via Grpc.Tools)
- gRPC service implementation
- UI components

**Next Steps:**

- Session 4.5 will create `.proto` files from completed documentation
- Much faster now that all types are documented and validated

---

### Session 1 (2026-01-02) ✅

**Accomplishments:**

- ✅ Created workspace at repository root with Cargo.toml
- ✅ Built `backend/client-core/` crate from scratch (NOT extracted from egui)
- ✅ Built `models/` crate for shared ErrorLocation utilities (renamed from `common/`)
- ✅ Implemented production-grade error handling:
  - `CoreError`, `DiscoveryError`, `SpawnError` with ErrorLocation tracking
  - All errors use `common` crate for location tracking
- ✅ Implemented discovery module:
  - `discover()` - finds running opencode server via `ps` + regex
  - `stop_pid()` - graceful shutdown with exponential backoff kill verification
  - `check_health()` - HTTP GET with exponential backoff retry
- ✅ Implemented spawn module:
  - `spawn_and_wait()` - spawns server and waits for health check
  - Exponential backoff for health checks
  - Process cleanup on failure paths
  - Stderr capture for debugging
- ✅ Zero magic numbers - all constants named
- ✅ DRY helpers throughout
- ✅ Regex compiled once with OnceLock
- ✅ Full rustdoc on public APIs
- ✅ Clippy clean with `-D warnings`

**Files Created:**

- `Cargo.toml` - workspace root
- `models/Cargo.toml` - shared utilities
- `models/src/lib.rs` - ErrorLocation trait
- `backend/client-core/Cargo.toml` - core logic
- `backend/client-core/src/lib.rs` - public API
- `backend/client-core/src/error/` - error types module
- `backend/client-core/src/discovery/mod.rs` - discovery module
- `backend/client-core/src/discovery/process.rs` - process logic
- `backend/client-core/src/spawn/mod.rs` - spawn module
- `README.md` - project structure docs

**Technical Decisions:**

- **Built fresh instead of extracting from egui** - Allows production-grade code without egui constraints (egui now in submodules/opencode-egui/)
- **Located code in `backend/client-core/`** - Repository root structure, not nested in clients/
- **Created `models/` crate** - Shared utilities between backend crates
- **ErrorLocation pattern** - Consistent error tracking across all error types
- **OnceLock for regex** - Compile once, reuse across all calls
- **Exponential backoff everywhere** - Robust retry logic for health checks and process cleanup
- **No magic numbers** - All timeouts, delays, retries are named constants

**Deferred to Session 2:**

- ~~Tauri scaffold (`apps/desktop/opencode/`)~~ ✅ Complete
- No changes to egui client (remains independent in submodules/opencode-egui/)

**Next Steps:**

- ~~Session 2 will scaffold Tauri backend and wire up commands to `client-core`~~ ✅ Complete

---

### Session 3 (2026-01-04) ✅

**Accomplishments:**

- ✅ Created Blazor WASM project at `frontend/desktop/opencode/`
- ✅ Configured .NET 10.0 with Radzen 8.4.2 + Markdig dependencies
- ✅ Set up Program.cs with Radzen service registration + IServerService DI
- ✅ Created `IServerService` interface with 4 async methods
- ✅ Implemented `ServerService` with IJSRuntime-based Tauri command invocation
- ✅ Created `TauriCommands` constants (DiscoverServer, SpawnServer, CheckHealth, StopServer)
- ✅ Created `TauriConstants` for eval method and invoke prefix configuration
- ✅ Implemented exception types:
  - `ServerOperationException` - base class
  - `ServerDiscoveryException` - discover_server failures
  - `ServerSpawnException` - spawn_server failures
  - `ServerHealthCheckException` - check_health failures
  - `ServerStopException` - stop_server failures
- ✅ Created Home.razor with complete server management UI:
  - Server status display (PID, host, port)
  - Discover Server button (auto-run on init)
  - Spawn Server button
  - Check Health button (disabled when no server)
  - Stop Server button (disabled when no server, danger styling)
  - Radzen notifications for success/error feedback
  - Loading states with IsBusy button prop
- ✅ Created Radzen component support in App.razor (Dialog, Notification, ContextMenu, Tooltip)
- ✅ Updated \_Imports.razor with global usings (System, Microsoft, Radzen, OpenCode namespaces)
- ✅ Configured JSON deserialization with snake_case naming policy (matches Rust ServerInfo)
- ✅ Published Blazor output to `apps/desktop/opencode/frontend/`
- ✅ Verified Tauri loads Blazor UI correctly
- ✅ Tested all 4 commands from UI - all working end-to-end
- ✅ Zero custom JavaScript (all Tauri IPC via C# IJSRuntime)

**Files Created:**

- `frontend/desktop/opencode/Opencode.csproj` - .NET 10 Blazor project
- `frontend/desktop/opencode/Program.cs` - DI configuration
- `frontend/desktop/opencode/_Imports.razor` - Global usings
- `frontend/desktop/opencode/App.razor` - Root component with Radzen services
- `frontend/desktop/opencode/Layout/MainLayout.razor` - Main layout
- `frontend/desktop/opencode/Layout/NavMenu.razor` - Navigation menu
- `frontend/desktop/opencode/Pages/Home.razor` - Server status page
- `frontend/desktop/opencode/Pages/NotFound.razor` - 404 page
- `frontend/desktop/opencode/Services/IServerService.cs` - Service interface
- `frontend/desktop/opencode/Services/ServerService.cs` - Service implementation
- `frontend/desktop/opencode/Services/TauriCommands.cs` - Command constants
- `frontend/desktop/opencode/Services/TauriConstants.cs` - Tauri constants
- `frontend/desktop/opencode/Services/Exceptions/ServerOperationException.cs` - Exception types
- `apps/desktop/opencode/frontend/` - Published Blazor output

**Technical Decisions:**

- **Separate frontend directory** - Located Blazor project at `frontend/desktop/opencode/` (not inside Rust app) for cleaner separation of concerns
- **IJSRuntime pattern** - Used `eval` method to invoke `window.__TAURI__.invoke()` (cleaner than alternatives)
- **JsonElement intermediate** - Deserialized Tauri responses via JsonElement before mapping to ServerInfo
- **Snake case naming** - Configured `JsonNamingPolicy.SnakeCaseLower` to match Rust API field names
- **Exception hierarchy** - Created custom exception types for each operation (improves error handling in UI)
- **Radzen components** - Used throughout (Cards, Stacks, Alerts, Buttons, Notifications) for consistency
- **ConfigureAwait(false)** - Added to all async calls for better performance in WASM context
- **Auto-discover on init** - Home.razor runs DiscoverServer in OnInitializedAsync for better UX

**Key Learnings:**

- Blazor IJSRuntime can invoke `eval` to access window properties (though not ideal long-term)
- Radzen's NotificationService provides toast notifications without custom DOM manipulation
- .NET 10 Blazor WASM with Radzen is very productive - full UI framework included
- JSON deserialization requires explicit naming policy to match Rust snake_case
- Exception wrapping improves Blazor UI error handling (can match on specific types)

**Deferred from Session 3:**

- gRPC client setup (will use in Session 4 for chat)
- Chat UI components (Session 4)
- Session/conversation management (Session 4)

**Next Steps:**

- Session 4 will implement chat UI and message streaming from Claude API

---

### Session 2 (2026-01-03) ✅

**Accomplishments:**

- ✅ Created Tauri 2.9.5 project at `apps/desktop/opencode/`
- ✅ Configured `tauri.conf.json` following Cognexus pattern (CSP null for Blazor)
- ✅ Created `build.rs` and `main.rs` with proper initialization
- ✅ Updated workspace `Cargo.toml` to include new member
- ✅ Implemented **production-grade state management**:
  - **Refactored from simple `Arc<Mutex<T>>` to actor pattern**
  - Eliminated all race conditions by design
  - State actor runs in dedicated task processing commands sequentially
  - Uses `Arc<RwLock<T>>` for lock-free reads
  - Created `StateCommand` enum for mutations
- ✅ Implemented error handling:
  - Created `OpencodeError` enum (Opencode, Core, NoServer, StopFailed)
  - All errors include `ErrorLocation` tracking
  - Removed `StateLock` error variant (impossible with actor pattern)
- ✅ Implemented 4 Tauri commands:
  - `discover_server()` - discovers running server, updates state via actor
  - `spawn_server()` - spawns new server, updates state via actor
  - `check_health()` - checks server health from state
  - `stop_server()` - stops server, clears state via actor
- ✅ Created production-grade logging infrastructure:
  - Dual output: colored stdout + plain text file
  - Thread-safe initialization with `Once` + `AtomicBool`
  - Different log levels for debug/release builds
  - Integrated into `main.rs` setup hook
- ✅ **Architecture refactoring:**
  - Renamed `common/` → `models/` for teaching clarity
  - Added module-level documentation explaining layered architecture
  - Moved `ServerInfo` to `models/server_info.rs`
  - All imports updated from `common::` to `models::`
- ✅ Testing:
  - Created test HTML frontend
  - Verified all commands work via browser console
  - Tested full workflow: discover/spawn → health check → stop
  - Confirmed state persistence and clean shutdown
- ✅ Clippy clean with `-D warnings`
- ✅ Full rustdoc on all public APIs

**Files Created:**

- `apps/desktop/opencode/Cargo.toml` - Tauri package configuration
- `apps/desktop/opencode/tauri.conf.json` - Tauri app configuration
- `apps/desktop/opencode/build.rs` - Build script
- `apps/desktop/opencode/.gitignore` - Tauri-specific ignores
- `apps/desktop/opencode/src/main.rs` - Entry point with logging setup
- `apps/desktop/opencode/src/state.rs` - Actor-based state management
- `apps/desktop/opencode/src/error.rs` - OpencodeError type
- `apps/desktop/opencode/src/logger.rs` - Production logging infrastructure
- `apps/desktop/opencode/src/commands/mod.rs` - Commands module
- `apps/desktop/opencode/src/commands/server.rs` - Server commands with logging
- `apps/desktop/opencode/frontend/wwwroot/index.html` - Test HTML
- `models/src/server_info.rs` - Shared ServerInfo model

**Files Modified:**

- `Cargo.toml` - Added `apps/desktop/opencode` member, renamed `common` → `models`
- `README.md` - Updated paths, documented `models/` crate
- `models/src/lib.rs` - Added teaching-focused documentation (formerly `common/`)
- `backend/client-core/Cargo.toml` - Updated dependency `common` → `models`
- `backend/client-core/src/lib.rs` - Updated imports `common::` → `models::`

**Technical Decisions:**

- **Actor pattern for state** - Eliminates race conditions by sequential command processing
- **`&Path` parameter** - More idiomatic than `PathBuf` or `&PathBuf` (clippy recommendation)
- **CSP null** - Required for Blazor WASM (uses eval)
- **Renamed `common/` → `models/`** - Better teaching clarity for layered architecture
- **Production logging from day 1** - Easier debugging, demonstrates best practices
- **Tech debt addressed immediately** - Refactored race conditions before committing

**Key Learnings:**

- Simple `Arc<Mutex<Option<T>>>` state has race conditions in concurrent environments
- Actor pattern is industry standard for eliminating state races by design
- Logging infrastructure pays for itself immediately in complex apps
- Clear module naming (`models/` vs `common/`) improves teaching effectiveness

**Next Steps:**

- Session 3 will initialize Blazor WASM project and create C# service layer

---

## Technical Constraints

1. **Zero Custom JavaScript** - All Tauri IPC via C# IJSRuntime only
2. **.NET 10.0+** - Target modern .NET for best Blazor support
3. **Tauri 2.9.5+** - Match Cognexus proven version
4. **Radzen Components** - Use for all UI (no custom DOM manipulation)
5. **Shared Rust Code** - Maximize code reuse between egui and tauri-blazor
6. **Tauri = Webview Host Only** - All application logic lives in client-core (not Tauri)

---

## Dependencies

- **Session 2** depends on Session 1 (needs shared crate)
- **Session 3** depends on Session 2 (needs Tauri commands)
- **Session 4** depends on Session 3 (needs Blazor scaffold)
- **Session 5** depends on Session 4 (needs chat working)
- **Session 6** depends on Sessions 1-5 (integration/polish)

---

## Risk Mitigation

**Risk:** Breaking egui client while extracting code

- **Mitigation:** egui client isolated in submodules/opencode-egui/, no cross-contamination

**Risk:** JSInterop for Tauri not working as expected

- **Mitigation:** Create minimal test case in Session 2 before building full UI

**Risk:** Blazor WASM bundle size too large

- **Mitigation:** Use trimming options, analyze bundle size early

**Risk:** Streaming performance issues

- **Mitigation:** Benchmark early, consider Tauri events vs SSE

---

## Open Questions

1. ~~Should we use .NET 9 or .NET 10?~~ → **Use .NET 10 (more stable, LTS-adjacent)**
2. ~~justfile vs Bun integration?~~ → **Start with justfile (proven in Cognexus), integrate with Bun later**
3. Audio/STT approach? → **Defer to future session (not in MVP)**
4. Mobile support? → **No, desktop only for now**

---

## Total Estimated Effort

**7 sessions total** (Adjusted after Session 4 scope change):

1. Session 1: ~60K (Shared Rust Core) ✅ Complete
2. Session 2: ~120K (Tauri Backend) ✅ Complete
3. Session 3: ~90K (Blazor Frontend Scaffold) ✅ Complete
4. Session 4: ~60K (Data Models Documentation) ✅ Complete (was 140K implementation, became 60K documentation)
5. Session 4.5: ~170K (Protobuf Creation + gRPC Implementation) ⏳ Next (was 150K, increased to include proto file creation)
6. Session 5: ~100K (Auth & Settings)
7. Session 6: ~80K (Polish & Testing)

**Total: ~680K tokens** (unchanged from original estimate, but work redistributed)

**Timeline Estimate:** 8-10 weeks (1 session per week, with buffer for discoveries)

**Scope Evolution:**

- **Original Session 4 plan:** Write `.proto` files + stub services (140K tokens)
- **Actual Session 4 work:** Create 72+ JSON schemas + 4,400 lines documentation (60K tokens) ✅
- **Updated Session 4.5 plan:** Create `.proto` from docs + full implementation (170K tokens) ⏳

**Why this is better:**

- **JSON Schema as canonical source** - Matches OpenCode server exactly (72+ types discovered, not 40+)
- **Automated validation** - Generator prevents drift between TypeScript ↔ JSON Schema ↔ Protobuf
- **Documentation-first** - All types documented before implementation reduces rework
- **Session 4.5 faster** - Copy from docs vs. discovery (well-defined types reduce implementation surprises)
- **Overall budget unchanged** - Session 4 used 80K fewer tokens, Session 4.5 gets 20K more (net same ~680K)

**Token Budget Reallocation:**

```
Original:  S4 (140K) + S4.5 (150K) = 290K
Actual:    S4 (60K)  + S4.5 (170K) = 230K (60K saved for future sessions)
```

---

**Last Updated:** 2026-01-05
