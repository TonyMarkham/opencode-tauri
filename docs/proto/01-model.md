# Model Metadata (`model.proto`)

**Status:** ✅ Complete  
**Last Updated:** 2026-01-04

---

## Purpose

Define comprehensive model metadata including capabilities, pricing, limits, and configurable options. This is the foundation schema - all other protos reference these model types.

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/modelInfo.schema.json` - Core model metadata
- `submodules/opencode/schema/modelCapabilities.schema.json` - Capability flags
- `submodules/opencode/schema/modelCost.schema.json` - Pricing
- `submodules/opencode/schema/modelLimits.schema.json` - Context/output limits
- `submodules/opencode/schema/modelStatus.schema.json` - Lifecycle status
- `submodules/opencode/schema/modelAPI.schema.json` - API configuration
- `submodules/opencode/schema/ioCapabilities.schema.json` - Input/output modalities

**Previously derived from (now superseded):**

- `packages/opencode/src/provider/provider.ts` @ `d72d7ab` lines 410-470
- `packages/opencode/src/provider/models.ts` @ `81fef60` lines 12-51

---

## Messages

```protobuf
syntax = "proto3";
package opencode.model;

import "google/protobuf/struct.proto";

// Model metadata (matches OpenCode server Provider.Model schema)
// Source: submodules/opencode/schema/modelInfo.schema.json (canonical)
message ModelInfo {
  string id = 1;                           // "claude-3-5-sonnet-20241022"
  string provider_id = 2;                  // "anthropic" (JSON: providerID)
  string name = 3;                         // "Claude 3.5 Sonnet"
  optional string family = 4;              // Model family name (e.g., "claude-3.5")
  ModelAPI api = 5;                        // API configuration
  ModelCapabilities capabilities = 6;      // What the model supports (bools)
  ModelCost cost = 7;                      // Pricing information
  ModelLimits limit = 8;                   // Context/output limits (JSON: "limit" singular)
  ModelStatus status = 9;                  // "alpha" | "beta" | "deprecated" | "active"
  map<string, google.protobuf.Value> options = 10;  // Configurable settings (flexible map)
  map<string, string> headers = 11;        // Custom headers for API calls
  string release_date = 12;                // Model release date (required)
  map<string, google.protobuf.Struct> variants = 13; // Model variants with custom configs (optional)
}

// API configuration for model
// Source: submodules/opencode/schema/modelAPI.schema.json (canonical)
message ModelAPI {
  string id = 1;     // API model ID (may differ from display ID)
  string url = 2;    // API endpoint URL
  string npm = 3;    // npm package for SDK
}

// Model capabilities (what the model can do - boolean flags)
// Source: submodules/opencode/schema/modelCapabilities.schema.json (canonical)
message ModelCapabilities {
  bool temperature = 1;    // Supports temperature parameter
  bool reasoning = 2;      // Supports extended reasoning
  bool attachment = 3;     // Supports file attachments
  bool toolcall = 4;       // Supports tool calls
  IOCapabilities input = 5;
  IOCapabilities output = 6;
  InterleavedCapability interleaved = 7;  // Supports interleaved content
}

// Interleaved content capability (can be simple bool or object with field spec)
// Source: submodules/opencode/schema/modelCapabilities.schema.json
message InterleavedCapability {
  oneof value {
    bool enabled = 1;                      // Simple boolean: true/false
    InterleavedFieldSpec field_spec = 2;   // Object with field specification
  }
}

message InterleavedFieldSpec {
  // "reasoning_content" or "reasoning_details"
  enum Field {
    FIELD_UNSPECIFIED = 0;
    REASONING_CONTENT = 1;
    REASONING_DETAILS = 2;
  }
  Field field = 1;
}

// Input/output modality capabilities
// Source: submodules/opencode/schema/ioCapabilities.schema.json (canonical)
message IOCapabilities {
  bool text = 1;
  bool audio = 2;
  bool image = 3;
  bool video = 4;
  bool pdf = 5;
}

// Model pricing (per token)
// Source: submodules/opencode/schema/modelCost.schema.json (canonical)
message ModelCost {
  double input = 1;                               // Input token cost
  double output = 2;                              // Output token cost
  CacheCost cache = 3;                            // Cache costs (required)
  optional ExperimentalPricing experimental_over_200k = 4;  // JSON: experimentalOver200K
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
// Source: submodules/opencode/schema/modelLimits.schema.json (canonical)
message ModelLimits {
  double context = 1;  // Context window size (use double to match server z.number())
  double output = 2;   // Max output tokens
}

// Model status
// Source: submodules/opencode/schema/modelStatus.schema.json (canonical)
enum ModelStatus {
  MODEL_STATUS_UNSPECIFIED = 0;
  ALPHA = 1;
  BETA = 2;
  ACTIVE = 3;
  DEPRECATED = 4;
}

// Model configurable options (actual runtime settings)
// Note: ModelInfo.options uses flexible map; this typed version is for UI/validation
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
  ProviderInfo provider = 1;  // Provider with curated models list
  string model_id = 2;        // "claude-3-5-sonnet-20241022" (must exist in provider.models)
  string name = 3;            // "Claude 3.5 Sonnet" (for display)
}
```

---

## Design Notes

**Why a separate file?**

- Models have 15+ messages (capabilities, cost, limits, options, etc.)
- Provider-specific options are complex (OpenAI has 16 fields, Google/Anthropic have nested configs)
- Keeps `session.proto` focused on session/tab management
- Allows model schema to evolve independently

**Key decisions:**

- `options` uses `google.protobuf.Value` map to match JSON Schema's `additionalProperties: true`
- `variants` uses `google.protobuf.Struct` map for nested flexible objects
- `ModelCapabilities` are booleans (what model supports)
- `interleaved` is a union type (bool OR object with field enum) - modeled as oneof
- `ModelCost` uses `double` for per-token pricing (matches server schema)
- `limit` (singular) matches JSON Schema naming (was `limits` in earlier proto versions)
- Field numbers shifted to accommodate new fields (`family`, `release_date`, `variants`)

---

## JSON Schema Cross-Reference

| Protobuf Field | JSON Schema Property | Notes |
|----------------|---------------------|-------|
| `provider_id` | `providerID` | Naming convention (snake_case vs camelCase) |
| `limit` | `limit` | Singular in both (was `limits` in earlier proto) |
| `experimental_over_200k` | `experimentalOver200K` | Naming convention |
| `options` | `options` | Both use flexible map (`additionalProperties: true`) |
| `interleaved` | `interleaved` | Proto uses oneof, JSON uses oneOf |
