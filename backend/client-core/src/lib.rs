pub mod discovery;
pub mod error;
pub mod field_normalizer;
pub mod ipc;
pub mod proto;

mod opencode_client;
#[cfg(test)]
mod tests;

pub const OPENCODE_BINARY: &str = "opencode";
pub const OPENCODE_SERVER_HOSTNAME: &str = "127.0.0.1";
pub const OPENCODE_SERVER_BASE_URL: &str =
    const_format::concatcp!("http://", OPENCODE_SERVER_HOSTNAME);
