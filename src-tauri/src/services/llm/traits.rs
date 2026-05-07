use anyhow::Result;
use rig::agent::Agent;

use super::config::LlmConfig;

/// LLM Provider 核心抽象
///
/// 每个 Provider 实现此 trait 即可接入系统。
/// 新增 Provider 只需：
///   1. 在 `providers/` 下创建新文件
///   2. 实现 `LlmProvider` trait
///   3. 在 `ProviderFactory` 中注册一行 match 分支
///
/// 上层代码通过 `Box<dyn LlmProvider>` 持有 Provider 引用（开闭原则），
/// 而内部返回的 [`AgentHandle`] 使用 enum dispatch（技术限制，见其文档）。
pub trait LlmProvider: Send + Sync {
    /// 返回 Provider 类型标识（如 "openai"、"anthropic"、"google"）
    fn provider_type(&self) -> &str;

    /// 创建一个 Agent 句柄，用于后续的 completion 和 streaming 调用
    ///
    /// `llm_config` 控制 temperature / max_tokens 等运行时参数。
    fn build_agent(
        &self,
        model_name: &str,
        system_prompt: Option<&str>,
        llm_config: &LlmConfig,
    ) -> Result<AgentHandle>;

    /// 查询指定模型是否支持视觉（图像输入）
    fn supports_vision(&self, model_name: &str) -> bool;

    /// 查询指定模型是否支持思维链（thinking blocks）
    fn supports_thinking(&self, model_name: &str) -> bool;
}

/// Agent 句柄 — 封装不同 Provider 的具体 Agent 类型
///
/// ## 为什么用 enum dispatch 而不是 trait object？
///
/// rig-core 的 `Agent<M: CompletionModel>` 是泛型结构体，`CompletionModel` trait
/// 有关联类型 (`Response`) 且要求 `Clone`，无法直接用 `Box<dyn CompletionModel>`
/// 作为 trait object。因此在 `LlmProvider` 层仍使用 `Box<dyn LlmProvider>`
/// 实现开闭原则（新增 Provider 不影响上层），但 Agent 层面退化为 enum dispatch。
///
/// 这是一个**已知技术妥协**：新增 Provider 需在此 enum 添加 variant +
/// `prompt`/`chat` 的 match 分支。由于 Provider 种类有限（通常 3-5 个），
/// 且绝大多数自定义 Provider 复用 `OpenAi` variant（通过 OpenAI 兼容协议），
/// 实际维护成本可控。
pub enum AgentHandle {
    OpenAi(Agent<rig::providers::openai::completion::CompletionModel>),
    Anthropic(Agent<rig::providers::anthropic::completion::CompletionModel>),
    Gemini(Agent<rig::providers::gemini::completion::CompletionModel>),
}

impl AgentHandle {
    /// 非流式单次对话
    pub async fn prompt(&self, input: &str) -> Result<String> {
        use rig::completion::Prompt;
        match self {
            Self::OpenAi(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
            Self::Anthropic(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
            Self::Gemini(a) => a.prompt(input).await.map_err(|e| anyhow::anyhow!("{e}")),
        }
    }

    /// 非流式多轮对话
    pub async fn chat(
        &self,
        input: &str,
        history: Vec<rig::completion::message::Message>,
    ) -> Result<String> {
        use rig::completion::Chat;
        match self {
            Self::OpenAi(a) => a
                .chat(input, history)
                .await
                .map_err(|e| anyhow::anyhow!("{e}")),
            Self::Anthropic(a) => a
                .chat(input, history)
                .await
                .map_err(|e| anyhow::anyhow!("{e}")),
            Self::Gemini(a) => a
                .chat(input, history)
                .await
                .map_err(|e| anyhow::anyhow!("{e}")),
        }
    }
}
