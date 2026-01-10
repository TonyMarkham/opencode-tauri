use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const OPENCODE_FIELDS_TOML: &str = "opencode_fields.toml";
const OPENCODE_FIELDS_GENERATED_FILE: &str = "field_normalizer.rs";

fn main() {
    // Existing: Compile protobuf files
    compile_protos();

    // New: Generate field normalizer
    generate_field_normalizer();
}

fn compile_protos() {
    prost_build::Config::new()
        .type_attribute(".", "#[allow(clippy::large_enum_variant)]")
        .extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .extern_path(".google.protobuf.ListValue", "::prost_wkt_types::ListValue")
        .type_attribute(
            "IpcServerInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionTime",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionSummary",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .field_attribute(
            "opencode.session.OcSessionSummary.diffs",
            "#[serde(default)]",
        )
        .type_attribute(
            "opencode.session.OcFileDiff",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionShare",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionRevert",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcPermissionAction",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcPermissionRule",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcPermissionRuleset",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.session.OcSessionList",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.OcAssistantMessage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.session_id",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.role",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.model",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.parts",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.text",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.tokens",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.cost",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcAssistantMessage.error",
            "#[serde(default)]",
        )
        .type_attribute(
            "opencode.message.OcUserMessage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.OcMessage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.OcModelReference",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .field_attribute(
            "opencode.message.OcModelReference.model_id",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcModelReference.provider_id",
            "#[serde(default)]",
        )
        .type_attribute(
            "opencode.message.OcTokens",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.OcMessageError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcTextPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcReasoningPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.OcMessage.message",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcPart.part",
            "#[derive(serde::Serialize, serde::Deserialize)] #[serde(rename_all = \"snake_case\")]",
        )
        .field_attribute("opencode.message.part.OcPart.part", "#[serde(flatten)]")
        .type_attribute(
            "opencode.message.OcTokenUsage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .field_attribute("opencode.message.OcTokenUsage.input", "#[serde(default)]")
        .field_attribute("opencode.message.OcTokenUsage.output", "#[serde(default)]")
        .field_attribute(
            "opencode.message.OcTokenUsage.cache_read",
            "#[serde(default)]",
        )
        .field_attribute(
            "opencode.message.OcTokenUsage.cache_write",
            "#[serde(default)]",
        )
        .type_attribute(
            "opencode.message.error.OcMessageError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcMessageError.error",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcApiError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcProviderAuthError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcUnknownError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcOutputLengthError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.error.OcAbortedError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcToolPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcFilePart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcPatchPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcSnapshotPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcAgentPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcCompactionPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcSubtaskPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcStepStartPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcStepFinishPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcRetryPart",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.message.part.OcTokenUsage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolState",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolState.state",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolStatePending",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolStateRunning",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolStateCompleted",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolStateError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolTime",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "opencode.tool.OcToolTimeWithEnd",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(
            &[
                // OpenCode canonical models (from JSON Schemas)
                "../../proto/oc_auth.proto",
                "../../proto/oc_model.proto",
                "../../proto/oc_provider.proto",
                "../../proto/oc_session.proto",
                "../../proto/oc_agent.proto",
                "../../proto/oc_tool.proto",
                "../../proto/oc_message.proto",
                "../../proto/oc_message_part.proto",
                "../../proto/oc_message_error.proto",
                "../../proto/oc_event.proto",
                // IPC protocol layer
                "../../proto/ipc.proto",
            ],
            &["../../proto/"],
        )
        .unwrap();
}

fn generate_field_normalizer() {
    // Read configuration file
    let config_path = PathBuf::from(OPENCODE_FIELDS_TOML);
    let config_content = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("Failed to read opencode_fields.toml: {e}"));

    let config: FieldConfig = toml::from_str(&config_content)
        .unwrap_or_else(|e| panic!("Failed to parse opencode_fields.toml: {e}"));

    // Validate mappings
    validate_mappings(&config.mappings);

    // Generate Rust code
    let code = generate_rust_code(&config.mappings);

    // Write to OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join(OPENCODE_FIELDS_GENERATED_FILE);
    fs::write(&dest_path, code)
        .unwrap_or_else(|e| panic!("Failed to write field_normalizer.rs: {e}"));

    // Rebuild if config changes
    println!("cargo:rerun-if-changed={OPENCODE_FIELDS_TOML}");
}

#[derive(serde::Deserialize)]
struct FieldConfig {
    mappings: HashMap<String, String>,
}

/// Validate mappings for correctness
fn validate_mappings(mappings: &HashMap<String, String>) {
    // Check for duplicate JavaScript keys (HashMap guarantees this, but validate for clarity)
    let js_keys: Vec<_> = mappings.keys().collect();
    if js_keys.len() != mappings.len() {
        panic!("Internal error: duplicate JavaScript keys in mappings");
    }

    // Check for duplicate snake_case values
    let mut snake_values: Vec<_> = mappings.values().collect();
    snake_values.sort();
    for window in snake_values.windows(2) {
        if window[0] == window[1] {
            panic!(
                "Duplicate snake_case mapping: '{}' appears multiple times",
                window[0]
            );
        }
    }

    // Verify round-trip safety (every mapping has a reverse)
    let reverse: HashMap<_, _> = mappings.iter().map(|(k, v)| (v, k)).collect();
    if reverse.len() != mappings.len() {
        panic!("Mappings are not bijective (cannot round-trip safely)");
    }
}

/// Generate Rust code with lookup tables and transformation functions
fn generate_rust_code(mappings: &HashMap<String, String>) -> String {
    let mut code = String::new();

    // Header
    code.push_str("// Generated by build.rs - DO NOT EDIT\n");
    code.push_str("// Source: opencode_fields.toml\n\n");
    code.push_str("use once_cell::sync::Lazy;\n");
    code.push_str("use std::borrow::Cow;\n");
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use serde_json::Value;\n\n");

    // TO_SNAKE lookup table (JavaScript → snake_case)
    code.push_str("/// JavaScript field name → snake_case field name\n");
    code.push_str(&format!(
        "static TO_SNAKE: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {{\n    let mut m = HashMap::with_capacity({});\n",
        mappings.len()
    ));

    let mut sorted_mappings: Vec<_> = mappings.iter().collect();
    sorted_mappings.sort_by_key(|(k, _)| *k);

    for (js_key, snake_key) in &sorted_mappings {
        code.push_str(&format!(
            "    m.insert(\"{}\", \"{}\");\n",
            js_key, snake_key
        ));
    }
    code.push_str("    m\n});\n\n");

    // TO_JS lookup table (snake_case → JavaScript)
    code.push_str("/// snake_case field name → JavaScript field name\n");
    code.push_str(&format!(
        "static TO_JS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {{\n    let mut m = HashMap::with_capacity({});\n",
        mappings.len()
    ));

    let mut sorted_reverse: Vec<_> = mappings.iter().map(|(k, v)| (v, k)).collect();
    sorted_reverse.sort_by_key(|(k, _)| *k);

    for (snake_key, js_key) in &sorted_reverse {
        code.push_str(&format!(
            "    m.insert(\"{}\", \"{}\");\n",
            snake_key, js_key
        ));
    }
    code.push_str("    m\n});\n\n");

    // normalize_key function
    code.push_str("/// Transform a single JavaScript field name to snake_case\n");
    code.push_str("/// Returns Cow::Borrowed if no transformation needed (zero-copy)\n");
    code.push_str("pub fn normalize_key(key: &str) -> Cow<'_, str> {\n");
    code.push_str("    TO_SNAKE.get(key)\n");
    code.push_str("        .map(|&s| Cow::Borrowed(s))\n");
    code.push_str("        .unwrap_or_else(|| Cow::Borrowed(key))\n");
    code.push_str("}\n\n");

    // denormalize_key function
    code.push_str("/// Transform a single snake_case field name to JavaScript\n");
    code.push_str("/// Returns Cow::Borrowed if no transformation needed (zero-copy)\n");
    code.push_str("pub fn denormalize_key(key: &str) -> Cow<'_, str> {\n");
    code.push_str("    TO_JS.get(key)\n");
    code.push_str("        .map(|&s| Cow::Borrowed(s))\n");
    code.push_str("        .unwrap_or_else(|| Cow::Borrowed(key))\n");
    code.push_str("}\n\n");

    // normalize_json function
    code.push_str("/// Transform JavaScript field names to snake_case recursively\n");
    code.push_str("/// Use this on JSON received from OpenCode server\n");
    code.push_str("pub fn normalize_json(value: Value) -> Value {\n");
    code.push_str("    match value {\n");
    code.push_str("        Value::Object(map) => {\n");
    code.push_str("            let normalized = map.into_iter()\n");
    code.push_str(
        "                .map(|(k, v)| (normalize_key(&k).into_owned(), normalize_json(v)))\n",
    );
    code.push_str("                .collect();\n");
    code.push_str("            Value::Object(normalized)\n");
    code.push_str("        }\n");
    code.push_str("        Value::Array(arr) => {\n");
    code.push_str("            Value::Array(arr.into_iter().map(normalize_json).collect())\n");
    code.push_str("        }\n");
    code.push_str("        other => other,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // denormalize_json function
    code.push_str("/// Transform snake_case field names to JavaScript recursively\n");
    code.push_str("/// Use this on JSON being sent to OpenCode server\n");
    code.push_str("pub fn denormalize_json(value: Value) -> Value {\n");
    code.push_str("    match value {\n");
    code.push_str("        Value::Object(map) => {\n");
    code.push_str("            let denormalized = map.into_iter()\n");
    code.push_str(
        "                .map(|(k, v)| (denormalize_key(&k).into_owned(), denormalize_json(v)))\n",
    );
    code.push_str("                .collect();\n");
    code.push_str("            Value::Object(denormalized)\n");
    code.push_str("        }\n");
    code.push_str("        Value::Array(arr) => {\n");
    code.push_str("            Value::Array(arr.into_iter().map(denormalize_json).collect())\n");
    code.push_str("        }\n");
    code.push_str("        other => other,\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}
