use anyhow::Result;
use rusqlite::Connection;

use crate::db::repository::MessageBlockRepo;
use crate::services::artifacts::ContentSafetyPolicy;

use super::types::ContentBlock;

pub struct MessageBlockService;

impl MessageBlockService {
    pub fn append(
        conn: &Connection,
        block: &ContentBlock,
        policy: &ContentSafetyPolicy,
    ) -> Result<()> {
        block.validate(policy).map_err(anyhow::Error::msg)?;
        MessageBlockRepo::insert(conn, block)
    }

    pub fn list_for_message(conn: &Connection, message_id: &str) -> Result<Vec<ContentBlock>> {
        MessageBlockRepo::find_by_message(conn, message_id)
    }
}
