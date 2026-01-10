use crate::error::opencode_client::OpencodeClientError;
use crate::field_normalizer::normalize_json;
use crate::proto::message::OcMessage;
use crate::proto::session::OcSessionInfo;

use common::ErrorLocation;

use std::panic::Location;
use std::time::Duration;

use log::{debug, info};
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

    /// Sends a message to an AI session and returns the assistant's response.
    ///
    /// This is a blocking call that waits for the complete AI response.
    /// For streaming, use SSE subscription (Session 15-16).
    pub async fn send_message(
        &self,
        session_id: &str,
        text: &str,
        model_id: &str,
        provider_id: &str,
        agent: Option<&str>,
    ) -> Result<OcMessage, OpencodeClientError> {
        let url = self.base_url.join(&format!(
            "{OPENCODE_SERVER_SESSION_ENDPOINT}/{session_id}/message"
        ))?;

        info!(
            "Sending message to session {} with model {}/{}",
            session_id, provider_id, model_id
        );

        // Build request body with camelCase field names (OpenCode server format)
        let body = serde_json::json!({
            "model": {
                "modelID": model_id,
                "providerID": provider_id
            },
            "parts": [{
                "type": "text",
                "text": text
            }],
            "agent": agent.unwrap_or("build")
        });

        debug!("Sending message to session {session_id}: {body:?}");

        let response = self
            .prepare_request(self.client.post(url))
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(OpencodeClientError::Server {
                message: format!("HTTP {} - {}", status.as_u16(), error_body),
                location: ErrorLocation::from(Location::caller()),
            });
        }

        let json: Value = response.json().await?;
        let mut normalized = normalize_json(json);

        // The response is { "info": {...}, "parts": [...] }
        // Parts come as flat objects with "type" discriminator, but proto expects
        // them wrapped as {"text": {...}} or {"tool": {...}} etc.
        let raw_parts = normalized
            .get("parts")
            .cloned()
            .unwrap_or(Value::Array(vec![]));

        // Transform parts from flat format to tagged format for proto oneOf
        let transformed_parts = if let Value::Array(parts_arr) = raw_parts {
            let wrapped: Vec<Value> = parts_arr
                .into_iter()
                .filter_map(|part| {
                    if let Value::Object(ref obj) = part {
                        // Get the "type" field to determine the variant
                        if let Some(Value::String(type_name)) = obj.get("type") {
                            // Convert kebab-case to snake_case for proto field names
                            let proto_field_name = type_name.replace('-', "_");
                            // Wrap the part object with its type as the key
                            let mut wrapper = serde_json::Map::new();
                            wrapper.insert(proto_field_name, part);
                            return Some(Value::Object(wrapper));
                        }
                    }
                    None
                })
                .collect();
            Value::Array(wrapped)
        } else {
            Value::Array(vec![])
        };

        let info_value = normalized
            .get_mut("info")
            .ok_or_else(|| OpencodeClientError::Server {
                message: "Response missing 'info' field".to_string(),
                location: ErrorLocation::from(Location::caller()),
            })?;

        debug!(
            "Transformed parts JSON: {}",
            serde_json::to_string_pretty(&transformed_parts).unwrap_or_default()
        );

        // Inject transformed parts into the info object
        if let Value::Object(info_map) = info_value {
            info_map.insert("parts".to_string(), transformed_parts);
        }

        let assistant: crate::proto::message::OcAssistantMessage =
            serde_json::from_value(info_value.clone()).map_err(|e| {
                OpencodeClientError::Server {
                    message: format!("Failed to parse assistant message: {e}"),
                    location: ErrorLocation::from(Location::caller()),
                }
            })?;

        info!(
            "Received response: {} tokens in, {} tokens out",
            assistant.tokens.as_ref().map(|t| t.input).unwrap_or(0),
            assistant.tokens.as_ref().map(|t| t.output).unwrap_or(0)
        );

        debug!("Assistant message received for session {session_id}: {assistant:?}");

        Ok(OcMessage {
            message: Some(crate::proto::message::oc_message::Message::Assistant(
                assistant,
            )),
        })
    }
}
