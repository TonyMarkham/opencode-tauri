// Unit tests for field_normalizer module
// Tests key transformations, round-trip safety, and JSON recursion

use crate::field_normalizer::{denormalize_json, denormalize_key, normalize_json, normalize_key};
use serde_json::json;

// ============================================
// UNIT TESTS: Individual Key Transformations
// ============================================

/// **VALUE**: Verifies all 12 acronym fields (ID, URL) are correctly transformed.
///
/// **WHY THIS MATTERS**: These are the problematic fields that standard case converters
/// fail on (projectID → project_i_d vs project_id). This is the core use case.
///
/// **BUG THIS CATCHES**: If acronym expansion algorithm breaks, these tests fail immediately.
#[test]
fn given_acronym_fields_when_normalize_key_then_converts_to_snake_case() {
    // ID suffix fields
    assert_eq!(normalize_key("projectID"), "project_id");
    assert_eq!(normalize_key("sessionID"), "session_id");
    assert_eq!(normalize_key("messageID"), "message_id");
    assert_eq!(normalize_key("providerID"), "provider_id");
    assert_eq!(normalize_key("modelID"), "model_id");
    assert_eq!(normalize_key("parentID"), "parent_id");
    assert_eq!(normalize_key("partID"), "part_id");
    assert_eq!(normalize_key("callID"), "call_id");
    assert_eq!(normalize_key("requestID"), "request_id");
    assert_eq!(normalize_key("snapshotID"), "snapshot_id");
    assert_eq!(normalize_key("subtaskID"), "subtask_id");

    // URL suffix fields
    assert_eq!(normalize_key("baseURL"), "base_url");
}

/// **VALUE**: Verifies the 3 explicit override fields that don't follow acronym rules.
///
/// **WHY THIS MATTERS**: Edge cases like "enterpriseUrl" (mixed case Url) and
/// "topP" (single letter) need explicit handling. Without overrides, these would fail.
///
/// **BUG THIS CATCHES**: If override merge logic breaks or precedence is wrong,
/// these fields would use incorrect transformations.
#[test]
fn given_override_fields_when_normalize_key_then_uses_explicit_mapping() {
    assert_eq!(normalize_key("enterpriseUrl"), "enterprise_url");
    assert_eq!(
        normalize_key("experimentalOver200K"),
        "experimental_over_200_k"
    );
    assert_eq!(normalize_key("topP"), "top_p");
}

/// **VALUE**: Verifies unknown fields pass through unchanged (zero-copy optimization).
///
/// **WHY THIS MATTERS**: Not all fields need transformation. Unknown fields should
/// pass through untouched, avoiding unnecessary allocations.
///
/// **BUG THIS CATCHES**: Would catch if we accidentally transformed all fields with
/// a generic algorithm instead of using the lookup table.
#[test]
fn given_unknown_field_when_normalize_key_then_returns_unchanged() {
    // Fields not in our mapping should pass through
    assert_eq!(normalize_key("unknownField"), "unknownField");
    assert_eq!(normalize_key("some_snake_case"), "some_snake_case");
    assert_eq!(normalize_key("ALLCAPS"), "ALLCAPS");
}

/// **VALUE**: Verifies reverse transformation (snake_case → JavaScript).
///
/// **WHY THIS MATTERS**: We need bidirectional transformation for sending requests
/// to OpenCode server (Rust snake_case → JavaScript camelCase).
///
/// **BUG THIS CATCHES**: Would catch if TO_JS lookup table wasn't generated correctly
/// or if denormalize_key logic is broken.
#[test]
fn given_snake_case_fields_when_denormalize_key_then_converts_to_javascript() {
    // Acronym fields
    assert_eq!(denormalize_key("project_id"), "projectID");
    assert_eq!(denormalize_key("session_id"), "sessionID");
    assert_eq!(denormalize_key("base_url"), "baseURL");

    // Overrides
    assert_eq!(denormalize_key("enterprise_url"), "enterpriseUrl");
    assert_eq!(denormalize_key("top_p"), "topP");

    // Unknown fields pass through
    assert_eq!(denormalize_key("unknown_field"), "unknown_field");
}

// ============================================
// ROUND-TRIP PROPERTY TESTS
// ============================================

/// **VALUE**: Verifies round-trip safety for all known mappings.
///
/// **WHY THIS MATTERS**: If normalize(denormalize(x)) != x or denormalize(normalize(x)) != x,
/// we have data loss. This is critical for correctness.
///
/// **BUG THIS CATCHES**: Would catch if mappings aren't bijective (one-to-one correspondence).
/// This is the most important property of the normalizer.
#[test]
fn given_all_mappings_when_round_trip_then_returns_original() {
    // All acronym fields
    let acronym_fields = vec![
        ("projectID", "project_id"),
        ("sessionID", "session_id"),
        ("messageID", "message_id"),
        ("providerID", "provider_id"),
        ("modelID", "model_id"),
        ("parentID", "parent_id"),
        ("partID", "part_id"),
        ("callID", "call_id"),
        ("requestID", "request_id"),
        ("snapshotID", "snapshot_id"),
        ("subtaskID", "subtask_id"),
        ("baseURL", "base_url"),
    ];

    // All override fields
    let override_fields = vec![
        ("enterpriseUrl", "enterprise_url"),
        ("experimentalOver200K", "experimental_over_200_k"),
        ("topP", "top_p"),
    ];

    // Test round-trip both directions
    for (js_key, snake_key) in acronym_fields.iter().chain(override_fields.iter()) {
        // JavaScript → snake_case → JavaScript
        assert_eq!(
            denormalize_key(&normalize_key(js_key)),
            *js_key,
            "Round-trip failed for JS key: {}",
            js_key
        );

        // snake_case → JavaScript → snake_case
        assert_eq!(
            normalize_key(&denormalize_key(snake_key)),
            *snake_key,
            "Round-trip failed for snake_case key: {}",
            snake_key
        );
    }
}

