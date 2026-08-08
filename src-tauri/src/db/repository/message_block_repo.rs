use anyhow::Result;
use rusqlite::Connection;

use crate::services::content::{BlockFallback, BlockStatus, ContentBlock, ContentBlockKind};

pub struct MessageBlockRepo;

impl MessageBlockRepo {
    pub fn insert(conn: &Connection, block: &ContentBlock) -> Result<()> {
        conn.execute(
            "INSERT INTO message_blocks (
                id, message_id, position, kind, schema_version, payload_json, status,
                fallback_json, generation, revision, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                block.id,
                block.message_id,
                block.position,
                serde_json::to_string(&block.kind)?,
                block.schema_version,
                serde_json::to_string(&block.payload)?,
                serde_json::to_string(&block.status)?,
                serde_json::to_string(&block.fallback)?,
                block.generation,
                block.revision,
                block.created_at,
                block.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_message(conn: &Connection, message_id: &str) -> Result<Vec<ContentBlock>> {
        let mut statement = conn.prepare(
            "SELECT id, message_id, position, kind, schema_version, payload_json, status,
                    fallback_json, generation, revision, created_at, updated_at
             FROM message_blocks WHERE message_id = ?1 ORDER BY position ASC, rowid ASC",
        )?;
        let rows = statement.query_map([message_id], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn find_by_messages(
        conn: &Connection,
        message_ids: &[String],
    ) -> Result<Vec<ContentBlock>> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = std::iter::repeat("?")
            .take(message_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT id, message_id, position, kind, schema_version, payload_json, status,
                    fallback_json, generation, revision, created_at, updated_at
             FROM message_blocks WHERE message_id IN ({placeholders}) ORDER BY message_id, position, rowid"
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(message_ids), Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentBlock> {
        let parse = |column: usize| -> rusqlite::Result<String> { row.get(column) };
        let kind = serde_json::from_str::<ContentBlockKind>(&parse(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let payload = serde_json::from_str(&parse(5)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let status = serde_json::from_str::<BlockStatus>(&parse(6)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let fallback = serde_json::from_str::<BlockFallback>(&parse(7)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        Ok(ContentBlock {
            id: row.get(0)?,
            message_id: row.get(1)?,
            position: row.get(2)?,
            kind,
            schema_version: row.get(4)?,
            payload,
            status,
            fallback,
            generation: row.get(8)?,
            revision: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::MessageBlockRepo;
    use crate::db::repository::MessageRepo;
    use crate::services::content::{BlockFallback, BlockStatus, ContentBlock, ContentBlockKind};

    #[test]
    fn round_trips_an_ordered_block_without_changing_legacy_message_content() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute("INSERT INTO sessions (id) VALUES ('session')", [])
            .unwrap();
        MessageRepo::insert_user_message(&conn, "message", "session", "legacy markdown", None)
            .unwrap();
        let block = ContentBlock::new(
            "message".to_string(),
            2,
            ContentBlockKind::Markdown,
            BlockStatus::Ready,
            serde_json::json!({"text": "structured markdown"}),
            BlockFallback::default(),
        );

        MessageBlockRepo::insert(&conn, &block).unwrap();
        let restored = MessageBlockRepo::find_by_message(&conn, "message").unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].id, block.id);
        assert_eq!(restored[0].position, 2);
        assert_eq!(restored[0].payload["text"], "structured markdown");
        let content: String = conn
            .query_row(
                "SELECT content FROM messages WHERE id = 'message'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(content, "legacy markdown");
    }
}
