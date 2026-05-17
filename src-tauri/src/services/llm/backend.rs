use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use rig::completion::message::{
    AssistantContent, ImageMediaType, Message as RigMessage, UserContent,
};
use rig::one_or_many::OneOrMany;
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
        images: &Option<Vec<ImageAttachment>>,
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
        let prompt = build_user_prompt(user_content, images);

        let stream_session = StreamSession::new(
            message_id.to_string(),
            session.id.clone(),
            app.clone(),
            abort_flag,
        );

        stream_session
            .execute_stream(&agent, prompt, chat_history)
            .await
    }
}

// ─── 消息构建辅助函数 ────────────────────────────────────────────────

/// 构建当前用户的 prompt（支持文本 + 图片多模态）
///
/// 当 `images` 非空时，构建包含文本和图片的多内容 User 消息；
/// 否则仅返回纯文本消息。
pub fn build_user_prompt(content: &str, images: &Option<Vec<ImageAttachment>>) -> RigMessage {
    let has_images = images.as_ref().is_some_and(|imgs| !imgs.is_empty());

    if !has_images {
        return RigMessage::user(content);
    }

    let imgs = images.as_ref().unwrap();
    let mut parts: Vec<UserContent> = Vec::with_capacity(1 + imgs.len());

    parts.push(UserContent::text(content));

    for img in imgs {
        let media_type = parse_image_media_type(&img.media_type);
        parts.push(UserContent::image_base64(&img.data, Some(media_type), None));
    }

    RigMessage::User {
        content: OneOrMany::many(parts).expect("parts is guaranteed non-empty"),
    }
}

/// 将 MIME type 字符串解析为 rig-core 的 ImageMediaType
fn parse_image_media_type(mime: &str) -> ImageMediaType {
    match mime.to_lowercase().as_str() {
        "image/png" => ImageMediaType::PNG,
        "image/gif" => ImageMediaType::GIF,
        "image/webp" => ImageMediaType::WEBP,
        "image/heic" => ImageMediaType::HEIC,
        "image/heif" => ImageMediaType::HEIF,
        "image/svg+xml" => ImageMediaType::SVG,
        _ => ImageMediaType::JPEG,
    }
}

/// 将 db Message 列表转换为 rig-core 的 chat history 格式
///
/// 历史消息中的图片附件也会被还原为多模态消息。
#[cfg_attr(feature = "test-private", allow(dead_code))]
pub fn build_rig_chat_history(messages: &[Message]) -> Vec<RigMessage> {
    messages
        .iter()
        .filter_map(|msg| match msg.role.as_str() {
            "user" => Some(build_history_user_message(msg)),
            "assistant" => Some(RigMessage::Assistant {
                id: None,
                content: OneOrMany::one(AssistantContent::text(msg.content.clone())),
            }),
            _ => None,
        })
        .collect()
}

/// 从历史 User 消息中还原多模态内容
fn build_history_user_message(msg: &Message) -> RigMessage {
    let images = msg
        .attachments
        .as_deref()
        .and_then(|json| serde_json::from_str::<Vec<ImageAttachment>>(json).ok())
        .filter(|imgs| !imgs.is_empty());

    build_user_prompt(&msg.content, &images)
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