// ============================================
// JSON TRANSFORMATION TESTS
// ============================================

/// **VALUE**: Verifies recursive transformation of nested JSON objects.
///
/// **WHY THIS MATTERS**: OpenCode returns complex nested JSON. We need to transform
/// field names at all levels, not just the top level.
///
/// **BUG THIS CATCHES**: Would catch if recursion logic is broken or if we only
/// transform top-level keys.
#[test]
fn given_nested_json_when_normalize_json_then_transforms_all_levels() {
    let input = json!({
        "projectID": "proj_123",
        "sessionID": "ses_456",
        "nested": {
            "messageID": "msg_789",
            "deeper": {
                "providerID": "anthropic"
            }
        }
    });

    let expected = json!({
        "project_id": "proj_123",
        "session_id": "ses_456",
        "nested": {
            "message_id": "msg_789",
            "deeper": {
                "provider_id": "anthropic"
            }
        }
    });

    assert_eq!(normalize_json(input), expected);
}

/// **VALUE**: Verifies transformation preserves arrays and transforms elements.
///
/// **WHY THIS MATTERS**: OpenCode returns arrays of objects (sessions, agents, etc.).
/// We need to transform objects inside arrays recursively.
///
/// **BUG THIS CATCHES**: Would catch if array handling is missing or broken.
#[test]
fn given_json_with_arrays_when_normalize_json_then_transforms_array_elements() {
    let input = json!({
        "sessions": [
            {"sessionID": "ses_1", "projectID": "proj_a"},
            {"sessionID": "ses_2", "projectID": "proj_b"}
        ]
    });

    let expected = json!({
        "sessions": [
            {"session_id": "ses_1", "project_id": "proj_a"},
            {"session_id": "ses_2", "project_id": "proj_b"}
        ]
    });

    assert_eq!(normalize_json(input), expected);
}

/// **VALUE**: Verifies non-object values (strings, numbers, booleans, null) pass through.
///
/// **WHY THIS MATTERS**: JSON has primitives that don't have field names. These should
/// remain unchanged during transformation.
///
/// **BUG THIS CATCHES**: Would catch if we accidentally try to transform primitive values.
#[test]
fn given_primitive_values_when_normalize_json_then_preserves_values() {
    assert_eq!(normalize_json(json!("string")), json!("string"));
    assert_eq!(normalize_json(json!(42)), json!(42));
    assert_eq!(normalize_json(json!(3.14)), json!(3.14));
    assert_eq!(normalize_json(json!(true)), json!(true));
    assert_eq!(normalize_json(json!(false)), json!(false));
    assert_eq!(normalize_json(json!(null)), json!(null));
}

/// **VALUE**: Verifies empty objects and arrays are handled correctly.
///
/// **WHY THIS MATTERS**: Edge case - empty collections should work without errors.
///
/// **BUG THIS CATCHES**: Would catch if empty iterator handling causes panics.
#[test]
fn given_empty_collections_when_normalize_json_then_returns_empty() {
    assert_eq!(normalize_json(json!({})), json!({}));
    assert_eq!(normalize_json(json!([])), json!([]));
}

/// **VALUE**: Verifies denormalize_json works (reverse transformation).
///
/// **WHY THIS MATTERS**: We need to send snake_case Rust structs to OpenCode server
/// with JavaScript field names.
///
/// **BUG THIS CATCHES**: Would catch if denormalize_json is broken or asymmetric.
#[test]
fn given_snake_case_json_when_denormalize_json_then_converts_to_javascript() {
    let input = json!({
        "project_id": "proj_123",
        "session_id": "ses_456",
        "nested": {
            "message_id": "msg_789"
        }
    });

    let expected = json!({
        "projectID": "proj_123",
        "sessionID": "ses_456",
        "nested": {
            "messageID": "msg_789"
        }
    });

    assert_eq!(denormalize_json(input), expected);
}

/// **VALUE**: Verifies full round-trip on complex realistic JSON.
///
/// **WHY THIS MATTERS**: End-to-end test with realistic OpenCode response structure.
/// This is what we'll actually use in production.
///
/// **BUG THIS CATCHES**: Would catch any bugs in the complete transformation pipeline.
#[test]
fn given_realistic_opencode_json_when_round_trip_then_returns_original() {
    let opencode_json = json!({
        "projectID": "proj_abc",
        "sessionID": "ses_123",
        "parentID": "ses_000",
        "title": "Test Session",
        "time": {
            "created": 1234567890,
            "updated": 1234567999
        },
        "summary": {
            "additions": 42,
            "deletions": 7,
            "files": 3
        }
    });

    // Normalize (OpenCode → Rust)
    let normalized = normalize_json(opencode_json.clone());

    // Verify transformation happened
    assert_eq!(normalized["project_id"], "proj_abc");
    assert_eq!(normalized["session_id"], "ses_123");
    assert_eq!(normalized["parent_id"], "ses_000");

    // Denormalize (Rust → OpenCode)
    let denormalized = denormalize_json(normalized);

    // Should match original exactly
    assert_eq!(denormalized, opencode_json);
}
