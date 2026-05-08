use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::Message;

pub struct MessageRepo;

impl MessageRepo {
    pub fn insert_user_message(
        conn: &Connection,
        msg_id: &str,
        session_id: &str,
        content: &str,
        attachments_json: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, attachments, status)
             VALUES (?1, ?2, 'user', ?3, ?4, 'complete')",
            rusqlite::params![msg_id, session_id, content, attachments_json],
        )?;

        Self::sync_fts(conn, msg_id, content, session_id, "user");
        Ok(())
    }

    pub fn insert_assistant_placeholder(
        conn: &Connection,
        msg_id: &str,
        session_id: &str,
        model_id: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, model, status)
             VALUES (?1, ?2, 'assistant', '', ?3, 'streaming')",
            rusqlite::params![msg_id, session_id, model_id],
        )?;
        Ok(())
    }

    pub fn update_assistant_content(
        conn: &Connection,
        msg_id: &str,
        content: &str,
        thinking: Option<&str>,
        usage_json: Option<&str>,
        was_aborted: bool,
    ) -> Result<()> {
        let status = if was_aborted { "aborted" } else { "complete" };

        conn.execute(
            "UPDATE messages SET content = ?1, status = ?2, token_usage = ?3,
                    thinking_content = ?4
             WHERE id = ?5",
            rusqlite::params![content, status, usage_json, thinking, msg_id],
        )?;

        let session_id: String = conn
            .query_row(
                "SELECT session_id FROM messages WHERE id = ?1",
                [msg_id],
                |row| row.get(0),
            )
            .context("Message not found when syncing FTS")?;

        Self::sync_fts(conn, msg_id, content, &session_id, "assistant");
        Ok(())
    }

    pub fn find_recent(
        conn: &Connection,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<Message>> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, created_at
             FROM messages
             WHERE session_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![session_id, limit],
            Self::map_row,
        )?;

        Self::collect_reversed(rows)
    }

    pub fn find_before(
        conn: &Connection,
        session_id: &str,
        before_id: &str,
        limit: u32,
    ) -> Result<Vec<Message>> {
        let created_at: String = conn
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1",
                [before_id],
                |row| row.get(0),
            )
            .context("Cursor message not found")?;

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at < ?2
             ORDER BY created_at DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![session_id, created_at, limit],
            Self::map_row,
        )?;

        Self::collect_reversed(rows)
    }

    /// 加载 regeneration 上下文：目标消息之前最近一条用户消息及更早的历史
    pub fn find_regeneration_context(
        conn: &Connection,
        session_id: &str,
        target_message_id: &str,
    ) -> Result<(String, String, Vec<Message>)> {
        let target_created_at: String = conn
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1 AND session_id = ?2",
                rusqlite::params![target_message_id, session_id],
                |row| row.get(0),
            )
            .context("Target message not found")?;

        let (user_content, user_msg_id) = conn
            .query_row(
                "SELECT content, id FROM messages
                 WHERE session_id = ?1 AND role = 'user' AND created_at < ?2
                 ORDER BY created_at DESC LIMIT 1",
                rusqlite::params![session_id, target_created_at],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .context("No user message found before target")?;

        let user_created_at: String = conn
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1",
                [&user_msg_id],
                |row| row.get(0),
            )?;

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at < ?2
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![session_id, user_created_at],
            Self::map_row,
        )?;

        let messages_before: Vec<Message> = rows.filter_map(|r| r.ok()).collect();

        Ok((user_content, user_msg_id, messages_before))
    }

    pub fn delete_from(
        conn: &Connection,
        session_id: &str,
        message_id: &str,
    ) -> Result<()> {
        let created_at: String = conn
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1 AND session_id = ?2",
                rusqlite::params![message_id, session_id],
                |row| row.get(0),
            )
            .context("Message not found for deletion")?;

        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1 AND created_at >= ?2",
            rusqlite::params![session_id, created_at],
        )?;

        Ok(())
    }

    fn sync_fts(
        conn: &Connection,
        msg_id: &str,
        content: &str,
        session_id: &str,
        role: &str,
    ) {
        if content.is_empty() {
            return;
        }
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO messages_fts (rowid, content, session_id, role)
             VALUES ((SELECT rowid FROM messages WHERE id = ?1), ?2, ?3, ?4)",
            rusqlite::params![msg_id, content, session_id, role],
        ) {
            tracing::warn!(error = %e, "FTS sync failed for message {}", msg_id);
        }
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Message> {
        Ok(Message {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            token_usage: row.get(4)?,
            model: row.get(5)?,
            thinking_content: row.get(6)?,
            attachments: row.get(7)?,
            status: row.get::<_, Option<String>>(8)?
                .unwrap_or_else(|| "complete".to_string()),
            created_at: row.get(9)?,
        })
    }

    fn collect_reversed(
        rows: rusqlite::MappedRows<
            '_,
            impl FnMut(&rusqlite::Row) -> rusqlite::Result<Message>,
        >,
    ) -> Result<Vec<Message>> {
        let mut messages: Vec<Message> = rows.filter_map(|r| r.ok()).collect();
        messages.reverse();
        Ok(messages)
    }
}
