use misaka_x_lib::services::llm::catalog::{endpoint, supported_vendors, ProviderApi};
use serde_json::Value;

#[test]
fn test_catalog_contains_required_vendors() {
    let vendors = supported_vendors();

    for vendor in [
        "openai",
        "anthropic",
        "google",
        "deepseek",
        "zhipu",
        "minimax",
        "stepfun",
        "moonshot",
        "siliconflow",
        "custom",
    ] {
        assert!(vendors.contains(&vendor), "missing vendor: {vendor}");
    }
}

#[test]
fn test_catalog_returns_known_endpoint() {
    let endpoint = endpoint(ProviderApi::OpenAi, "zhipu").expect("zhipu openai endpoint");

    assert_eq!(endpoint.base_url, "https://open.bigmodel.cn/api/paas/v4");
    assert_eq!(endpoint.models_path, Some("/models"));
}

#[test]
fn test_catalog_rejects_unsupported_combination() {
    assert!(endpoint(ProviderApi::Google, "deepseek").is_none());
}

#[test]
fn test_catalog_matches_shared_frontend_snapshot() {
    let snapshot: Value = serde_json::from_str(include_str!(
        "../../src/lib/providers/catalog.snapshot.json"
    ))
    .expect("valid catalog snapshot");

    for vendor in supported_vendors() {
        for api in [
            ProviderApi::OpenAi,
            ProviderApi::Anthropic,
            ProviderApi::Google,
        ] {
            let api_key = api_key(api);
            let expected = snapshot.get(vendor).and_then(|entry| entry.get(api_key));
            let actual = endpoint(api, vendor);

            match (actual, expected) {
                (Some(actual), Some(expected)) => {
                    assert_eq!(actual.base_url, expected["baseUrl"]);
                    assert_eq!(
                        actual.models_path,
                        expected.get("modelsPath").and_then(Value::as_str)
                    );
                }
                (None, None) => {}
                (actual, expected) => panic!(
                    "catalog mismatch for vendor={vendor} api={api_key}: actual={actual:?} expected={expected:?}"
                ),
            }
        }
    }
}

fn api_key(api: ProviderApi) -> &'static str {
    match api {
        ProviderApi::OpenAi => "openai",
        ProviderApi::Anthropic => "anthropic",
        ProviderApi::Google => "google",
    }
}
