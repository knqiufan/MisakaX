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
    pub fn create(
        config: &RouterConfig,
        decrypted_key: &str,
    ) -> Result<Box<dyn LlmProvider>> {
        match config.provider.as_str() {
            "openai" => {
                let provider =
                    OpenAiProvider::new(decrypted_key, config.base_url.as_deref())?;
                Ok(Box::new(provider))
            }
            "anthropic" => {
                let provider =
                    AnthropicProvider::new(decrypted_key, config.base_url.as_deref())?;
                Ok(Box::new(provider))
            }
            "google" => {
                let provider =
                    GeminiProvider::new(decrypted_key, config.base_url.as_deref())?;
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
        let compat = config
            .api_compat
            .as_deref()
            .unwrap_or("openai");

        let base_url = config
            .base_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!(
                "Custom provider '{}' requires a base_url",
                config.provider
            ))?;

        match compat {
            "openai" => {
                let provider = OpenAiCompatProvider::new(
                    decrypted_key,
                    base_url,
                    &config.provider,
                )?;
                Ok(Box::new(provider))
            }
            "anthropic" => {
                let provider = AnthropicProvider::new(
                    decrypted_key,
                    Some(base_url),
                )?;
                Ok(Box::new(provider))
            }
            other => Err(anyhow::anyhow!(
                "Unsupported api_compat: '{}'. Expected 'openai' or 'anthropic'",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(provider: &str, base_url: Option<&str>, api_compat: Option<&str>) -> RouterConfig {
        RouterConfig {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            provider: provider.to_string(),
            api_key_encrypted: None,
            model: Some("test-model".to_string()),
            base_url: base_url.map(String::from),
            config_json: None,
            is_active: true,
            created_at: "2025-01-01".to_string(),
            api_compat: api_compat.map(String::from),
        }
    }

    #[test]
    fn test_create_openai_provider() {
        let config = make_config("openai", None, None);
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678")
            .expect("Should create OpenAI provider");
        assert_eq!(provider.provider_type(), "openai");
    }

    #[test]
    fn test_create_openai_with_custom_base_url() {
        let config = make_config("openai", Some("https://my-proxy.com/v1"), None);
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678")
            .expect("Should create OpenAI provider with custom base URL");
        assert_eq!(provider.provider_type(), "openai");
    }

    #[test]
    fn test_create_anthropic_provider() {
        let config = make_config("anthropic", None, None);
        let provider = ProviderFactory::create(&config, "sk-ant-test-key-12345678")
            .expect("Should create Anthropic provider");
        assert_eq!(provider.provider_type(), "anthropic");
    }

    #[test]
    fn test_create_anthropic_with_base_url() {
        let config = make_config("anthropic", Some("https://anthropic-proxy.com"), None);
        let provider = ProviderFactory::create(&config, "sk-ant-test-key-12345678")
            .expect("Should create Anthropic provider with custom base URL");
        assert_eq!(provider.provider_type(), "anthropic");
    }

    #[test]
    fn test_create_gemini_provider() {
        let config = make_config("google", None, None);
        let provider = ProviderFactory::create(&config, "AIza-test-key-12345678")
            .expect("Should create Gemini provider");
        assert_eq!(provider.provider_type(), "google");
    }

    #[test]
    fn test_create_custom_provider_openai_compat() {
        let config = make_config("deepseek", Some("https://api.deepseek.com"), Some("openai"));
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678")
            .expect("Should create OpenAI-compat provider for DeepSeek");
        assert_eq!(provider.provider_type(), "deepseek");
    }

    #[test]
    fn test_create_custom_provider_default_compat() {
        let config = make_config("my-custom", Some("https://my-api.com/v1"), None);
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678")
            .expect("Should create provider with default OpenAI compat");
        assert_eq!(provider.provider_type(), "my-custom");
    }

    #[test]
    fn test_custom_provider_without_base_url_fails() {
        let config = make_config("deepseek", None, Some("openai"));
        let result = ProviderFactory::create(&config, "sk-test-key-fake-12345678");
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("requires a base_url"), "Error: {}", err_msg);
    }

    #[test]
    fn test_unsupported_compat_fails() {
        let config = make_config("custom", Some("https://api.example.com"), Some("unknown"));
        let result = ProviderFactory::create(&config, "sk-test-key-fake-12345678");
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Unsupported api_compat"), "Error: {}", err_msg);
    }

    #[test]
    fn test_provider_supports_vision() {
        let config = make_config("openai", None, None);
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678").unwrap();
        assert!(provider.supports_vision("gpt-4o"));
        assert!(provider.supports_vision("gpt-4.1-mini"));
        assert!(!provider.supports_vision("some-random-model"));
    }

    #[test]
    fn test_provider_supports_thinking() {
        let config = make_config("anthropic", None, None);
        let provider = ProviderFactory::create(&config, "sk-ant-test-key-12345678").unwrap();
        assert!(provider.supports_thinking("claude-sonnet-4-20250514"));
        assert!(provider.supports_thinking("claude-opus-4-20250514"));
        assert!(!provider.supports_thinking("claude-haiku-3-5-20241022"));
    }

    #[test]
    fn test_build_agent_handle() {
        use crate::services::llm::config::LlmConfig;
        let config = make_config("openai", None, None);
        let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678").unwrap();
        let llm_config = LlmConfig::default();
        let agent = provider.build_agent("gpt-4o", Some("You are a helpful assistant."), &llm_config);
        assert!(agent.is_ok());
    }
}
