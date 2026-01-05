# Provider Management (`provider.proto`)

**Status:** ✅ Complete  
**Last Updated:** 2026-01-04

---

## Purpose

Define AI model providers (Anthropic, OpenAI, Google, etc.) and their available models. Providers expose collections of models with specific configurations, authentication requirements, and API endpoints.

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/providerInfo.schema.json` - Core provider metadata
- `submodules/opencode/schema/providerSource.schema.json` - Configuration source enum
- `submodules/opencode/schema/providerOptions.schema.json` - SDK initialization options
- `submodules/opencode/schema/providerList.schema.json` - API response for provider listing

**Previously derived from (now superseded):**

- `packages/opencode/src/provider/provider.ts` @ `d72d7ab` lines 479-491

---

## Messages

```protobuf
syntax = "proto3";
package opencode.provider;

import "model.proto";
import "google/protobuf/struct.proto";

// Provider metadata (aggregates models and configuration)
// Source: submodules/opencode/schema/providerInfo.schema.json (canonical)
message ProviderInfo {
  string id = 1;                            // "anthropic", "openai", etc.
  string name = 2;                          // "Anthropic" (display name)
  ProviderSource source = 3;                // How provider was configured
  repeated string env = 4;                  // Environment variables used (e.g., ["ANTHROPIC_API_KEY"])
  optional string key = 5;                  // API key hint (partial/masked)
  ProviderOptions options = 6;              // Provider-level SDK initialization options
  map<string, ModelInfo> models = 7;        // Curated models keyed by model ID
}

// Provider SDK initialization options
// Source: submodules/opencode/schema/providerOptions.schema.json (canonical)
// Note: Flexible schema - additionalProperties: true allows provider-specific options
message ProviderOptions {
  optional string base_url = 1;             // API endpoint override (JSON: baseURL)
                                            // Examples: "https://api.openai.com/v1", "https://api.anthropic.com"
  optional string api_key = 2;              // API authentication key (JSON: apiKey)
  map<string, string> headers = 3;          // Custom HTTP headers
                                            // Example: {"anthropic-beta": "claude-code-20250219,interleaved-thinking-2025-05-14"}
  optional int32 timeout = 4;               // Request timeout in milliseconds (minimum: 0)
  
  // Additional provider-specific options stored as flexible struct
  // Examples:
  //   - Azure: useCompletionUrls (bool)
  //   - Bedrock: region, credentialProvider
  //   - Vertex: project, location
  google.protobuf.Struct extra = 5;
}

// How the provider was configured/discovered
// Source: submodules/opencode/schema/providerSource.schema.json (canonical)
enum ProviderSource {
  PROVIDER_SOURCE_UNSPECIFIED = 0;
  ENV = 1;        // Configured via environment variables
  CONFIG = 2;     // Configured via config file (opencode.json)
  CUSTOM = 3;     // Custom provider definition (plugin-based)
  API = 4;        // Discovered via API (auth.json)
}

// List of all available providers (API response)
// Source: submodules/opencode/schema/providerList.schema.json (canonical)
// Maps to: GET /config/providers
message ProviderList {
  repeated ProviderInfo providers = 1;      // All configured providers with curated models
  map<string, string> default = 2;          // Default model ID per provider ID
                                            // Example: {"anthropic": "claude-3-5-sonnet-20241022", "openai": "gpt-4"}
}

message Empty {}
```

---

## Service Definition

```protobuf
service ProviderService {
  rpc GetProviders(Empty) returns (ProviderList);  // Get all providers with curated models
}
```

---

## Maps to OpenCode Server

- `ProviderService.GetProviders` → `GET /config/providers`
  - Returns all configured providers with their curated model lists
  - Includes default provider/model selections

---

## Design Notes

**Why separate from model.proto?**

- Provider is the **container**, Model is the **content**
- Providers have their own lifecycle (auth, initialization, discovery)
- Clear ownership: ProviderInfo owns the models map
- API endpoint mapping: `GET /config/providers` returns ProviderList

**Key decisions:**

- `ProviderOptions` uses typed fields for common options + `extra` Struct for provider-specific
- `models` is a `map<string, ModelInfo>` (keyed by model ID) matching JSON Schema's `additionalProperties`
- `ProviderSource` enum matches JSON Schema's `enum: ["env", "config", "custom", "api"]`
- `ProviderList.default` maps provider ID → default model ID for that provider

**Provider-specific option examples:**

| Provider | Extra Options |
|----------|---------------|
| Azure | `useCompletionUrls: bool` |
| Bedrock | `region: string`, `credentialProvider: object` |
| Vertex | `project: string`, `location: string` |
| OpenRouter | `HTTP-Referer: string`, `X-Title: string` (in headers) |
| Anthropic | `anthropic-beta: string` (in headers) |

---

## JSON Schema Cross-Reference

| Protobuf Field | JSON Schema Property | Notes |
|----------------|---------------------|-------|
| `base_url` | `baseURL` | Naming convention (snake_case vs camelCase) |
| `api_key` | `apiKey` | Naming convention |
| `models` | `models` | Both use map keyed by model ID |
| `ProviderSource` | `source` | Enum values match: env, config, custom, api |
