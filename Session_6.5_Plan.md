# Session 6.5: JSON Field Name Normalizer

> **⚠️ CRITICAL: Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting this session.**

**Status:** ✅ COMPLETE (2026-01-07)  
**Prerequisite:** Session 6 complete  
**Blocks:** Session 7 (Session Handlers need this to parse OpenCode JSON)

---

## Goal

Implement [ADR-0004](docs/adr/0004-json-field-name-normalization.md): Build-time code generation for bidirectional JSON field name transformation between OpenCode's JavaScript-style naming (`projectID`) and protobuf snake_case (`project_id`).

---

## Problem Statement

OpenCode server returns JSON with JavaScript-style field names:
```json
{ "projectID": "abc", "sessionID": "xyz", "cacheRead": 100 }
```

Our proto types use snake_case:
```rust
struct OcSessionInfo { project_id: String, session_id: String, cache_read: i32 }
```

**Standard case-conversion libraries fail on uppercase acronyms:**
- `heck`: `projectID` → `project_i_d` ❌
- `convert_case`: `projectID` → `project_i_d` ❌
- We need: `projectID` → `project_id` ✅

---

## Deliverables

| # | Deliverable | Description | Status |
|---|-------------|-------------|--------|
| 1 | `opencode_fields.toml` | Configuration: acronym list + explicit overrides | ✅ Complete |
| 2 | Modified `build.rs` | Reads TOML, expands rules, generates Rust code | ✅ Complete |
| 3 | Generated `field_normalizer.rs` | Lookup tables + transformation functions | ✅ Complete |
| 4 | Unit tests | Key transformation + round-trip verification | ✅ Complete (12 tests) |

---

## Files Created/Modified

| File | Action | Status |
|------|--------|--------|
| `Cargo.toml` (workspace) | Add `toml`, `once_cell` dependencies | ✅ Complete |
| `backend/client-core/Cargo.toml` | Add `serde_json`, `once_cell`, `serde` build-deps | ✅ Complete |
| `backend/client-core/opencode_fields.toml` | Create - config source of truth | ✅ Complete |
| `backend/client-core/build.rs` | Modify - add normalizer generation | ✅ Complete |
| `backend/client-core/src/field_normalizer.rs` | Create - include! wrapper | ✅ Complete |
| `backend/client-core/src/lib.rs` | Modify - add field_normalizer module | ✅ Complete |
| `backend/client-core/src/tests/field_normalizer.rs` | Create - comprehensive test suite | ✅ Complete |
| `$OUT_DIR/field_normalizer.rs` | Generated - lookup tables + functions | ✅ Complete |

---

## Public API (Generated)

```rust
pub fn normalize_json(value: Value) -> Value;    // Response: projectID → project_id
pub fn denormalize_json(value: Value) -> Value;  // Request: project_id → projectID
pub fn normalize_key(key: &str) -> Cow<str>;     // Single key transform
pub fn denormalize_key(key: &str) -> Cow<str>;   // Single key reverse
```

---

## Success Criteria

- [x] `cargo build -p client-core` succeeds (generates normalizer)
- [x] `cargo test -p client-core` passes all tests (18 unit + 24 integration)
- [x] Round-trip property: `denormalize(normalize(x)) == x` verified in tests
- [x] Build fails if config has duplicate mappings (validated in `validate_mappings()`)
- [x] `cargo clippy -p client-core` passes with no warnings

**Completed:** 2026-01-07

---

## Out of Scope

- Integration with `OpencodeClient` (Session 7)
- Actual HTTP calls to OpenCode server (Session 7)
- Performance benchmarking (defer unless issues arise)

---

# Research: Complete Field Mapping Reference

**This section contains the results of schema analysis. Do not re-research - use these mappings.**

Source: OpenCode JSON schemas at `submodules/opencode/schema/*.schema.json`

## Acronym Fields (ID, URL)

Fields where uppercase acronyms need special handling.

