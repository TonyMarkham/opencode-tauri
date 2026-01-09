use crate::error::opencode_client::OpencodeClientError;
use crate::field_normalizer::normalize_json;
use crate::proto::session::OcSessionInfo;

use common::ErrorLocation;

use std::panic::Location;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use url::Url;

const DEFAULT_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
const OPENCODE_DIRECTORY_HEADER_KEY: &str = "x-opencode-directory";
const OPENCODE_SERVER_SESSION_ENDPOINT: &str = "session";

#[derive(Clone)]
pub struct OpencodeClient {
    base_url: Url,
    client: Client,
    pub directory: Option<String>,
}

impl OpencodeClient {
    pub fn new(base_url_str: &str) -> Result<Self, OpencodeClientError> {
        let base_url = Url::parse(base_url_str)?;
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT_DURATION)
            .build()?;

        Ok(Self {
            base_url,
            client,
            directory: None,
        })
    }

    fn prepare_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut request = request;
        if let Some(dir) = &self.directory {
            request = request.header(OPENCODE_DIRECTORY_HEADER_KEY, dir);
        }
        request
    }

    pub async fn list_sessions(&self) -> Result<Vec<OcSessionInfo>, OpencodeClientError> {
        let url = self.base_url.join(OPENCODE_SERVER_SESSION_ENDPOINT)?;

        let response = self.prepare_request(self.client.get(url)).send().await?;

        if !response.status().is_success() {
            return Err(OpencodeClientError::Server {
                message: format!(
                    "HTTP {} - {}",
                    response.status().as_u16(),
                    response.text().await.unwrap_or_default()
                ),
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let json: Value = response.json().await?;
        let normalized = normalize_json(json);
        let sessions: Vec<OcSessionInfo> = serde_json::from_value(normalized)?;

        Ok(sessions)
    }

    pub async fn create_session(
        &self,
        title: Option<&str>,
    ) -> Result<OcSessionInfo, OpencodeClientError> {
        let url = self.base_url.join(OPENCODE_SERVER_SESSION_ENDPOINT)?;

        let body = match title {
            Some(t) => serde_json::json!({"title": t}),
            None => serde_json::json!({}),
        };

        let response = self
            .prepare_request(self.client.post(url))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OpencodeClientError::Server {
                message: format!(
                    "HTTP {} - {}",
                    response.status().as_u16(),
                    response.text().await.unwrap_or_default(),
                ),
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let json: Value = response.json().await?;
        let normalized = normalize_json(json);
        let session: OcSessionInfo = serde_json::from_value(normalized)?;

        Ok(session)
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<bool, OpencodeClientError> {
        let url = self
            .base_url
            .join(&format!("{OPENCODE_SERVER_SESSION_ENDPOINT}/{session_id}"))?;

        let response = self.prepare_request(self.client.delete(url)).send().await?;

        Ok(response.status().is_success())
    }

    /// Sync an API key for a provider to the OpenCode server.
    ///
    /// # Arguments
    /// * `provider` - Provider ID (e.g., "openai")
    /// * `api_key` - The API key value
    ///
    /// # Errors
    /// Returns [`OpencodeClientError`] if the HTTP request fails or server rejects the key.
    pub async fn sync_api_key(
        &self,
        provider: &str,
        api_key: &str,
    ) -> Result<(), OpencodeClientError> {
        let url = self.base_url.join(&format!("auth/{}", provider))?;

        let body = serde_json::json!({
            "type": "api",
            "key": api_key
        });

        let response = self
            .prepare_request(self.client.put(url))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OpencodeClientError::Server {
                message: format!(
                    "HTTP {} - {}",
                    response.status().as_u16(),
                    response.text().await.unwrap_or_default()
                ),
                location: ErrorLocation::from(Location::caller()),
            });
        }

        Ok(())
    }
}
