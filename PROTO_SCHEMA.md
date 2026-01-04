# Protobuf Schema Documentation

**Version:** 1.0.0  
**Last Updated:** 2026-01-04  
**Related:** [SESSION_PLAN.md](./SESSION_PLAN.md) - Session 4

---

## Overview

This document defines the **complete protobuf schema** for the Tauri-Blazor desktop client's gRPC communication layer. The schema is organized into **8 proto files** defining the data contract between:

- **Blazor Frontend (C#)** ↔ **Rust Backend (client-core)** ↔ **OpenCode Server (HTTP/SSE)**

### Design Principles

1. **Type Safety Over Flexibility** - Use proper protobuf types (enums, messages) instead of generic maps
2. **Composition Over Duplication** - `TabInfo` contains `SessionInfo`, `ModelSelection` contains `ProviderInfo`
3. **Separation of Concerns** - Stable data (provider metadata) vs volatile data (auth status)
4. **Per-Provider Auth** - Each provider has its own auth mode, not global
5. **UI-Centric Models** - Reduce lookups, cache intelligently, support rich UI features

### File Organization

```
clients/tauri-blazor/proto/
├── session.proto       - Sessions, tabs, provider info
├── model.proto         - Model metadata, capabilities, options, cost, limits
├── message.proto       - User/assistant messages, attachments
├── tool.proto          - Tool execution state, permissions
├── agent.proto         - Agent listing and metadata
├── auth.proto          - Authentication, provider status
├── event.proto         - SSE event streaming (gRPC translation)
└── opencode.proto      - Main service definitions
```

---

## 1. Model Metadata (`model.proto`)

### Purpose

Define comprehensive model metadata including capabilities, pricing, limits, and configurable options. This is the foundation schema - all other protos reference these model types.

### Source of Truth

**OpenCode Server Schema (TonyMarkham/opencode fork):**

- `submodules/opencode/packages/opencode/src/provider/models.ts` @ `81fef60` (2025-12-30)
  - ModelsDev.Model (line 12-51)
- `submodules/opencode/packages/opencode/src/provider/provider.ts` @ `d72d7ab` (2026-01-04)
  - Provider.Model (line 315-374)
- `submodules/opencode/packages/opencode/src/provider/transform.ts` @ `2fd9737` (2026-01-02)
  - Model options logic (line 172-273)
- `submodules/opencode/packages/opencode/src/provider/sdk/openai-compatible/src/responses/openai-responses-language-model.ts` @ `6e2379a` (2025-11-29)
  - OpenAI options (line 1671-1711)

### Messages

```protobuf
syntax = "proto3";
package opencode.model;

// Model metadata (matches OpenCode server Provider.Model schema)
// Source: submodules/opencode/packages/opencode/src/provider/provider.ts @ d72d7ab (2026-01-04) line 315-374
message ModelInfo {
  string id = 1;                           // "claude-3-5-sonnet-20241022"
  string provider_id = 2;                  // "anthropic"
  string name = 3;                         // "Claude 3.5 Sonnet"
  ModelAPI api = 4;                        // API configuration
  ModelCapabilities capabilities = 5;      // What the model supports (bools)
  ModelCost cost = 6;                      // Pricing information
  ModelLimits limits = 7;                  // Context/output limits
  ModelStatus status = 8;                  // "alpha" | "beta" | "deprecated" | "active"
  ModelOptions options = 9;                // Configurable settings (actual values)
  map<string, string> headers = 10;        // Custom headers for API calls
}

// API configuration for model
// Source: submodules/opencode/packages/opencode/src/provider/provider.ts @ d72d7ab (2026-01-04) line 315-374
message ModelAPI {
  string id = 1;     // API model ID (may differ from display ID)
  string url = 2;    // API endpoint URL
  string npm = 3;    // npm package for SDK
}

// Model capabilities (what the model can do - boolean flags)
// Source: submodules/opencode/packages/opencode/src/provider/provider.ts @ d72d7ab (2026-01-04) line 325-344
message ModelCapabilities {
  bool temperature = 1;    // Supports temperature parameter
  bool reasoning = 2;      // Supports extended reasoning
  bool attachment = 3;     // Supports file attachments
  bool toolcall = 4;       // Supports tool calls
  IOCapabilities input = 5;
  IOCapabilities output = 6;
}

// Input/output modality capabilities
// Source: submodules/opencode/packages/opencode/src/provider/models.ts @ 81fef60 (2025-12-30) line 40-44
message IOCapabilities {
  bool text = 1;
  bool audio = 2;
  bool image = 3;
  bool video = 4;
  bool pdf = 5;
}

// Model pricing (per token)
// Source: submodules/opencode/packages/opencode/src/provider/provider.ts @ d72d7ab (2026-01-04) line 345-361
message ModelCost {
  double input = 1;                               // Input token cost
  double output = 2;                              // Output token cost
  CacheCost cache = 3;                            // Cache costs (required)
  optional ExperimentalPricing over_200k = 4;     // Experimental pricing tier
}

message CacheCost {
  double read = 1;
  double write = 2;
}

message ExperimentalPricing {
  double input = 1;
  double output = 2;
  CacheCost cache = 3;
}

// Model context limits
// Source: submodules/opencode/packages/opencode/src/provider/models.ts @ 81fef60 (2025-12-30) line 36-39
message ModelLimits {
  double context = 1;  // Context window size (use double to match server z.number())
  double output = 2;   // Max output tokens
}

// Model status
// Source: submodules/opencode/packages/opencode/src/provider/models.ts @ 81fef60 (2025-12-30) line 47
enum ModelStatus {
  MODEL_STATUS_UNSPECIFIED = 0;
  ALPHA = 1;
  BETA = 2;
  ACTIVE = 3;
  DEPRECATED = 4;
}

// Model configurable options (actual runtime settings)
// Source: submodules/opencode/packages/opencode/src/provider/models.ts (line 48) - z.record(z.string(), z.any())
// BUT: We use typed submessages for compile-time safety instead of generic map
message ModelOptions {
  // Universal options (all providers/models)
  UniversalOptions universal = 1;

  // Provider-specific options
  oneof provider_options {
    OpenAIOptions openai = 2;
    GoogleOptions google = 3;
    AnthropicOptions anthropic = 4;
  }
}

// Universal settings supported by all models (when capabilities allow)
// Source: submodules/opencode/packages/opencode/src/provider/transform.ts @ 2fd9737 (2026-01-02) line 180-182, 250-252
message UniversalOptions {
  optional double temperature = 1;        // 0.0-2.0, when model.capabilities.temperature == true
  optional int32 max_output_tokens = 2;   // Override model limit
}

// OpenAI/OpenRouter/Responses API options
// Source: submodules/opencode/packages/opencode/src/provider/sdk/openai-compatible/src/responses/openai-responses-language-model.ts (line 1671-1711)
message OpenAIOptions {
  optional string reasoning_effort = 1;      // "low", "medium", "minimal" (reasoning models only)
  optional string reasoning_summary = 2;     // "auto" or custom summary
  optional TextVerbosity text_verbosity = 3; // Response verbosity
  optional ServiceTier service_tier = 4;     // Processing priority
  optional LogprobsConfig logprobs = 5;      // Token probability logging
  optional int32 max_tool_calls = 6;         // Max built-in tool calls
  optional bool parallel_tool_calls = 7;     // Allow parallel tool execution
  optional bool store = 8;                   // Store conversation for training
  optional bool strict_json_schema = 9;      // Enforce strict JSON schema validation
  optional string prompt_cache_key = 10;     // Prompt cache key for optimization
  repeated string include = 11;              // Additional response fields to include
  optional string metadata = 12;             // Custom metadata JSON string
  optional string user = 13;                 // End-user identifier
  optional string instructions = 14;         // System instructions
  optional string safety_identifier = 15;    // Safety classification ID
  optional string previous_response_id = 16; // Continue from previous response
}

// Source: submodules/opencode/packages/opencode/src/provider/sdk/openai-compatible/src/responses/openai-responses-language-model.ts (line 1709)
enum TextVerbosity {
  TEXT_VERBOSITY_UNSPECIFIED = 0;
  TEXT_VERBOSITY_LOW = 1;
  TEXT_VERBOSITY_MEDIUM = 2;
  TEXT_VERBOSITY_HIGH = 3;
}

// Source: submodules/opencode/packages/opencode/src/provider/sdk/openai-compatible/src/responses/openai-responses-language-model.ts (line 1706)
enum ServiceTier {
  SERVICE_TIER_UNSPECIFIED = 0;
  SERVICE_TIER_AUTO = 1;
  SERVICE_TIER_FLEX = 2;
  SERVICE_TIER_PRIORITY = 3;
}

// Source: submodules/opencode/packages/opencode/src/provider/sdk/openai-compatible/src/responses/openai-responses-language-model.ts (line 1690)
message LogprobsConfig {
  oneof config {
    bool enabled = 1;              // true = return logprobs
    int32 top_logprobs = 2;        // 1-20, number of top tokens to return
  }
}

// Google/Gemini options
// Source: submodules/opencode/packages/opencode/src/provider/transform.ts @ 2fd9737 (2026-01-02) line 199-205, 264-270
message GoogleOptions {
  optional ThinkingConfig thinking_config = 1;
}

message ThinkingConfig {
  optional bool include_thoughts = 1;  // Include thinking process in response
  optional int32 thinking_budget = 2;  // Max tokens for thinking (0 = minimal)
}

// Anthropic options (Claude extended thinking)
// Source: submodules/opencode/packages/opencode/src/provider/transform.ts @ 2fd9737 (2026-01-02) line 323-334
message AnthropicOptions {
  optional ThinkingOptions thinking = 1;
}

message ThinkingOptions {
  optional string type = 1;        // "enabled" or other modes
  optional int32 budget_tokens = 2; // Max tokens for extended thinking
}

// Model selection for a session (references model within provider)
message ModelSelection {
  ProviderInfo provider = 1;  // Provider with curated models list (from session.proto)
  string model_id = 2;        // "claude-3-5-sonnet-20241022" (must exist in provider.models)
  string name = 3;            // "Claude 3.5 Sonnet" (for display)
}
```

### Design Notes

**Why a separate file?**

- Models have 15+ messages (capabilities, cost, limits, options, etc.)
- Provider-specific options are complex (OpenAI has 16 fields, Google/Anthropic have nested configs)
- Keeps `session.proto` focused on session/tab management
- Allows model schema to evolve independently

**Key decisions:**

- `ModelOptions` uses typed submessages, not `map<string, string>` (type safety over flexibility)
- `ModelCapabilities` are booleans (what model supports), `ModelOptions` are actual values (user configuration)
- `ModelCost` uses `double` for per-token pricing (matches server schema)
- `ModelLimits` uses `double` to match server `z.number()` type

---

## 2. Session Management (`session.proto`)

### Purpose

Track conversation sessions/tabs with provider selection and working directory.

### Messages

```protobuf
syntax = "proto3";
package opencode.session;

import "model.proto";

// Core session identity and metadata (server-managed)
message SessionInfo {
  string id = 1;
  string title = 2;
  string directory = 3;           // Working directory
  optional string version = 4;    // Session version for optimistic updates
  optional SessionTime time = 5;
}

message SessionTime {
  int64 created = 1;   // Unix timestamp (ms)
  int64 updated = 2;
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

// Provider metadata (stable information for UI display)
message ProviderInfo {
  string id = 1;                     // "anthropic"
  string name = 2;                   // "Anthropic" (display name)
  string source = 3;                 // "api" | "config" | "custom" | "env"
  repeated ModelInfo models = 4;     // Curated models for this provider (from model.proto)
}

// Request/response messages
message SessionList {
  repeated SessionInfo sessions = 1;
}

message ProviderList {
  repeated ProviderInfo providers = 1;
  map<string, string> defaults = 2;  // Default provider/model per capability
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

### Service Definition

```protobuf
service SessionService {
  rpc ListSessions(Empty) returns (SessionList);
  rpc CreateSession(CreateSessionRequest) returns (SessionInfo);
  rpc DeleteSession(DeleteSessionRequest) returns (Empty);
  rpc UpdateSessionDirectory(UpdateDirectoryRequest) returns (Empty);
}

service ProviderService {
  rpc GetProviders(Empty) returns (ProviderList);  // Get all providers with curated models
}
```

### Maps to OpenCode Server

- `SessionService.ListSessions` → `GET /session`
- `SessionService.CreateSession` → `POST /session`
- `SessionService.DeleteSession` → `DELETE /session/{id}`
- `SessionService.UpdateSessionDirectory` → Updates internal state (sent as `x-opencode-directory` header)
- `ProviderService.GetProviders` → `GET /config/providers`

---

## 2. Model Metadata (`model.proto`)

### Purpose

Define comprehensive model metadata including capabilities, pricing, limits, and configurable options. This is a separate file because models have complex, provider-specific configurations.

### Messages

```protobuf
syntax = "proto3";
package opencode.model;

// Model metadata (matches OpenCode server Provider.Model schema)
message ModelInfo {
  string id = 1;                           // "claude-3-5-sonnet-20241022"
  string provider_id = 2;                  // "anthropic"
  string name = 3;                         // "Claude 3.5 Sonnet"
  ModelAPI api = 4;                        // API configuration
  ModelCapabilities capabilities = 5;      // What the model supports (bools)
  ModelCost cost = 6;                      // Pricing information
  ModelLimits limits = 7;                  // Context/output limits
  ModelStatus status = 8;                  // "alpha" | "beta" | "deprecated" | "active"
  ModelOptions options = 9;                // Configurable settings (actual values)
  map<string, string> headers = 10;        // Custom headers for API calls
}

// API configuration for model
message ModelAPI {
  string id = 1;     // API model ID (may differ from display ID)
  string url = 2;    // API endpoint URL
  string npm = 3;    // npm package for SDK
}

// Model capabilities (what the model can do - boolean flags)
message ModelCapabilities {
  bool temperature = 1;    // Supports temperature parameter
  bool reasoning = 2;      // Supports extended reasoning
  bool attachment = 3;     // Supports file attachments
  bool toolcall = 4;       // Supports tool calls
  IOCapabilities input = 5;
  IOCapabilities output = 6;
}

// Input/output modality capabilities
message IOCapabilities {
  bool text = 1;
  bool audio = 2;
  bool image = 3;
  bool video = 4;
  bool pdf = 5;
}

// Model pricing (per token)
message ModelCost {
  double input = 1;                               // Input token cost
  double output = 2;                              // Output token cost
  CacheCost cache = 3;                            // Cache costs (required)
  optional ExperimentalPricing over_200k = 4;     // Experimental pricing tier
}

message CacheCost {
  double read = 1;
  double write = 2;
}

message ExperimentalPricing {
  double input = 1;
  double output = 2;
  CacheCost cache = 3;
}

// Model context limits
message ModelLimits {
  double context = 1;  // Context window size (use double to match server z.number())
  double output = 2;   // Max output tokens
}

// Model status
enum ModelStatus {
  MODEL_STATUS_UNSPECIFIED = 0;
  ALPHA = 1;
  BETA = 2;
  ACTIVE = 3;
  DEPRECATED = 4;
}

// Model configurable options (actual runtime settings)
message ModelOptions {
  // Universal options (all providers/models)
  UniversalOptions universal = 1;

  // Provider-specific options
  oneof provider_options {
    OpenAIOptions openai = 2;
    GoogleOptions google = 3;
    AnthropicOptions anthropic = 4;
  }
}

// Universal settings supported by all models (when capabilities allow)
message UniversalOptions {
  optional double temperature = 1;        // 0.0-2.0, when model.capabilities.temperature == true
  optional int32 max_output_tokens = 2;   // Override model limit
}

// OpenAI/OpenRouter/Responses API options
message OpenAIOptions {
  optional string reasoning_effort = 1;      // "low", "medium", "minimal" (reasoning models only)
  optional string reasoning_summary = 2;     // "auto" or custom summary
  optional TextVerbosity text_verbosity = 3; // Response verbosity
  optional ServiceTier service_tier = 4;     // Processing priority
  optional LogprobsConfig logprobs = 5;      // Token probability logging
  optional int32 max_tool_calls = 6;         // Max built-in tool calls
  optional bool parallel_tool_calls = 7;     // Allow parallel tool execution
  optional bool store = 8;                   // Store conversation for training
  optional bool strict_json_schema = 9;      // Enforce strict JSON schema validation
  optional string prompt_cache_key = 10;     // Prompt cache key for optimization
  repeated string include = 11;              // Additional response fields to include
  optional string metadata = 12;             // Custom metadata JSON string
  optional string user = 13;                 // End-user identifier
  optional string instructions = 14;         // System instructions
  optional string safety_identifier = 15;    // Safety classification ID
  optional string previous_response_id = 16; // Continue from previous response
}

enum TextVerbosity {
  TEXT_VERBOSITY_UNSPECIFIED = 0;
  TEXT_VERBOSITY_LOW = 1;
  TEXT_VERBOSITY_MEDIUM = 2;
  TEXT_VERBOSITY_HIGH = 3;
}

enum ServiceTier {
  SERVICE_TIER_UNSPECIFIED = 0;
  SERVICE_TIER_AUTO = 1;
  SERVICE_TIER_FLEX = 2;
  SERVICE_TIER_PRIORITY = 3;
}

message LogprobsConfig {
  oneof config {
    bool enabled = 1;              // true = return logprobs
    int32 top_logprobs = 2;        // 1-20, number of top tokens to return
  }
}

// Google/Gemini options
message GoogleOptions {
  optional ThinkingConfig thinking_config = 1;
}

message ThinkingConfig {
  optional bool include_thoughts = 1;  // Include thinking process in response
  optional int32 thinking_budget = 2;  // Max tokens for thinking (0 = minimal)
}

// Anthropic options (Claude extended thinking)
message AnthropicOptions {
  optional ThinkingOptions thinking = 1;
}

message ThinkingOptions {
  optional string type = 1;        // "enabled" or other modes
  optional int32 budget_tokens = 2; // Max tokens for extended thinking
}

// Model selection for a session (references model within provider)
message ModelSelection {
  ProviderInfo provider = 1;  // Provider with curated models list (from session.proto)
  string model_id = 2;        // "claude-3-5-sonnet-20241022" (must exist in provider.models)
  string name = 3;            // "Claude 3.5 Sonnet" (for display)
}
```

### Design Notes

**Why a separate file?**

- Models have 15+ messages (capabilities, cost, limits, options, etc.)
- Provider-specific options are complex (OpenAI has 16 fields, Google/Anthropic have nested configs)
- Keeps `session.proto` focused on session/tab management
- Allows model schema to evolve independently

**Key decisions:**

- `ModelOptions` uses typed submessages, not `map<string, string>` (type safety over flexibility)
- `ModelCapabilities` are booleans (what model supports), `ModelOptions` are actual values (user configuration)
- `ModelCost` uses `double` for per-token pricing (matches server schema)
- `ModelLimits` uses `double` to match server `z.number()` type

---

## 3. Message & Content (`message.proto`)

### Purpose

Handle user/assistant messages with text, reasoning, file attachments, and token tracking.

### Messages

```protobuf
syntax = "proto3";
package opencode.message;

import "session.proto";

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

### Service Definition

```protobuf
service MessageService {
  rpc SendMessage(SendMessageRequest) returns (Empty);
  rpc GetMessages(GetMessagesRequest) returns (MessageList);
  rpc AbortSession(AbortSessionRequest) returns (Empty);
}
```

### Maps to OpenCode Server

- `MessageService.SendMessage` → `POST /session/{id}/message`
- `MessageService.GetMessages` → `GET /session/{id}/message`
- `MessageService.AbortSession` → `POST /session/{id}/abort`

---

## 4. Tool Execution (`tool.proto`)

### Purpose

Track tool execution with comprehensive state (logs, metadata, timing, permissions).

### Messages

```protobuf
syntax = "proto3";
package opencode.tool;

// Tool execution state (comprehensive from egui audit)
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

### Service Definition

```protobuf
service PermissionService {
  rpc RespondToPermission(PermissionResponse) returns (Empty);
}
```

### Maps to OpenCode Server

- `PermissionService.RespondToPermission` → `POST /session/{id}/permissions/{id}`

---

## 5. Agent Management (`agent.proto`)

### Purpose

List available agents with metadata (name, description, mode, color).

### Messages

```protobuf
syntax = "proto3";
package opencode.agent;

message AgentInfo {
  string name = 1;
  optional string description = 2;
  optional string mode = 3;         // "subagent" or null
  bool built_in = 4;
  optional string color = 5;        // Hex color for UI
}

message AgentList {
  repeated AgentInfo agents = 1;
}

message Empty {}
```

### Service Definition

```protobuf
service AgentService {
  rpc ListAgents(Empty) returns (AgentList);
}
```

### Maps to OpenCode Server

- `AgentService.ListAgents` → `GET /agent`

---

## 6. Authentication (`auth.proto`)

### Purpose

Track OAuth vs API key mode per provider, provider connections, OAuth expiry.

### Messages

```protobuf
syntax = "proto3";
package opencode.auth;

// Per-provider auth info (auth is provider-specific, not global!)
message ProviderAuthInfo {
  string provider_id = 1;
  AuthType type = 2;                // API_KEY or OAUTH
  optional uint64 expires_at = 3;   // OAuth expiry (Unix timestamp ms)
}

enum AuthType {
  AUTH_TYPE_UNSPECIFIED = 0;
  API_KEY = 1;
  OAUTH = 2;
}

// All provider auth status
message AuthStatus {
  repeated ProviderAuthInfo providers = 1;
}

// Provider status (which providers have auth configured)
message ProviderStatus {
  repeated string connected = 1;    // Provider IDs (e.g., ["anthropic", "openai"])
}

// Query auth for single provider
message ProviderAuthRequest {
  string provider_id = 1;
}

// Switch auth mode for provider
message SwitchProviderAuthRequest {
  string provider_id = 1;
  AuthType type = 2;
}

message Empty {}
```

### Service Definition

```protobuf
service AuthService {
  rpc GetAuthStatus(Empty) returns (AuthStatus);                        // All providers
  rpc GetProviderAuth(ProviderAuthRequest) returns (ProviderAuthInfo);  // Single provider
  rpc GetProviderStatus(Empty) returns (ProviderStatus);
  rpc SwitchProviderAuth(SwitchProviderAuthRequest) returns (Empty);
}
```

### Maps to OpenCode Server

- `AuthService.GetAuthStatus` → Reads `~/.local/share/opencode/auth.json`
- `AuthService.GetProviderAuth` → Reads `~/.local/share/opencode/auth.json` (single provider)
- `AuthService.GetProviderStatus` → `GET /provider`
- `AuthService.SwitchProviderAuth` → Internal state switch (triggers re-initialization)

---

## 7. Event Streaming (`event.proto`)

### Purpose

Translate OpenCode server SSE events into gRPC streams for real-time updates.

### Messages

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

### Service Definition

```protobuf
service EventService {
  rpc SubscribeGlobalEvents(Empty) returns (stream GlobalEvent);
}
```

### Maps to OpenCode Server

- `EventService.SubscribeGlobalEvents` → `GET /global/event` (SSE stream)
  - Backend subscribes to SSE, translates events to gRPC stream
  - Events: `message.updated`, `message.part.updated`, `permission.created`

---

## 8. Main Service Definition (`opencode.proto`)

### Complete Service Interface

```protobuf
syntax = "proto3";
package opencode;

import "session.proto";
import "message.proto";
import "tool.proto";
import "agent.proto";
import "auth.proto";
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

## Implementation Notes

### Rust (client-core)

1. **Code Generation:**

   ```toml
   [dependencies]
   tonic = "0.10"
   prost = "0.12"

   [build-dependencies]
   tonic-build = "0.10"
   ```

2. **Build Script (`build.rs`):**

   ```rust
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

3. **Service Implementation:**
   - All services return `Result<Response<T>, Status>`
   - Use `Status::unimplemented()` for stubbed methods
   - Add tracing/logging to all methods

### C# (Blazor)

1. **Code Generation:**

   ```xml
   <ItemGroup>
     <PackageReference Include="Grpc.Net.Client" Version="2.60.0" />
     <PackageReference Include="Google.Protobuf" Version="3.25.1" />
     <PackageReference Include="Grpc.Tools" Version="2.60.0" PrivateAssets="All" />
   </ItemGroup>

   <ItemGroup>
     <Protobuf Include="../../proto/**/*.proto" GrpcServices="Client" />
   </ItemGroup>
   ```

2. **Client Usage:**

   ```csharp
   var channel = GrpcChannel.ForAddress("http://localhost:50051");
   var client = new OpenCodeService.OpenCodeServiceClient(channel);

   var sessions = await client.ListSessionsAsync(new Empty());
   ```

---

## Version History

### 1.0.0 (2026-01-04)

- Initial schema definition
- 7 proto files with logical domain grouping
- Comprehensive ModelOptions with provider-specific submessages
- Full service interface (40+ messages, 6 services, 15+ RPC methods)
