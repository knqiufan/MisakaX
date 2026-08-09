//! Provider-neutral seam for R5. R0–R4 deliberately keep it free of provider code.

use serde_json::Value;

use super::types::{BlockFallback, BlockStatus, ContentBlock, ContentBlockKind};

pub fn markdown_block(message_id: String, position: i64, text: String) -> ContentBlock {
    ContentBlock::new(
        message_id,
        position,
        ContentBlockKind::Markdown,
        BlockStatus::Ready,
        serde_json::json!({ "text": text }),
        BlockFallback::default(),
    )
}

pub fn unsupported_block(
    message_id: String,
    position: i64,
    payload: Value,
    fallback: BlockFallback,
) -> ContentBlock {
    ContentBlock::new(
        message_id,
        position,
        ContentBlockKind::Notice,
        BlockStatus::Unsupported,
        payload,
        fallback,
    )
}
