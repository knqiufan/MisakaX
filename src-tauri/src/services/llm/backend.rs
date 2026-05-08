use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tauri::AppHandle;

use crate::db::models::{Message, Session};
use crate::services::llm::config::LlmConfig;
use crate::services::llm::factory::ProviderFactory;
use crate::services::llm::streaming::{StreamResult, StreamSession};

use super::traits::LlmProvider;

/// 图片附件数据（Base64 编码）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageAttachment {
    pub data: String,
    pub media_type: String,
    pub file_name: Option<String>,
}

/// 对话后端策略 trait — Phase 4 兼容性的关键抽象
///
/// Phase 2: `RigBackend`（Rig 直调）
/// Phase 4: `SidecarBackend`（Sidecar 代理，仅需新增一个实现）
///
/// chat Command 通过此 trait 与后端交互，
/// Phase 4 切换时前端和 Command 层完全不变。
#[async_trait]
pub trait ChatBackend: Send + Sync {
    /// 发送消息并以流式方式返回响应
    ///
    /// - `app`: Tauri AppHandle，用于 emit Event
    /// - `session`: 当前会话信息
    /// - `messages`: 历史消息列表
    /// - `user_content`: 用户输入内容
    /// - `images`: 可选的图片附件
    /// - `model_id`: 模型标识符
    /// - `config_id`: router config ID（用于获取 Provider）
    /// - `abort_flag`: 中止标记（用户点击"停止生成"时设置为 true）
    /// - `message_id`: 消息 ID（用于 Event 标识）
    async fn send_and_stream(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        images: &Option<Vec<ImageAttachment>>,
        model_id: &str,
        abort_flag: Arc<AtomicBool>,
        message_id: &str,
    ) -> Result<StreamResult>;
}

/// Phase 2: Rig 直调实现（临时后端 stub）
///
/// 通过 `LlmProvider` trait 构建 `AgentHandle`，
/// 再由 `StreamSession` 驱动流式调用。
/// Phase 4 时此实现将被 `SidecarBackend` 替代。
pub struct RigBackend {
    provider: Box<dyn LlmProvider>,
    llm_config: LlmConfig,
}

impl RigBackend {
    pub fn new(provider: Box<dyn LlmProvider>, llm_config: LlmConfig) -> Self {
        Self {
            provider,
            llm_config,
        }
    }

    /// 从 RouterConfig + 解密后的 API Key 构建 RigBackend
    pub fn from_config(
        config: &crate::db::models::RouterConfig,
        decrypted_key: &str,
        llm_config: LlmConfig,
    ) -> Result<Self> {
        let provider = ProviderFactory::create(config, decrypted_key)?;
        Ok(Self::new(provider, llm_config))
    }
}

#[async_trait]
impl ChatBackend for RigBackend {
    async fn send_and_stream(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        _images: &Option<Vec<ImageAttachment>>,
        model_id: &str,
        abort_flag: Arc<AtomicBool>,
        message_id: &str,
    ) -> Result<StreamResult> {
        let agent = self.provider.build_agent(
            model_id,
            session.system_prompt.as_deref(),
            &self.llm_config,
        )?;

        let chat_history = build_rig_chat_history(messages);

        let stream_session = StreamSession::new(
            message_id.to_string(),
            session.id.clone(),
            app.clone(),
            abort_flag,
        );

        stream_session
            .execute_stream(&agent, user_content, chat_history)
            .await
    }
}

/// 将 db Message 列表转换为 rig-core 的 chat history 格式
#[cfg_attr(feature = "test-private", allow(dead_code))]
pub fn build_rig_chat_history(
    messages: &[Message],
) -> Vec<rig::completion::message::Message> {
    use rig::completion::message::{AssistantContent, Message as RigMessage, UserContent};
    use rig::one_or_many::OneOrMany;

    messages
        .iter()
        .filter_map(|msg| match msg.role.as_str() {
            "user" => Some(RigMessage::User {
                content: OneOrMany::one(UserContent::text(msg.content.clone())),
            }),
            "assistant" => Some(RigMessage::Assistant {
                id: None,
                content: OneOrMany::one(AssistantContent::text(msg.content.clone())),
            }),
            _ => None,
        })
        .collect()
}

// Phase 4 预留接口（当前注释，Phase 4 时解注释并实现）
// pub struct SidecarBackend {
//     sidecar_url: String,
// }
//
// #[async_trait]
// impl ChatBackend for SidecarBackend {
//     async fn send_and_stream(
//         &self,
//         app: &AppHandle,
//         session: &Session,
//         messages: &[Message],
//         user_content: &str,
//         images: &Option<Vec<ImageAttachment>>,
//         model_id: &str,
//         abort_flag: Arc<AtomicBool>,
//         message_id: &str,
//     ) -> Result<StreamResult> {
//         // POST to Sidecar + SSE parsing
//         todo!("Phase 4: Implement SidecarBackend")
//     }
// }
