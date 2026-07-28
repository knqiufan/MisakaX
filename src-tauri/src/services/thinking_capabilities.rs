//! Known-model thinking capabilities and native disable strategies.

use serde::{Deserialize, Serialize};

/// How a model can natively control thinking when the user toggles it off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeThinkingControl {
    /// DeepSeek-style `extra_body.thinking.type = enabled|disabled`.
    DeepseekThinkingType,
    /// Gemini 2.5 Flash / Flash-Lite `thinking_budget = 0`.
    GeminiThinkingBudget,
    /// Explicit Anthropic thinking disabled param for catalog-verified models.
    AnthropicThinkingDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThinkingCapability {
    pub supports_thinking: bool,
    pub native_control: Option<NativeThinkingControl>,
}

/// Look up capability by vendor + model id. Unknown models default to
/// "no reliable native disable" so a configured fallback is required.
pub fn lookup_thinking_capability(vendor: &str, model_id: &str) -> ThinkingCapability {
    let vendor = vendor.trim().to_ascii_lowercase();
    let model = model_id.trim().to_ascii_lowercase();

    if vendor == "deepseek" || model.contains("deepseek") {
        return ThinkingCapability {
            // The DeepSeek API accepts its native thinking switch for every
            // DeepSeek model, including model identifiers not in this catalog.
            supports_thinking: true,
            native_control: Some(NativeThinkingControl::DeepseekThinkingType),
        };
    }

    if is_gemini_flash_budget(&model) {
        return ThinkingCapability {
            supports_thinking: true,
            native_control: Some(NativeThinkingControl::GeminiThinkingBudget),
        };
    }

    if model.contains("gemini") {
        return ThinkingCapability {
            supports_thinking: true,
            native_control: None,
        };
    }

    if is_anthropic_native_disable(&model) {
        return ThinkingCapability {
            supports_thinking: true,
            native_control: Some(NativeThinkingControl::AnthropicThinkingDisabled),
        };
    }

    if model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.contains("gpt-5")
    {
        return ThinkingCapability {
            supports_thinking: true,
            native_control: None,
        };
    }

    ThinkingCapability {
        supports_thinking: false,
        native_control: None,
    }
}

fn is_gemini_flash_budget(model: &str) -> bool {
    (model.contains("gemini-2.5-flash") || model.contains("gemini-2.0-flash-lite"))
        && !model.contains("pro")
}

fn is_anthropic_native_disable(model: &str) -> bool {
    // Only catalog-verified Claude 3.x extended-thinking IDs; newer adaptive
    // models must use an explicit non-thinking alternate.
    model.contains("claude-3-7-sonnet") || model.contains("claude-3-5-sonnet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepseek_has_native_control() {
        let cap = lookup_thinking_capability("deepseek", "deepseek-reasoner");
        assert_eq!(
            cap.native_control,
            Some(NativeThinkingControl::DeepseekThinkingType)
        );
    }

    #[test]
    fn gemini_pro_requires_fallback() {
        let cap = lookup_thinking_capability("google", "gemini-2.5-pro");
        assert!(cap.supports_thinking);
        assert!(cap.native_control.is_none());
    }

    #[test]
    fn gemini_flash_has_budget() {
        let cap = lookup_thinking_capability("google", "gemini-2.5-flash");
        assert_eq!(
            cap.native_control,
            Some(NativeThinkingControl::GeminiThinkingBudget)
        );
    }
}
