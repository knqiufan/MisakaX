use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// --------------------------------------------------------------------------- //
//  Request / Response types (aligned with Python agent/app/models.py)
// --------------------------------------------------------------------------- //

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
}

fn default_temperature() -> f64 {
    0.7
}

impl Default for AgentChatConfig {
    fn default() -> Self {
        Self {
            model: None,
            temperature: 0.7,
            max_tokens: None,
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRequest {
    pub messages: Vec<AgentChatMessage>,
    #[serde(default)]
    pub config: AgentChatConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTokenUsage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatResponse {
    pub message: AgentChatMessage,
    #[serde(default)]
    pub tool_calls: Vec<AgentToolCall>,
    #[serde(default)]
    pub usage: Option<AgentTokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub uptime_seconds: f64,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub agent_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub python_version: String,
    #[serde(default)]
    pub langgraph_available: bool,
    #[serde(default)]
    pub powermem_available: bool,
}

/// SSE event from /agent/stream (Phase 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamEvent {
    pub event: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

// --------------------------------------------------------------------------- //
//  SidecarClient
// --------------------------------------------------------------------------- //

pub struct SidecarClient {
    base_url: String,
    client: Client,
}

impl SidecarClient {
    pub fn new(port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            client,
        }
    }

    pub async fn health(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Health request failed: {}", e))?
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("Health response parse failed: {}", e))
    }

    pub async fn info(&self) -> Result<InfoResponse, String> {
        let url = format!("{}/info", self.base_url);
        self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Info request failed: {}", e))?
            .json::<InfoResponse>()
            .await
            .map_err(|e| format!("Info response parse failed: {}", e))
    }

    /// Phase 4: Send a chat request to the agent.
    pub async fn chat(&self, request: &AgentChatRequest) -> Result<AgentChatResponse, String> {
        let url = format!("{}/agent/chat", self.base_url);
        self.client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Chat request failed: {}", e))?
            .json::<AgentChatResponse>()
            .await
            .map_err(|e| format!("Chat response parse failed: {}", e))
    }

    /// Phase 4: Start a streaming chat request, returning the raw response
    /// for SSE parsing.
    pub async fn stream(&self, request: &AgentChatRequest) -> Result<reqwest::Response, String> {
        let url = format!("{}/agent/stream", self.base_url);
        self.client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Stream request failed: {}", e))
    }
}
