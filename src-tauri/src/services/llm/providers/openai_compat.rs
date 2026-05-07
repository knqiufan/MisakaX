use anyhow::Result;
use rig::client::CompletionClient;

use crate::services::llm::config::LlmConfig;
use crate::services::llm::traits::{AgentHandle, LlmProvider};

/// OpenAI 兼容 Provider — 处理所有使用 OpenAI Chat Completions API 协议的第三方服务
///
/// 包括 DeepSeek、自建中转、第三方 API 代理等。
/// 用户在设置中选择 "接口兼容 = OpenAI" 时走此实现。
pub struct OpenAiCompatProvider {
    client: rig::providers::openai::CompletionsClient,
    provider_name: String,
    vision_models: Vec<String>,
    thinking_models: Vec<String>,
}

impl OpenAiCompatProvider {
    pub fn new(api_key: &str, base_url: &str, name: &str) -> Result<Self> {
        let client = rig::providers::openai::CompletionsClient::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build OpenAI-compat client: {e}"))?;
        Ok(Self {
            client,
            provider_name: name.to_string(),
            vision_models: Vec::new(),
            thinking_models: Vec::new(),
        })
    }

    /// 设置已知支持视觉的模型列表（从 custom_models 表读取时使用）
    pub fn with_vision_models(mut self, models: Vec<String>) -> Self {
        self.vision_models = models;
        self
    }

    /// 设置已知支持思维链的模型列表
    pub fn with_thinking_models(mut self, models: Vec<String>) -> Self {
        self.thinking_models = models;
        self
    }
}

impl LlmProvider for OpenAiCompatProvider {
    fn provider_type(&self) -> &str {
        &self.provider_name
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
        self.vision_models
            .iter()
            .any(|m| m.eq_ignore_ascii_case(model_name))
    }

    fn supports_thinking(&self, model_name: &str) -> bool {
        self.thinking_models
            .iter()
            .any(|m| m.eq_ignore_ascii_case(model_name))
    }
}
