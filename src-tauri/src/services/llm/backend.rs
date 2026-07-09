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

use super::traits::{AgentHandle, LlmProvider};

/// 图片附件数据（Base64 编码）。保留独立结构以兼容既有测试与旧消息 JSON。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageAttachment {
    pub data: String,
    pub media_type: String,
    pub file_name: Option<String>,
}

/// 通用消息附件。当前支持图片与已提取文本；PDF/Office 文档先在前端占位。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MessageAttachment {
    Image {
        data: String,
        media_type: String,
        file_name: Option<String>,
    },
    Text {
        extracted_text: String,
        mime: String,
        file_name: String,
        size: u64,
    },
}

impl From<ImageAttachment> for MessageAttachment {
    fn from(image: ImageAttachment) -> Self {
        Self::Image {
            data: image.data,
            media_type: image.media_type,
            file_name: image.file_name,
        }
    }
}

impl<'de> serde::Deserialize<'de> for MessageAttachment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.get("kind").and_then(|kind| kind.as_str()) {
            Some("image") => deserialize_image_attachment(value).map_err(serde::de::Error::custom),
            Some("text") => deserialize_text_attachment(value).map_err(serde::de::Error::custom),
            None if value.get("data").is_some() => {
                deserialize_image_attachment(value).map_err(serde::de::Error::custom)
            }
            other => Err(serde::de::Error::custom(format!(
                "unknown attachment kind: {other:?}"
            ))),
        }
    }
}

fn deserialize_image_attachment(
    value: serde_json::Value,
) -> std::result::Result<MessageAttachment, String> {
    Ok(MessageAttachment::Image {
        data: required_string(&value, "data")?,
        media_type: required_string(&value, "media_type")?,
        file_name: value
            .get("file_name")
            .and_then(|name| name.as_str())
            .map(ToString::to_string),
    })
}

fn deserialize_text_attachment(
    value: serde_json::Value,
) -> std::result::Result<MessageAttachment, String> {
    Ok(MessageAttachment::Text {
        extracted_text: required_string(&value, "extracted_text")?,
        mime: required_string(&value, "mime")?,
        file_name: required_string(&value, "file_name")?,
        size: value
            .get("size")
            .and_then(|size| size.as_u64())
            .unwrap_or(0),
    })
}

fn required_string(value: &serde_json::Value, key: &str) -> std::result::Result<String, String> {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing attachment field: {key}"))
}

/// 对话后端策略 trait — Phase 4 兼容性的关键抽象
///
/// Phase 2: `RigBackend`（Rig 直调）
/// Phase 4: `SidecarBackend`（Sidecar 代理，仅需新增一个实现）
///
/// chat Command 通过此 trait 与后端交互，
/// Phase 4 切换时前端和 Command 层完全不变。
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ChatBackend: Send + Sync {
    /// 发送消息并以流式方式返回响应
    async fn send_and_stream(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        attachments: &Option<Vec<MessageAttachment>>,
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

    /// 构建单轮流式所需的 agent / prompt / history / session（send_and_stream 与 stream_round 共用）
    #[allow(clippy::too_many_arguments)]
    fn build_round(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        attachments: &Option<Vec<MessageAttachment>>,
        model_id: &str,
        abort_flag: Arc<AtomicBool>,
        message_id: &str,
    ) -> Result<(AgentHandle, RigMessage, Vec<RigMessage>, StreamSession)> {
        let agent =
            self.provider
                .build_agent(model_id, session.system_prompt.as_deref(), &self.llm_config)?;
        let chat_history = build_rig_chat_history(messages);
        let prompt = build_user_prompt(user_content, attachments);
        let stream_session = StreamSession::new(
            message_id.to_string(),
            session.id.clone(),
            app.clone(),
            abort_flag,
        );
        Ok((agent, prompt, chat_history, stream_session))
    }

    /// 流式执行一轮对话但**不 emit** 终结的 `stream_complete`
    ///
    /// 供 MCP 工具循环逐轮调用；由循环在结束后统一 emit 一次完成事件。
    #[allow(clippy::too_many_arguments)]
    pub async fn stream_round(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        attachments: &Option<Vec<MessageAttachment>>,
        model_id: &str,
        abort_flag: Arc<AtomicBool>,
        message_id: &str,
    ) -> Result<StreamResult> {
        let (agent, prompt, chat_history, stream_session) = self.build_round(
            app,
            session,
            messages,
            user_content,
            attachments,
            model_id,
            abort_flag,
            message_id,
        )?;
        stream_session
            .execute_stream_collect(&agent, prompt, chat_history)
            .await
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
        attachments: &Option<Vec<MessageAttachment>>,
        model_id: &str,
        abort_flag: Arc<AtomicBool>,
        message_id: &str,
    ) -> Result<StreamResult> {
        let (agent, prompt, chat_history, stream_session) = self.build_round(
            app,
            session,
            messages,
            user_content,
            attachments,
            model_id,
            abort_flag,
            message_id,
        )?;

        stream_session
            .execute_stream(&agent, prompt, chat_history)
            .await
    }
}

// ─── 消息构建辅助函数 ────────────────────────────────────────────────

/// 构建当前用户的 prompt（支持文本附件 + 图片多模态）
///
/// 当 `images` 非空时，构建包含文本和图片的多内容 User 消息；
/// 否则仅返回纯文本消息。
pub fn build_user_prompt(
    content: &str,
    attachments: &Option<Vec<MessageAttachment>>,
) -> RigMessage {
    let prompt_text = build_prompt_text(content, attachments);
    let images = collect_image_attachments(attachments);

    if images.is_empty() {
        return RigMessage::user(prompt_text);
    }

    let mut parts: Vec<UserContent> = Vec::with_capacity(1 + images.len());

    parts.push(UserContent::text(prompt_text));

    for (data, mime) in images {
        let media_type = parse_image_media_type(mime);
        parts.push(UserContent::image_base64(data, Some(media_type), None));
    }

    RigMessage::User {
        content: OneOrMany::many(parts).expect("parts is guaranteed non-empty"),
    }
}

fn build_prompt_text(content: &str, attachments: &Option<Vec<MessageAttachment>>) -> String {
    let Some(items) = attachments else {
        return content.to_string();
    };

    let mut text_parts: Vec<String> = items
        .iter()
        .filter_map(|item| match item {
            MessageAttachment::Text {
                extracted_text,
                mime,
                file_name,
                ..
            } => Some(format!(
                "[file: {file_name} ({mime})]\n{extracted_text}\n[/file]"
            )),
            MessageAttachment::Image { .. } => None,
        })
        .collect();
    text_parts.push(content.to_string());
    text_parts.join("\n\n")
}

fn collect_image_attachments(attachments: &Option<Vec<MessageAttachment>>) -> Vec<(&str, &str)> {
    attachments
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| match item {
                    MessageAttachment::Image {
                        data, media_type, ..
                    } => Some((data.as_str(), media_type.as_str())),
                    MessageAttachment::Text { .. } => None,
                })
                .collect()
        })
        .unwrap_or_default()
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
    let attachments = msg
        .attachments
        .as_deref()
        .and_then(|json| serde_json::from_str::<Vec<MessageAttachment>>(json).ok())
        .filter(|items| !items.is_empty());

    build_user_prompt(&msg.content, &attachments)
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
