use serde::{Deserialize, Serialize};

/// LLM 运行时参数配置
///
/// 控制模型推理行为的参数集合。
/// 前端通过 Tauri Command 传入，Rust 端构建请求时应用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 采样温度 (0.0 - 2.0)，越高越随机
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// 最大生成 token 数
    #[serde(default)]
    pub max_tokens: Option<u32>,

    /// 核采样概率 (0.0 - 1.0)
    #[serde(default)]
    pub top_p: Option<f64>,

    /// 频率惩罚 (-2.0 - 2.0)
    #[serde(default)]
    pub frequency_penalty: Option<f64>,

    /// 存在惩罚 (-2.0 - 2.0)
    #[serde(default)]
    pub presence_penalty: Option<f64>,

    /// 停止序列
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,

    /// Composer thinking preference snapshot for this turn (default true).
    #[serde(default = "default_thinking_enabled")]
    pub thinking_enabled: bool,

    /// Composer deep-research preference (`chat` | `research`, default chat).
    #[serde(default = "default_agent_mode")]
    pub agent_mode: String,
}

fn default_temperature() -> f64 {
    0.7
}

fn default_thinking_enabled() -> bool {
    true
}

fn default_agent_mode() -> String {
    "chat".to_string()
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
            thinking_enabled: true,
            agent_mode: default_agent_mode(),
        }
    }
}

impl LlmConfig {
    /// 校验参数范围，超出范围则钳位到合法值
    pub fn sanitized(mut self) -> Self {
        self.temperature = self.temperature.clamp(0.0, 2.0);
        if let Some(tp) = self.top_p {
            self.top_p = Some(tp.clamp(0.0, 1.0));
        }
        if let Some(fp) = self.frequency_penalty {
            self.frequency_penalty = Some(fp.clamp(-2.0, 2.0));
        }
        if let Some(pp) = self.presence_penalty {
            self.presence_penalty = Some(pp.clamp(-2.0, 2.0));
        }
        let mode = self.agent_mode.trim().to_ascii_lowercase();
        self.agent_mode = if mode == "research" {
            "research".to_string()
        } else {
            "chat".to_string()
        };
        self
    }
}
