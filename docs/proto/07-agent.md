# Agent Management (`agent.proto`)

**Status:** ✅ Complete (JSON Schema created)  
**Last Updated:** 2026-01-05

---

## Purpose

List available agents with metadata (name, description, mode, permissions, model configuration).

---

## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/agentInfo.schema.json` - Agent configuration and metadata
- `submodules/opencode/schema/agentModel.schema.json` - Model selection for an agent

**Previously derived from (now superseded):**

- `packages/opencode/src/agent/agent.ts` @ `a14f9d216` lines 17-41 (Agent.Info)

---

## Generated Validators

The following validators are auto-generated from JSON Schema:

```
submodules/opencode/packages/opencode/generated/validators/
├── agentInfo.ts
└── agentModel.ts
```

Import with `@generated/validators/<name>`.

---

## Messages

```protobuf
syntax = "proto3";
package opencode.agent;

import "permission.proto";  // For PermissionRuleset

// Agent configuration and metadata
// Source: agentInfo.schema.json
message AgentInfo {
  string name = 1;                          // Agent name identifier (required)
  optional string description = 2;          // Human-readable description
  string mode = 3;                          // "subagent", "primary", or "all" (required)
  optional bool native = 4;                 // Whether this is a built-in native agent
  optional bool hidden = 5;                 // Whether to hide from UI listings
  optional double top_p = 6;                // Top-p sampling parameter
  optional double temperature = 7;          // Temperature parameter
  optional string color = 8;                // Hex color for UI differentiation
  repeated PermissionRule permission = 9;   // Permission ruleset (required)
  optional AgentModel model = 10;           // Default model selection
  optional string prompt = 11;              // Custom system prompt
  google.protobuf.Struct options = 12;      // Additional agent-specific options (required)
  optional int32 steps = 13;                // Maximum inference steps (minimum: 1)
}

// Model selection for an agent
// Source: agentModel.schema.json
message AgentModel {
  string model_id = 1;                      // Model identifier (required)
  string provider_id = 2;                   // Provider identifier (required)
}

message AgentList {
  repeated AgentInfo agents = 1;
}

message Empty {}
```

---

## Service Definition

```protobuf
service AgentService {
  rpc ListAgents(Empty) returns (AgentList);
}
```

---

## Maps to OpenCode Server

- `AgentService.ListAgents` → `GET /agent`

---

## JSON Schema Cross-Reference

### AgentInfo Fields

| JSON Schema Property | Type | Required | Protobuf Field | Type | Notes |
|---------------------|------|----------|----------------|------|-------|
| `name` | string | ✓ | `string name` | string | |
| `description` | string | no | `optional string description` | string | |
| `mode` | string (enum) | ✓ | `string mode` | string | "subagent", "primary", "all" |
| `native` | boolean | no | `optional bool native` | bool | |
| `hidden` | boolean | no | `optional bool hidden` | bool | |
| `topP` | number | no | `optional double top_p` | double | camelCase → snake_case |
| `temperature` | number | no | `optional double temperature` | double | |
| `color` | string | no | `optional string color` | string | |
| `permission` | $ref PermissionRuleset | ✓ | `repeated PermissionRule permission` | repeated | Array of rules |
| `model` | $ref AgentModel | no | `optional AgentModel model` | message | |
| `prompt` | string | no | `optional string prompt` | string | |
| `options` | object (additionalProperties) | ✓ | `Struct options` | Struct | Flexible key-value |
| `steps` | integer (min: 1) | no | `optional int32 steps` | int32 | |

### AgentModel Fields

| JSON Schema Property | Type | Required | Protobuf Field | Type | Notes |
|---------------------|------|----------|----------------|------|-------|
| `modelID` | string | ✓ | `string model_id` | string | camelCase → snake_case |
| `providerID` | string | ✓ | `string provider_id` | string | camelCase → snake_case |

---

## Step 5.3 Protobuf Verification Summary

| # | Schema | Protobuf Message | Fields Verified | Types Match | Optionality Match | Result |
|---|--------|------------------|-----------------|-------------|-------------------|--------|
| 1 | agentInfo.schema.json | AgentInfo | 13/13 ✅ | All match | All match | ✅ VERIFIED |
| 2 | agentModel.schema.json | AgentModel | 2/2 ✅ | All match | All match | ✅ VERIFIED |

**Total schemas verified:** 2/2  
**Nested types verified:** AgentModel, PermissionRuleset (from permission.proto)  
**Confidence level:** 100%

---

## Design Notes

**Agent Types:**

- `native: true` - Core built-in agents (build, plan, explore, compaction, title, summary)
- `native: false` - User-defined or plugin agents

**Agent Mode:**

- `"primary"` - Standard agent for direct user interaction
- `"subagent"` - Spawned by other agents (via Task tool)
- `"all"` - Can function in both modes

**Permission Ruleset:**

- Array of permission rules defining what tools/actions the agent can use
- References `PermissionRule` from `permission.proto` / `permissionRule.schema.json`

**Options:**

- Flexible key-value store for agent-specific configuration
- Uses `google.protobuf.Struct` in protobuf (maps to `additionalProperties: true` in JSON Schema)

**Model Configuration:**

- Optional default model override for the agent
- Contains `providerID` and `modelID` for model selection

---

## Refactored TypeScript Source

The following types now use generated validators:

```typescript
// packages/opencode/src/agent/agent.ts
import { agentInfoSchema, type AgentInfo } from "@generated/validators/agentInfo"

export namespace Agent {
  // Generated from JSON Schema - see schema/agentInfo.schema.json
  export const Info = agentInfoSchema
  export type Info = AgentInfo
}
```

---

## Verification

- ✅ All 74 JSON Schema files valid (`bun run generate:schemas`)
- ✅ All generated validators match original Zod (field-by-field verified)
- ✅ TypeScript typecheck passes (`bun run typecheck`)
- ✅ All 544 tests pass (`bun test`)
- ✅ Production build succeeds for 11 platforms (`bun run build`)

---

## TODO

- [x] Create `agentInfo.schema.json` for agent metadata
- [x] Create `agentModel.schema.json` for model selection
- [x] Update generator to support `minimum`/`maximum` constraints
- [x] Refactor TypeScript to use generated validators
- [x] Add JSON Schema cross-reference tables
- [x] Complete Step 5.3 protobuf verification
- [ ] Document built-in agents vs plugin agents in detail
- [ ] Validate against actual agent API responses