| JavaScript | snake_case | Occurrences |
|------------|------------|-------------|
| `projectID` | `project_id` | SessionInfo |
| `sessionID` | `session_id` | All message parts, events, permissions (18+ schemas) |
| `messageID` | `message_id` | All message parts, events (16+ schemas) |
| `providerID` | `provider_id` | ModelInfo, AgentModel, AssistantMessage, errors |
| `modelID` | `model_id` | AgentModel, AssistantMessage, UserMessage.model |
| `parentID` | `parent_id` | SessionInfo, AssistantMessage |
| `partID` | `part_id` | SessionRevert, MessagePartRemovedEvent |
| `callID` | `call_id` | ToolPart, PermissionToolContext |
| `requestID` | `request_id` | PermissionRepliedEvent |
| `snapshotID` | `snapshot_id` | SnapshotPart |
| `subtaskID` | `subtask_id` | SubtaskPart |
| `baseURL` | `base_url` | ProviderOptions |

## Explicit Overrides (Edge Cases)

Fields that don't fit acronym rules and need explicit mapping.

| JavaScript | snake_case | Reason |
|------------|------------|--------|
| `enterpriseUrl` | `enterprise_url` | Mixed "Url" not "URL" |
| `experimentalOver200K` | `experimental_over_200_k` | Numeric + K suffix |
| `topP` | `top_p` | Single letter suffix |

## Standard CamelCase Fields

Standard camelCase that algorithms handle correctly, but we include for completeness.

| JavaScript | snake_case | Schema |
|------------|------------|--------|
| `cacheRead` | `cache_read` | TokenUsage |
| `cacheWrite` | `cache_write` | TokenUsage |
| `reasoningEffort` | `reasoning_effort` | OpenAIOptions |
| `reasoningSummary` | `reasoning_summary` | OpenAIOptions |
| `textVerbosity` | `text_verbosity` | OpenAIOptions |
| `serviceTier` | `service_tier` | OpenAIOptions |
| `maxToolCalls` | `max_tool_calls` | OpenAIOptions |
| `parallelToolCalls` | `parallel_tool_calls` | OpenAIOptions |
| `strictJsonSchema` | `strict_json_schema` | OpenAIOptions |
| `promptCacheKey` | `prompt_cache_key` | OpenAIOptions |
| `safetyIdentifier` | `safety_identifier` | OpenAIOptions |
| `previousResponseId` | `previous_response_id` | OpenAIOptions |
| `statusCode` | `status_code` | APIErrorData |
| `isRetryable` | `is_retryable` | APIErrorData |
| `responseHeaders` | `response_headers` | APIErrorData |
| `responseBody` | `response_body` | APIErrorData |
| `apiKey` | `api_key` | ProviderOptions |
| `builtIn` | `built_in` | AgentInfo |
| `maxOutputTokens` | `max_output_tokens` | UniversalOptions |
| `topLogprobs` | `top_logprobs` | LogprobsConfig |
| `thinkingConfig` | `thinking_config` | GoogleOptions |
| `includeThoughts` | `include_thoughts` | ThinkingConfig |
| `thinkingBudget` | `thinking_budget` | ThinkingConfig |
| `budgetTokens` | `budget_tokens` | ThinkingOptions |

## Summary Statistics

| Category | Count |
|----------|-------|
| Acronym fields (ID, URL) | 12 |
| Explicit overrides | 3 |
| Standard camelCase | 24 |
| **Total mappings** | **39** |

---

## Config File Template

Use this as the starting point for `opencode_fields.toml`:

```toml
# OpenCode JSON ↔ Protobuf Field Name Mappings
# Source of truth - read by build.rs at compile time
# See: docs/adr/0004-json-field-name-normalization.md

acronyms = ["ID", "URL"]

[overrides]
"enterpriseUrl" = "enterprise_url"
"experimentalOver200K" = "experimental_over_200_k"
"topP" = "top_p"
```

The build.rs should use the acronym list to generate all `*ID` → `*_id` and `*URL` → `*_url` mappings from the tables above.
