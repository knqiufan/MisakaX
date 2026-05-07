use anyhow::Result;
use rig::client::CompletionClient;

use crate::services::llm::config::LlmConfig;
use crate::services::llm::traits::{AgentHandle, LlmProvider};

/// OpenAI 原生 Provider，使用 Chat Completions API
pub struct OpenAiProvider {
    client: rig::providers::openai::CompletionsClient,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Result<Self> {
        let mut builder = rig::providers::openai::CompletionsClient::builder()
            .api_key(api_key);
        if let Some(url) = base_url {
            builder = builder.base_url(url);
        }
        let client = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build OpenAI client: {e}"))?;
        Ok(Self { client })
    }
}

impl LlmProvider for OpenAiProvider {
    fn provider_type(&self) -> &str {
        "openai"
    }

    fn build_agent(
        &self,
        model_name: &str,
        system_prompt: Option<&str>,
        llm_config: &LlmConfig,
    ) -> Result<AgentHandle> {
        let mut builder = self.client.agent(model_name)
            .temperature(llm_config.temperature);
        if let Some(prompt) = system_prompt {
            builder = builder.preamble(prompt);
        }
        if let Some(max) = llm_config.max_tokens {
            builder = builder.max_tokens(max as u64);
        }
        Ok(AgentHandle::OpenAi(builder.build()))
    }

    fn supports_vision(&self, model_name: &str) -> bool {
        let name = model_name.to_lowercase();
        name.starts_with("gpt-4o")
            || name.starts_with("gpt-4.1")
            || name.starts_with("gpt-5")
            || name == "o1"
            || name.starts_with("o1-")
            || name == "o3"
            || (name.starts_with("o3-") && !name.starts_with("o3-mini"))
            || name == "o4-mini"
            || name.starts_with("o4-mini-")
    }

    fn supports_thinking(&self, model_name: &str) -> bool {
        let name = model_name.to_lowercase();
        name == "o1"
            || name.starts_with("o1-")
            || name == "o3"
            || name.starts_with("o3-")
            || name == "o4-mini"
            || name.starts_with("o4-mini-")
    }
}
