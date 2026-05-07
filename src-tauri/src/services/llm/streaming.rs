use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::traits::{AgentHandle, StreamDelta, StreamUsage};

// ─── Event Payload 结构体 ─────────────────────────────────────────────

/// 流式文本 Token 事件
#[derive(Debug, Clone, Serialize)]
pub struct StreamTokenPayload {
    pub session_id: String,
    pub message_id: String,
    pub delta: String,
}

/// 流式思维链事件（Anthropic thinking blocks / OpenAI reasoning）
#[derive(Debug, Clone, Serialize)]
pub struct StreamThinkingPayload {
    pub session_id: String,
    pub message_id: String,
    pub thinking_delta: String,
}

/// 流式完成事件
#[derive(Debug, Clone, Serialize)]
pub struct StreamCompletePayload {
    pub session_id: String,
    pub message_id: String,
    pub full_content: String,
    pub full_thinking: String,
    pub usage: Option<TokenUsageInfo>,
    pub was_aborted: bool,
}

/// 流式错误事件
#[derive(Debug, Clone, Serialize)]
pub struct StreamErrorPayload {
    pub session_id: String,
    pub message_id: String,
    pub error: String,
}

/// Token 用量信息
#[derive(Debug, Clone, Serialize)]
pub struct TokenUsageInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl From<StreamUsage> for TokenUsageInfo {
    fn from(u: StreamUsage) -> Self {
        Self {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            total_tokens: u.total_tokens,
        }
    }
}

// ─── StreamResult ─────────────────────────────────────────────────────

/// 流式调用的完整结果
#[derive(Debug, Clone)]
pub struct StreamResult {
    pub content: String,
    pub thinking: String,
    pub usage: Option<TokenUsageInfo>,
    pub was_aborted: bool,
}

// ─── StreamSession ────────────────────────────────────────────────────

/// 单次流式会话的执行器
///
/// 管理一次流式 LLM 调用的生命周期：
/// 启动流 → 逐块处理 delta → emit Tauri Event → 汇总完成
pub struct StreamSession {
    message_id: String,
    session_id: String,
    app_handle: AppHandle,
    abort_flag: Arc<AtomicBool>,
    accumulated_content: String,
    accumulated_thinking: String,
    usage: Option<TokenUsageInfo>,
}

