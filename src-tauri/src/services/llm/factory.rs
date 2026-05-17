use anyhow::Result;

use crate::db::models::RouterConfig;
use crate::services::llm::providers::anthropic_provider::AnthropicProvider;
use crate::services::llm::providers::gemini_provider::GeminiProvider;
use crate::services::llm::providers::openai_compat::OpenAiCompatProvider;
use crate::services::llm::providers::openai_provider::OpenAiProvider;
use crate::services::llm::traits::LlmProvider;

/// Provider 工厂 — 根据 RouterConfig 和解密后的 API Key 创建对应 Provider
///
/// 扩展方式：新增 Provider 时只需添加一个 match 分支。
/// 遵守开闭原则：上层代码依赖 `Box<dyn LlmProvider>` 而非具体实现。
pub struct ProviderFactory;

impl ProviderFactory {
    /// 根据 RouterConfig 创建 Provider 实例
    ///
    /// - 已知 Provider（openai/anthropic/google）直接匹配创建
    /// - 未知 Provider 根据 `api_compat` 字段选择兼容实现
    /// - 默认兼容模式为 OpenAI Chat Completions API
    pub fn create(config: &RouterConfig, decrypted_key: &str) -> Result<Box<dyn LlmProvider>> {
        match config.provider.as_str() {
            "openai" => {
                let provider = OpenAiProvider::new(decrypted_key, config.base_url.as_deref())?;
                Ok(Box::new(provider))
            }
            "anthropic" => {
                let provider = AnthropicProvider::new(decrypted_key, config.base_url.as_deref())?;
                Ok(Box::new(provider))
            }
            "google" => {
                let provider = GeminiProvider::new(decrypted_key, config.base_url.as_deref())?;
                Ok(Box::new(provider))
            }
            _ => Self::create_compat_provider(config, decrypted_key),
        }
    }

    /// 为自定义/未知 Provider 创建兼容层实例
    fn create_compat_provider(
        config: &RouterConfig,
        decrypted_key: &str,
    ) -> Result<Box<dyn LlmProvider>> {
        let compat = config.api_compat.as_deref().unwrap_or("openai");

        let base_url = config.base_url.as_deref().ok_or_else(|| {
            anyhow::anyhow!("Custom provider '{}' requires a base_url", config.provider)
        })?;

        match compat {
            "openai" => {
                let provider =
                    OpenAiCompatProvider::new(decrypted_key, base_url, &config.provider)?;
                Ok(Box::new(provider))
            }
            "anthropic" => {
                let provider = AnthropicProvider::new(decrypted_key, Some(base_url))?;
                Ok(Box::new(provider))
            }
            other => Err(anyhow::anyhow!(
                "Unsupported api_compat: '{}'. Expected 'openai' or 'anthropic'",
                other
            )),
        }
    }
}
