pub mod backend;
pub mod catalog;
pub mod config;
pub mod factory;
pub mod providers;
pub mod registry;
pub mod streaming;
pub mod traits;

pub use backend::{
    build_rig_chat_history, build_user_prompt, ChatBackend, ImageAttachment, RigBackend,
};
pub use catalog::{
    chat_url, endpoint, google_generate_url, models_url, supported_vendors, ProviderApi,
    VendorEndpoint,
};
pub use config::LlmConfig;
pub use factory::ProviderFactory;
pub use registry::{ModelInfo, ModelRegistry};
pub use streaming::{
    StreamCompletePayload, StreamErrorPayload, StreamRegistry, StreamResult, StreamSession,
    StreamThinkingPayload, StreamTokenPayload, TokenUsageInfo,
};
pub use traits::{AgentHandle, DeltaStream, LlmProvider, StreamDelta, StreamUsage};
