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
pub use streaming::{StreamRegistry, StreamResult, StreamSession, TokenUsageInfo};
pub use traits::{AgentHandle, DeltaStream, LlmProvider, StreamDelta, StreamUsage};
