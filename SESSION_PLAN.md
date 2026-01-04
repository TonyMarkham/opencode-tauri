# Session Plan: Tauri + Blazor Desktop Client (ADR-0001)

## Goal

Build a new Tauri + Blazor WebAssembly desktop client as an alternative to egui, sharing Rust backend code and following the zero-custom-JavaScript policy.

---

## Session 1: Shared Rust Core & Project Scaffold

### Step 1: Extract Shared Client Core

- ✅ Create shared workspace at `clients/tauri-blazor/`
- ✅ Build production-grade `backend/client-core/` crate from scratch
- ✅ Build production-grade `common/` crate for shared utilities
- ⏭️ Extract from egui (DEFERRED - built fresh code instead)
- ⏭️ Update egui to use shared crate (DEFERRED - egui unchanged)

### Step 2: Create Tauri-Blazor Directory Structure

- ✅ Create `clients/tauri-blazor/` workspace layout
- ✅ Create workspace Cargo.toml with proper dependencies
- ✅ Create README.md documenting structure
- ⏭️ Set up Tauri project skeleton (`apps/desktop/opencode/`) (DEFERRED to Session 2)

**Status:** ✅ Complete

**Actual Tokens:** ~60K

**Deliverables:**

- ✅ Working `backend/client-core` with discovery + spawn logic
- ✅ Working `common` crate with ErrorLocation utilities
- ✅ egui client unchanged (no breaking changes)
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

- ✅ Create `frontend/` directory at `clients/tauri-blazor/frontend/desktop/opencode/`
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

**Goal:** Define comprehensive protobuf schema and gRPC service interface based on egui client audit. Group data models logically by domain: Sessions, Messages, Tools, Agents, Auth, and Events.

---

### Design Rationale

**Why This Session Matters:**