impl StreamSession {
    pub fn new(
        message_id: String,
        session_id: String,
        app_handle: AppHandle,
        abort_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            message_id,
            session_id,
            app_handle,
            abort_flag,
            accumulated_content: String::new(),
            accumulated_thinking: String::new(),
            usage: None,
        }
    }

    /// 执行流式调用主循环
    ///
    /// 通过 `AgentHandle::stream_chat` 获取类型擦除的 delta 流，
    /// 逐块分发到 `handle_delta`，最后调用 `finalize` 汇总。
    pub async fn execute_stream(
        &mut self,
        agent: &AgentHandle,
        prompt: &str,
        chat_history: Vec<rig::completion::message::Message>,
    ) -> anyhow::Result<StreamResult> {
        use futures::StreamExt;

        let mut stream = agent
            .stream_chat(prompt, chat_history)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start stream: {e}"))?;

        while let Some(chunk) = stream.next().await {
            if self.abort_flag.load(Ordering::Relaxed) {
                tracing::info!(
                    session_id = %self.session_id,
                    message_id = %self.message_id,
                    "Stream aborted by user"
                );
                break;
            }

            match chunk {
                Ok(delta) => {
                    if let Err(e) = self.handle_delta(delta) {
                        tracing::warn!(error = %e, "Failed to emit stream event, continuing");
                    }
                }
                Err(e) => {
                    self.emit_error(&e.to_string())?;
                    return Err(anyhow::anyhow!("Stream error: {e}"));
                }
            }
        }

        self.finalize()
    }

    /// 处理单个流式 delta（已经过类型擦除）
    fn handle_delta(&mut self, delta: StreamDelta) -> anyhow::Result<()> {
        match delta {
            StreamDelta::Text(text) => {
                if !text.is_empty() {
                    self.emit_text_delta(&text)?;
                }
            }
            StreamDelta::Thinking(thinking) => {
                if !thinking.is_empty() {
                    self.emit_thinking_delta(&thinking)?;
                }
            }
            StreamDelta::ThinkingBlock(block) => {
                if !block.is_empty() {
                    self.emit_thinking_delta(&block)?;
                }
            }
            StreamDelta::Usage(usage) => {
                self.usage = Some(TokenUsageInfo::from(usage));
            }
        }
        Ok(())
    }

    fn emit_text_delta(&mut self, text: &str) -> anyhow::Result<()> {
        self.accumulated_content.push_str(text);
        self.app_handle
            .emit(
                "stream_token",
                StreamTokenPayload {
                    session_id: self.session_id.clone(),
                    message_id: self.message_id.clone(),
                    delta: text.to_string(),
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit stream_token: {e}"))
    }

    fn emit_thinking_delta(&mut self, thinking: &str) -> anyhow::Result<()> {
        self.accumulated_thinking.push_str(thinking);
        self.app_handle
            .emit(
                "stream_thinking",
                StreamThinkingPayload {
                    session_id: self.session_id.clone(),
                    message_id: self.message_id.clone(),
                    thinking_delta: thinking.to_string(),
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit stream_thinking: {e}"))
    }

    /// 汇总流式结果并 emit 完成事件
    fn finalize(&self) -> anyhow::Result<StreamResult> {
        let was_aborted = self.abort_flag.load(Ordering::Relaxed);

        let result = StreamResult {
            content: self.accumulated_content.clone(),
            thinking: self.accumulated_thinking.clone(),
            usage: self.usage.clone(),
            was_aborted,
        };

        self.app_handle
            .emit(
                "stream_complete",
                StreamCompletePayload {
                    session_id: self.session_id.clone(),
                    message_id: self.message_id.clone(),
                    full_content: result.content.clone(),
                    full_thinking: result.thinking.clone(),
                    usage: result.usage.clone(),
                    was_aborted,
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit stream_complete: {e}"))?;

        tracing::info!(
            session_id = %self.session_id,
            message_id = %self.message_id,
            content_len = result.content.len(),
            thinking_len = result.thinking.len(),
            aborted = was_aborted,
            "Stream completed"
        );

        Ok(result)
    }

    /// 发送错误事件
    pub fn emit_error(&self, error: &str) -> anyhow::Result<()> {
        tracing::error!(
            session_id = %self.session_id,
            message_id = %self.message_id,
            error = %error,
            "Stream error"
        );
        self.app_handle
            .emit(
                "stream_error",
                StreamErrorPayload {
                    session_id: self.session_id.clone(),
                    message_id: self.message_id.clone(),
                    error: error.to_string(),
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to emit stream_error: {e}"))
    }
}

// ─── StreamRegistry ───────────────────────────────────────────────────

/// 全局流式会话注册表
///
/// 跟踪所有活跃的流式会话，支持 abort（停止生成）。
/// 使用 `DashMap` 实现无锁并发安全，无需外层 Mutex。
pub struct StreamRegistry {
    active_streams: DashMap<String, Arc<AtomicBool>>,
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self {
            active_streams: DashMap::new(),
        }
    }

    /// 注册新的流式会话，返回用于中断的 abort flag
    pub fn register(&self, session_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.active_streams
            .insert(session_id.to_string(), flag.clone());
        flag
    }

    /// 中止指定会话的流式调用
    ///
    /// 返回 `true` 表示成功标记中止，`false` 表示该会话无活跃流
    pub fn abort(&self, session_id: &str) -> bool {
        if let Some(flag) = self.active_streams.get(session_id) {
            flag.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// 注销已完成的流式会话
    pub fn unregister(&self, session_id: &str) {
        self.active_streams.remove(session_id);
    }

    /// 检查指定会话是否有活跃的流
    pub fn is_active(&self, session_id: &str) -> bool {
        self.active_streams.contains_key(session_id)
    }

    /// 返回当前活跃流的数量（用于调试/监控）
    pub fn active_count(&self) -> usize {
        self.active_streams.len()
    }
}

impl Default for StreamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── StreamRegistry 测试 ──────────────────────────────

    #[test]
    fn test_registry_register_and_unregister() {
        let registry = StreamRegistry::new();
        assert_eq!(registry.active_count(), 0);

        let flag = registry.register("session-1");
        assert!(registry.is_active("session-1"));
        assert_eq!(registry.active_count(), 1);
        assert!(!flag.load(Ordering::Relaxed));

        registry.unregister("session-1");
        assert!(!registry.is_active("session-1"));
        assert_eq!(registry.active_count(), 0);
    }

    #[test]
    fn test_registry_abort() {
        let registry = StreamRegistry::new();
        let flag = registry.register("session-1");

        assert!(!flag.load(Ordering::Relaxed));
        let aborted = registry.abort("session-1");
        assert!(aborted);
        assert!(flag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_registry_abort_nonexistent() {
        let registry = StreamRegistry::new();
        let aborted = registry.abort("nonexistent");
        assert!(!aborted);
    }

    #[test]
    fn test_registry_multiple_sessions() {
        let registry = StreamRegistry::new();
        let flag1 = registry.register("session-1");
        let flag2 = registry.register("session-2");
        assert_eq!(registry.active_count(), 2);

        registry.abort("session-1");
        assert!(flag1.load(Ordering::Relaxed));
        assert!(!flag2.load(Ordering::Relaxed));

        registry.unregister("session-1");
        assert_eq!(registry.active_count(), 1);
        assert!(registry.is_active("session-2"));
    }

    #[test]
    fn test_registry_re_register_replaces_flag() {
        let registry = StreamRegistry::new();
        let old_flag = registry.register("session-1");
        registry.abort("session-1");
        assert!(old_flag.load(Ordering::Relaxed));

        let new_flag = registry.register("session-1");
        assert!(!new_flag.load(Ordering::Relaxed));
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_registry_default() {
        let registry = StreamRegistry::default();
        assert_eq!(registry.active_count(), 0);
    }

    // ─── TokenUsageInfo 测试 ──────────────────────────────

    #[test]
    fn test_token_usage_info_serialize() {
        let usage = TokenUsageInfo {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"input_tokens\":100"));
        assert!(json.contains("\"output_tokens\":50"));
        assert!(json.contains("\"total_tokens\":150"));
    }

    #[test]
    fn test_token_usage_from_stream_usage() {
        let stream_usage = StreamUsage {
            input_tokens: 42,
            output_tokens: 13,
            total_tokens: 55,
        };
        let info = TokenUsageInfo::from(stream_usage);
        assert_eq!(info.input_tokens, 42);
        assert_eq!(info.output_tokens, 13);
        assert_eq!(info.total_tokens, 55);
    }

    // ─── Event Payload 序列化测试 ─────────────────────────

    #[test]
    fn test_stream_token_payload_serialize() {
        let payload = StreamTokenPayload {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            delta: "Hello".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"session_id\":\"s1\""));
        assert!(json.contains("\"delta\":\"Hello\""));
    }

    #[test]
    fn test_stream_thinking_payload_serialize() {
        let payload = StreamThinkingPayload {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            thinking_delta: "Let me think...".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"thinking_delta\":\"Let me think...\""));
    }

    #[test]
    fn test_stream_complete_payload_serialize() {
        let payload = StreamCompletePayload {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            full_content: "Hello world".to_string(),
            full_thinking: "I thought about it".to_string(),
            usage: Some(TokenUsageInfo {
                input_tokens: 10,
                output_tokens: 5,
                total_tokens: 15,
            }),
            was_aborted: false,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"full_content\":\"Hello world\""));
        assert!(json.contains("\"was_aborted\":false"));
        assert!(json.contains("\"input_tokens\":10"));
    }

    #[test]
    fn test_stream_complete_payload_without_usage() {
        let payload = StreamCompletePayload {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            full_content: "".to_string(),
            full_thinking: "".to_string(),
            usage: None,
            was_aborted: true,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"was_aborted\":true"));
        assert!(json.contains("\"usage\":null"));
    }

    #[test]
    fn test_stream_error_payload_serialize() {
        let payload = StreamErrorPayload {
            session_id: "s1".to_string(),
            message_id: "m1".to_string(),
            error: "Connection refused".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"error\":\"Connection refused\""));
    }
}
