fn main() {
    prost_build::Config::new()
        .type_attribute(".", "#[allow(clippy::large_enum_variant)]")
        .type_attribute(
            "IpcServerInfo",
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
