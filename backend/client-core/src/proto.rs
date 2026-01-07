// IPC protocol layer (package: opencode)
// Contains: IpcClientMessage, IpcServerMessage, IpcServerInfo, etc.
include!(concat!(env!("OUT_DIR"), "/opencode.rs"));

// OpenCode canonical models (package: opencode.auth)
pub mod auth {
    include!(concat!(env!("OUT_DIR"), "/opencode.auth.rs"));
}

// OpenCode canonical models (package: opencode.model)
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/opencode.model.rs"));
}

// OpenCode canonical models (package: opencode.provider)
pub mod provider {
    include!(concat!(env!("OUT_DIR"), "/opencode.provider.rs"));
}

// OpenCode canonical models (package: opencode.session)
pub mod session {
    include!(concat!(env!("OUT_DIR"), "/opencode.session.rs"));
}

// OpenCode canonical models (package: opencode.agent)
pub mod agent {
    include!(concat!(env!("OUT_DIR"), "/opencode.agent.rs"));
}

// OpenCode canonical models (package: opencode.tool)
pub mod tool {
    include!(concat!(env!("OUT_DIR"), "/opencode.tool.rs"));
}

// OpenCode canonical models (package: opencode.message)
pub mod message {
    include!(concat!(env!("OUT_DIR"), "/opencode.message.rs"));

    // Nested package: opencode.message.part
    pub mod part {
        include!(concat!(env!("OUT_DIR"), "/opencode.message.part.rs"));
    }

    // Nested package: opencode.message.error
    pub mod error {
        include!(concat!(env!("OUT_DIR"), "/opencode.message.error.rs"));
    }
}

// OpenCode canonical models (package: opencode.event)
pub mod event {
    include!(concat!(env!("OUT_DIR"), "/opencode.event.rs"));
}
