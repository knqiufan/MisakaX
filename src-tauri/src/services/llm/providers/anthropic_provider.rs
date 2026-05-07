use anyhow::Result;
use rig::client::CompletionClient;

use crate::services::llm::config::LlmConfig;
use crate::services::llm::traits::{AgentHandle, LlmProvider};

/// Anthropic Provider（Claude 系列模型）
pub struct AnthropicProvider {
    client: rig::providers::anthropic::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Result<Self> {
        let client = if let Some(url) = base_url {
            rig::providers::anthropic::Client::builder()
                .api_key(api_key)
                .base_url(url)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build Anthropic client: {e}"))?
        } else {
            rig::providers::anthropic::Client::new(api_key)
                .map_err(|e| anyhow::anyhow!("Failed to build Anthropic client: {e}"))?
        };
        Ok(Self { client })
    }
}

impl LlmProvider for AnthropicProvider {
    fn provider_type(&self) -> &str {
        "anthropic"
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
        Ok(AgentHandle::Anthropic(builder.build()))
    }

    fn supports_vision(&self, _model_name: &str) -> bool {
        true
    }

    fn supports_thinking(&self, model_name: &str) -> bool {
        let name = model_name.to_lowercase();
        name.contains("sonnet-4")
            || name.contains("opus-4")
            || name.contains("sonnet-5")
            || name.contains("opus-5")
    }
}
