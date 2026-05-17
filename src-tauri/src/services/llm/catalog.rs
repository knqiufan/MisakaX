#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderApi {
    OpenAi,
    Anthropic,
    Google,
}

impl ProviderApi {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "openai" => Some(Self::OpenAi),
            "anthropic" => Some(Self::Anthropic),
            "google" => Some(Self::Google),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorEndpoint {
    pub base_url: &'static str,
    pub models_path: Option<&'static str>,
}

pub fn supported_vendors() -> &'static [&'static str] {
    &[
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
    ]
}

pub fn endpoint(api: ProviderApi, vendor: &str) -> Option<VendorEndpoint> {
    match (api, vendor) {
        (ProviderApi::OpenAi, "openai") => Some(openai("https://api.openai.com/v1")),
        (ProviderApi::Anthropic, "anthropic") => Some(anthropic("https://api.anthropic.com")),
        (ProviderApi::Google, "google") => {
            Some(google("https://generativelanguage.googleapis.com"))
        }
        (ProviderApi::OpenAi, "deepseek") => Some(openai("https://api.deepseek.com/v1")),
        (ProviderApi::Anthropic, "deepseek") => Some(no_list("https://api.deepseek.com/anthropic")),
        (ProviderApi::OpenAi, "zhipu") => Some(openai("https://open.bigmodel.cn/api/paas/v4")),
        (ProviderApi::Anthropic, "zhipu") => {
            Some(no_list("https://open.bigmodel.cn/api/anthropic"))
        }
        (ProviderApi::OpenAi, "minimax") => Some(openai("https://api.minimaxi.com/v1")),
        (ProviderApi::OpenAi, "stepfun") => Some(openai("https://api.stepfun.com/v1")),
        (ProviderApi::OpenAi, "moonshot") => Some(openai("https://api.moonshot.cn/v1")),
        (ProviderApi::Anthropic, "moonshot") => Some(no_list("https://api.moonshot.cn/anthropic")),
        (ProviderApi::OpenAi, "siliconflow") => Some(openai("https://api.siliconflow.cn/v1")),
        (_, "custom") => Some(no_list("")),
        _ => None,
    }
}

pub fn models_url(
    api: ProviderApi,
    vendor: &str,
    base_url_override: Option<&str>,
) -> Option<String> {
    let endpoint = endpoint(api, vendor)?;
    let models_path = endpoint.models_path?;
    let base_url = base_url_override
        .filter(|url| !url.trim().is_empty())
        .unwrap_or(endpoint.base_url);
    Some(join_url(base_url, models_path))
}

pub fn chat_url(api: ProviderApi, vendor: &str, base_url_override: Option<&str>) -> Option<String> {
    let endpoint = endpoint(api, vendor)?;
    let base_url = base_url_override
        .filter(|url| !url.trim().is_empty())
        .unwrap_or(endpoint.base_url);
    let path = match api {
        ProviderApi::OpenAi => "/chat/completions",
        ProviderApi::Anthropic => "/v1/messages",
        ProviderApi::Google => return None,
    };
    Some(join_url(base_url, path))
}

pub fn google_generate_url(base_url: Option<&str>, model_id: &str) -> String {
    let base = base_url
        .filter(|url| !url.trim().is_empty())
        .unwrap_or("https://generativelanguage.googleapis.com");
    let model = model_id.trim_start_matches("models/");
    format!(
        "{}/v1beta/models/{}:generateContent",
        base.trim_end_matches('/'),
        model
    )
}

fn openai(base_url: &'static str) -> VendorEndpoint {
    VendorEndpoint {
        base_url,
        models_path: Some("/models"),
    }
}

fn anthropic(base_url: &'static str) -> VendorEndpoint {
    VendorEndpoint {
        base_url,
        models_path: Some("/v1/models"),
    }
}

fn google(base_url: &'static str) -> VendorEndpoint {
    VendorEndpoint {
        base_url,
        models_path: Some("/v1beta/models"),
    }
}

fn no_list(base_url: &'static str) -> VendorEndpoint {
    VendorEndpoint {
        base_url,
        models_path: None,
    }
}

fn join_url(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}
