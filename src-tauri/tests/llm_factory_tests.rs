use misaka_x_lib::db::models::RouterConfig;
use misaka_x_lib::services::llm::ProviderFactory;

fn make_config(provider: &str, base_url: Option<&str>, api_compat: Option<&str>) -> RouterConfig {
    RouterConfig {
        id: "test-id".to_string(),
        name: "Test".to_string(),
        provider: provider.to_string(),
        vendor: None,
        api_key_encrypted: None,
        model: Some("test-model".to_string()),
        base_url: base_url.map(String::from),
        config_json: None,
        advanced_json: None,
        is_active: true,
        created_at: "2025-01-01".to_string(),
        api_compat: api_compat.map(String::from),
    }
}

fn make_vendor_config(provider: &str, vendor: &str, base_url: &str) -> RouterConfig {
    let mut config = make_config(provider, Some(base_url), None);
    config.vendor = Some(vendor.to_string());
    config
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
fn test_create_openai_provider_for_zhipu_vendor_uses_openai_api_style() {
    let config = make_vendor_config("openai", "zhipu", "https://open.bigmodel.cn/api/paas/v4");
    let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678")
        .expect("Should create OpenAI provider for Zhipu OpenAI-compatible endpoint");
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
    assert!(
        err_msg.contains("requires a base_url"),
        "Error: {}",
        err_msg
    );
}

#[test]
fn test_unsupported_compat_fails() {
    let config = make_config("custom", Some("https://api.example.com"), Some("unknown"));
    let result = ProviderFactory::create(&config, "sk-test-key-fake-12345678");
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Unsupported api_compat"),
        "Error: {}",
        err_msg
    );
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
    use misaka_x_lib::services::llm::LlmConfig;
    let config = make_config("openai", None, None);
    let provider = ProviderFactory::create(&config, "sk-test-key-fake-12345678").unwrap();
    let llm_config = LlmConfig::default();
    let agent = provider.build_agent("gpt-4o", Some("You are a helpful assistant."), &llm_config);
    assert!(agent.is_ok());
}
