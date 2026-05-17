use anyhow::Result;
use rig::client::CompletionClient;

use crate::services::llm::config::LlmConfig;
use crate::services::llm::traits::{AgentHandle, LlmProvider};

/// Google Gemini Provider
///
/// 注意：rig-core 的 Gemini Client 暂不支持自定义 base_url，
/// 如需代理请使用 OpenAI 兼容模式（api_compat = "openai"）。
pub struct GeminiProvider {
    client: rig::providers::gemini::Client,
}

impl GeminiProvider {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Result<Self> {
        if base_url.is_some() {
            tracing::warn!(
                "Gemini provider does not support custom base_url in current rig-core version; \
                 the configured base_url will be ignored. \
                 Consider using api_compat='openai' with a compatible proxy instead."
            );
        }
        let client = rig::providers::gemini::Client::new(api_key)
            .map_err(|e| anyhow::anyhow!("Failed to build Gemini client: {e}"))?;
        Ok(Self { client })
    }
}

impl LlmProvider for GeminiProvider {
    fn provider_type(&self) -> &str {
        "google"
    }

    fn build_agent(
        &self,
        model_name: &str,
        system_prompt: Option<&str>,
        llm_config: &LlmConfig,
    ) -> Result<AgentHandle> {
        let mut builder = self
            .client
            .agent(model_name)
            .temperature(llm_config.temperature);
        if let Some(prompt) = system_prompt {
            builder = builder.preamble(prompt);
        }
        if let Some(max) = llm_config.max_tokens {
            builder = builder.max_tokens(max as u64);
        }
        Ok(AgentHandle::Gemini(builder.build()))
    }

    fn supports_vision(&self, _model_name: &str) -> bool {
        true
    }

    fn supports_thinking(&self, model_name: &str) -> bool {
        let name = model_name.to_lowercase();
        name.contains("2.5-pro") || name.contains("2.5-flash")
    }
}