This session defines the **complete data contract** between Blazor UI (C#) and Rust backend (client-core). Getting this right is critical because:

1. Changes after implementation are expensive (protobuf breaking changes)
2. Poor data models lead to UI complexity (excessive lookups, stale data)
3. Missing fields discovered later require rework across the stack

**Key Design Decisions:**

1. **Composition Over Duplication**
   - `TabInfo` contains `SessionInfo` (not duplicate fields)
   - `ModelSelection` contains `ProviderInfo` (not just provider_id string)
   - **Why:** Single source of truth, no sync issues between duplicated fields

2. **Stable vs. Volatile Data Separation**
   - `ProviderInfo` (name, source) included in `ModelSelection` - stable, safe to cache
   - `ProviderAuthInfo` (type, expires) queried separately - volatile, must be fresh
   - **Why:** Auth can change (OAuth ↔ API key switch), provider metadata doesn't

3. **Per-Provider Auth (Not Global)**
   - Each provider has its own auth mode (`anthropic`: OAuth, `openai`: API key)
   - Auth tracked per provider_id, not globally
   - **Why:** Server's `auth.json` structure has per-provider auth entries

4. **UI-Centric Data Models**
   - `ModelSelection` includes `ProviderInfo` for clean UI display
   - `ProviderInfo` includes curated models list (not all discoverable models)
   - No extra lookups needed for common UI tasks (model picker, tab header)
   - **Why:** Frontend perspective - reduce boilerplate, improve code clarity

5. **Curated Models Over Discovery**
   - `ProviderInfo.models` contains only configured/curated models (from server)
   - Model discovery UI (add arbitrary models) deferred to Session 5
   - **Why:** Curated list is what user actually wants in model picker (not giant unfiltered list)
   - **Maps to:** OpenCode server `GET /config/providers` endpoint

6. **Comprehensive Tool State**
   - Tool execution includes logs, metadata, timing (learned from egui audit)
   - Supports rich UI (progress indicators, log viewers, timing charts)
   - **Why:** Users need visibility into what tools are doing

7. **Event Streaming Translation**
   - SSE events from OpenCode server translated to gRPC streams
   - Blazor consumes unified event stream (no SSE client needed)
   - **Why:** Consistent API surface, easier testing, better type safety

8. **Typed Model Options (Not Generic Maps)**
   - `ModelOptions` uses proper protobuf types with provider-specific submessages
   - Each provider (OpenAI, Google, Anthropic) has its own strongly-typed options message
   - Universal options (temperature, maxOutputTokens) in separate message
   - **Why:** Type safety in C#, IDE autocomplete, schema clarity, compile-time validation
   - **Trade-off:** More maintenance when providers add options vs. flexible `map<string, string>`
   - **Decision:** Type safety wins - desktop client should validate before sending to server

**Scope Boundaries:**

- ✅ **In Scope:** All data models, service definitions, protobuf schemas
- ✅ **In Scope:** Stubbed gRPC service methods (return "Not implemented")
- ❌ **Out of Scope:** OpenCode server HTTP calls (Session 4.5)
- ❌ **Out of Scope:** UI implementation (Session 4.5)
- ❌ **Out of Scope:** SSE event parsing (Session 4.5)

---

### Implementation Steps

**See [PROTO_SCHEMA.md](./PROTO_SCHEMA.md) for complete protobuf definitions (40+ messages, 6 services)**

**Session 4 Workflow:**

1. **Create Proto Files** - 7 files organized by domain (session, message, tool, agent, auth, event, service)
2. **Generate Code** - Run `protoc` to generate Rust (tonic/prost) and C# (Grpc.Tools) code
3. **Stub Services** - Implement all 6 gRPC services in `client-core`, all methods return "Not implemented"
4. **Wire Tauri** - Initialize gRPC server on `localhost:50051` in Tauri main.rs
5. **Test Connectivity** - Blazor → gRPC → Rust stub → "Not implemented" error (success!)
6. **Verify Logging** - Production-grade logging throughout

---

**Status:** ⏳ Pending

**Estimated Tokens:** ~160K (expanded significantly due to comprehensive data modeling)

**Deliverables:**

- ✅ 7 protobuf files with logical grouping (session, message, tool, agent, auth, event, service)
- ✅ All protobuf messages defined (40+ messages total)
- ✅ 6 gRPC services stubbed in client-core (SessionService, MessageService, PermissionService, AgentService, AuthService, EventService)
- ✅ Tauri hosts gRPC server on localhost:50051
- ✅ Basic connectivity test from Blazor passes
- ✅ Comprehensive logging throughout
- ✅ Zero OpenCode server calls (all stubs)

---

## Session 4.5: App State & gRPC Service Implementation

### Step 1: Implement gRPC Service Logic in client-core

**Chat Operations:**

- Call OpenCode server HTTP REST API
- Handle message streaming from OpenCode SSE
- Convert SSE chunks to gRPC streams

**Session/Tab Management:**

- Implement tab CRUD in memory (or persistent storage)
- Track active tab state
- Persist selected model/agent per tab
- Track and send working directory per tab (x-opencode-directory header)

**Agent Management:**

- Fetch agents from OpenCode server
- List available agents
- Handle agent selection per tab

**Provider/Auth Management:**

- Fetch provider status from OpenCode server
- Implement auth mode switching (subscription vs API key)
- Track OAuth expiry

### Step 2: Build Tab/Session Management UI

- SessionTabs.razor (list of open sessions/tabs)
- TabBar showing session title and state
- Create/delete tab buttons
- Select active tab
- Show current model/agent selection
- Directory input per tab (sent as x-opencode-directory header)
- Stop/Abort button for active streaming response

### Step 3: Build Chat UI Components

- Chat.razor with message display
- ChatService.cs gRPC client wrapper
- Message input box
- Connect to streaming responses
- Error handling and loading states

### Step 4: Integrate Auth Mode UI & Provider Status

- Auth mode toggle (subscription vs API key)
- Display OAuth expiry time
- Show connected providers
- Handle auth switching

**Status:** ⏳ Pending

**Estimated Tokens:** ~150K (increased due to scope)

**Deliverables:**

- ✅ Tab/session management working
- ✅ Agent selection per tab
- ✅ Auth mode switching functional
- ✅ Chat UI with message history
- ✅ Streaming responses working
- ✅ Markdown, reasoning, and tool calls rendered
- ✅ Provider status displayed

---

## Session 5: Authentication & Settings

### Step 1: OAuth Integration

- Extract auth logic from egui to shared core
- Create `commands/auth.rs` in Tauri
- Implement OAuth flow for Anthropic
- Add API key management

### Step 2: Settings Panel

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
- Document differences from egui client
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

### Session 4

**Data Modeling:**

- [ ] Session models complete (SessionInfo, TabInfo, ModelSelection, SessionTime)
- [ ] Provider models complete (ProviderInfo with curated models list)
- [ ] Model models complete (ModelInfo, ModelCapabilities, ModelCost, ModelLimits, IOCapabilities, CacheCost)
- [ ] ModelOptions complete (UniversalOptions, OpenAIOptions, GoogleOptions, AnthropicOptions with all discovered settings)
- [ ] Message models complete (SendMessageRequest, DisplayMessage, MessagePart, TextPart, FilePart)
- [ ] Tool models complete (ToolCallState, PermissionRequest, PermissionResponse)
- [ ] Agent models complete (AgentInfo, AgentList)
- [ ] Auth models complete (ProviderAuthInfo, AuthStatus, AuthType, ProviderStatus)
- [ ] Event models complete (GlobalEvent, MessageUpdated, MessagePartUpdated, PermissionCreated, TokenUsage)

**Service Definitions:**

- [ ] SessionService defined (ListSessions, CreateSession, DeleteSession, UpdateSessionDirectory)
- [ ] ProviderService defined (GetProviders - returns providers with curated models)
- [ ] MessageService defined (SendMessage, GetMessages, AbortSession)
- [ ] PermissionService defined (RespondToPermission)
- [ ] AgentService defined (ListAgents)
- [ ] AuthService defined (GetAuthStatus, GetProviderAuth, GetProviderStatus, SwitchProviderAuth)
- [ ] EventService defined (SubscribeGlobalEvents)

**Implementation:**

- [ ] 7 protobuf files created with logical grouping
- [ ] All 7 services stubbed in client-core (return "Not implemented" errors)
- [ ] ProviderInfo includes curated models list (from GET /config/providers structure)
- [ ] ModelInfo includes capabilities, cost, limits (rich metadata for UI)
- [ ] Tauri initializes gRPC server on localhost:50051
- [ ] Graceful shutdown on app exit
- [ ] Basic connectivity test from Blazor passes (Blazor → gRPC → Rust)
- [ ] Production-grade logging throughout
- [ ] Zero OpenCode server HTTP calls (all stubs)

### Session 4.5

- [ ] Tab/session CRUD operations implemented in client-core
- [ ] Working directory tracked per tab and sent as x-opencode-directory header
- [ ] AbortSession implemented (POST /session/{id}/abort)
- [ ] Agent management working (list, select per tab)
- [ ] Provider status fetched from OpenCode server
- [ ] Auth mode switching implemented (subscription vs API key)
- [ ] Version tracking implemented
- [ ] OpenCode server communication working (HTTP REST)
- [ ] Message streaming via gRPC functional end-to-end
- [ ] SessionTabs.razor displays open tabs and allows creation/deletion
- [ ] Directory input per tab functional
- [ ] Stop/Abort button stops active streaming responses
- [ ] Chat.razor displays messages and accepts input
- [ ] Markdown renders correctly (text, reasoning, tool calls)
- [ ] Token counts displayed
- [ ] Auth UI shows current mode and expiry
- [ ] Error handling and loading states working

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

### Session 1 (2026-01-02) ✅

**Accomplishments:**

- ✅ Created workspace at `clients/tauri-blazor/` with Cargo.toml
- ✅ Built `backend/client-core/` crate from scratch (NOT extracted from egui)
- ✅ Built `common/` crate for shared ErrorLocation utilities
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

- `clients/tauri-blazor/Cargo.toml` - workspace root
- `clients/tauri-blazor/common/Cargo.toml` - shared utilities
- `clients/tauri-blazor/common/src/lib.rs` - ErrorLocation trait
- `clients/tauri-blazor/backend/client-core/Cargo.toml` - core logic
- `clients/tauri-blazor/backend/client-core/src/lib.rs` - public API
- `clients/tauri-blazor/backend/client-core/src/error.rs` - error types
- `clients/tauri-blazor/backend/client-core/src/discovery/mod.rs` - discovery module
- `clients/tauri-blazor/backend/client-core/src/discovery/process.rs` - process logic
- `clients/tauri-blazor/backend/client-core/src/spawn/mod.rs` - spawn module
- `clients/tauri-blazor/README.md` - project structure docs

**Technical Decisions:**

- **Built fresh instead of extracting from egui** - Allows production-grade code without egui constraints
- **Located code in `clients/tauri-blazor/backend/client-core/`** - Not `crates/` since it's tauri-specific for now
- **Created `common/` crate** - Shared utilities between backend crates
- **ErrorLocation pattern** - Consistent error tracking across all error types
- **OnceLock for regex** - Compile once, reuse across all calls
- **Exponential backoff everywhere** - Robust retry logic for health checks and process cleanup
- **No magic numbers** - All timeouts, delays, retries are named constants

**Deferred to Session 2:**

- ~~Tauri scaffold (`apps/desktop/opencode/`)~~ ✅ Complete
- No changes to egui client (remains independent)

**Next Steps:**

- ~~Session 2 will scaffold Tauri backend and wire up commands to `client-core`~~ ✅ Complete

---

### Session 3 (2026-01-04) ✅

**Accomplishments:**

- ✅ Created Blazor WASM project at `clients/tauri-blazor/frontend/desktop/opencode/`
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

- `clients/tauri-blazor/frontend/desktop/opencode/Opencode.csproj` - .NET 10 Blazor project
- `clients/tauri-blazor/frontend/desktop/opencode/Program.cs` - DI configuration
- `clients/tauri-blazor/frontend/desktop/opencode/_Imports.razor` - Global usings
- `clients/tauri-blazor/frontend/desktop/opencode/App.razor` - Root component with Radzen services
- `clients/tauri-blazor/frontend/desktop/opencode/Layout/MainLayout.razor` - Main layout
- `clients/tauri-blazor/frontend/desktop/opencode/Layout/NavMenu.razor` - Navigation menu
- `clients/tauri-blazor/frontend/desktop/opencode/Pages/Home.razor` - Server status page
- `clients/tauri-blazor/frontend/desktop/opencode/Pages/NotFound.razor` - 404 page
- `clients/tauri-blazor/frontend/desktop/opencode/Services/IServerService.cs` - Service interface
- `clients/tauri-blazor/frontend/desktop/opencode/Services/ServerService.cs` - Service implementation
- `clients/tauri-blazor/frontend/desktop/opencode/Services/TauriCommands.cs` - Command constants
- `clients/tauri-blazor/frontend/desktop/opencode/Services/TauriConstants.cs` - Tauri constants
- `clients/tauri-blazor/frontend/desktop/opencode/Services/Exceptions/ServerOperationException.cs` - Exception types
- `clients/tauri-blazor/apps/desktop/opencode/frontend/` - Published Blazor output

**Technical Decisions:**

- **Separate frontend directory** - Located Blazor project at `clients/tauri-blazor/frontend/desktop/opencode/` (not inside Rust app) for cleaner separation of concerns
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

- `clients/tauri-blazor/Cargo.toml` - Added `apps/desktop/opencode` member, renamed `common` → `models`
- `clients/tauri-blazor/README.md` - Updated paths, documented `models/` crate
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

- **Mitigation:** Test egui after each extraction, make changes incrementally

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

**7 sessions total** (Scoped comprehensively):

1. Session 1: ~60K (Shared Rust Core)
2. Session 2: ~120K (Tauri Backend)
3. Session 3: ~90K (Blazor Frontend Scaffold)
4. Session 4: ~140K (Data Models & gRPC Service Infrastructure) ← EXPANDED
5. Session 4.5: ~150K (App State & gRPC Service Implementation) ← EXPANDED
6. Session 5: ~100K (Auth & Settings)
7. Session 6: ~80K (Polish & Testing)

**Total: ~740K tokens** (increased from 680K as scope clarified)

**Timeline Estimate:** 8-10 weeks (1 session per week, with buffer for discoveries)

**Note:** Sessions 4 & 4.5 expanded significantly after review of egui client revealed:

- Tab/session management complexity
- Agent selection per tab
- Provider status and auth mode switching
- Version tracking requirements

---

**Last Updated:** 2026-01-04
