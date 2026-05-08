pub mod backend;
pub mod config;
pub mod factory;
pub mod providers;
pub mod registry;
pub mod streaming;
pub mod traits;

pub use backend::{ChatBackend, ImageAttachment, RigBackend};
pub use config::LlmConfig;
pub use factory::ProviderFactory;
pub use registry::{ModelInfo, ModelRegistry};
pub use streaming::{
    StreamCompletePayload, StreamErrorPayload, StreamRegistry, StreamResult,
    StreamSession, StreamThinkingPayload, StreamTokenPayload, TokenUsageInfo,
};
pub use traits::{AgentHandle, DeltaStream, LlmProvider, StreamDelta, StreamUsage};
