use std::pin::Pin;

use anyhow::Result;
use futures::Stream;
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

// ─── 类型擦除的流式 delta ─────────────────────────────────────────────

/// Provider 无关的流式 delta 类型
///
/// 将各 Provider 不同泛型参数的 `StreamedAssistantContent<R>` 映射为
/// 统一的类型擦除枚举，消除上层代码对具体 Provider 类型的依赖。
#[derive(Debug, Clone)]
pub enum StreamDelta {
    /// 文本增量
    Text(String),
    /// 思维链/推理增量（Anthropic thinking blocks / OpenAI reasoning）
    Thinking(String),
    /// 完整的思维链块（一次性接收，通常来自 Reasoning variant）
    ThinkingBlock(String),
    /// Token 用量统计（流结束时发送）
    Usage(StreamUsage),
}

/// 类型擦除的 token 用量
#[derive(Debug, Clone)]
pub struct StreamUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// 类型擦除的流式输出流
pub type DeltaStream =
    Pin<Box<dyn Stream<Item = Result<StreamDelta, anyhow::Error>> + Send>>;

/// 将 rig-core 的 `MultiTurnStreamItem<R>` 转换为 `Option<StreamDelta>`
///
/// 返回 `None` 表示该 item 不需要向前端推送（如 UserContent 工具结果回传）。
fn map_multi_turn_item<R>(item: rig::agent::MultiTurnStreamItem<R>) -> Option<StreamDelta>
where
    R: Clone + std::marker::Unpin + rig::completion::request::GetTokenUsage,
{
    use rig::agent::MultiTurnStreamItem;
    use rig::streaming::StreamedAssistantContent;

    match item {
        MultiTurnStreamItem::StreamAssistantItem(content) => match content {
            StreamedAssistantContent::Text(text) => {
                if text.text.is_empty() {
                    None
                } else {
                    Some(StreamDelta::Text(text.text))
                }
            }
            StreamedAssistantContent::Reasoning(reasoning) => {
                let display = reasoning.display_text();
                if display.is_empty() {
                    None
                } else {
                    Some(StreamDelta::ThinkingBlock(display))
                }
            }
            StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                if reasoning.is_empty() {
                    None
                } else {
                    Some(StreamDelta::Thinking(reasoning))
                }
            }
            StreamedAssistantContent::Final(response) => {
                if let Some(usage) = response.token_usage() {
                    Some(StreamDelta::Usage(StreamUsage {
                        input_tokens: usage.input_tokens,
                        output_tokens: usage.output_tokens,
                        total_tokens: usage.total_tokens,
                    }))
                } else {
                    None
                }
            }
            _ => None,
        },
        MultiTurnStreamItem::FinalResponse(_) => None,
        MultiTurnStreamItem::StreamUserItem(_) => None,
        _ => None,
    }
}

// ─── AgentHandle ──────────────────────────────────────────────────────

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
/// `prompt`/`chat`/`stream_chat` 的 match 分支。由于 Provider 种类有限（通常 3-5 个），
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

    /// 流式多轮对话 — 返回类型擦除的 `DeltaStream`
    ///
    /// `prompt` 支持多模态输入（文本 + 图片），接受 `rig::completion::message::Message`。
    /// 内部通过 `StreamingChat::stream_chat` 获取 Provider 特定类型的流，
    /// 然后用 `map_multi_turn_item` 将每个 `MultiTurnStreamItem` 统一为 `StreamDelta`。
    pub async fn stream_chat(
        &self,
        prompt: rig::completion::message::Message,
        history: Vec<rig::completion::message::Message>,
    ) -> Result<DeltaStream> {
        use futures::StreamExt;
        use rig::streaming::StreamingChat;

        match self {
            Self::OpenAi(agent) => {
                let stream = agent.stream_chat(prompt, history).await;
                Ok(Box::pin(stream.filter_map(|item| async move {
                    match item {
                        Ok(multi) => map_multi_turn_item(multi).map(Ok),
                        Err(e) => Some(Err(anyhow::anyhow!("{e}"))),
                    }
                })))
            }
            Self::Anthropic(agent) => {
                let stream = agent.stream_chat(prompt, history).await;
                Ok(Box::pin(stream.filter_map(|item| async move {
                    match item {
                        Ok(multi) => map_multi_turn_item(multi).map(Ok),
                        Err(e) => Some(Err(anyhow::anyhow!("{e}"))),
                    }
                })))
            }
            Self::Gemini(agent) => {
                let stream = agent.stream_chat(prompt, history).await;
                Ok(Box::pin(stream.filter_map(|item| async move {
                    match item {
                        Ok(multi) => map_multi_turn_item(multi).map(Ok),
                        Err(e) => Some(Err(anyhow::anyhow!("{e}"))),
                    }
                })))
            }
        }
    }
}
