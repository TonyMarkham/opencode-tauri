#[derive(Clone)]
pub struct IpcConfig {
    port: u16,
    auth_token: String,
}

impl IpcConfig {
    pub fn new(port: u16, auth_token: String) -> Self {
        Self { port, auth_token }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }
}
