use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::{Message, MessageSearchResult};

/// Regeneration 上下文：包含需要重新生成的用户消息及其之前的历史
pub struct RegenerationContext {
    pub user_content: String,
    pub user_msg_id: String,
    pub user_attachments: Option<String>,
    pub messages_before: Vec<Message>,
}

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

    /// 导入完整消息记录（保留所有字段，含 id/时间戳），并同步 FTS 索引。
    pub fn import(conn: &Connection, m: &Message) -> Result<()> {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, token_usage, model,
                thinking_content, attachments, status, tool_calls, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            rusqlite::params![
                m.id,
                m.session_id,
                m.role,
                m.content,
                m.token_usage,
                m.model,
                m.thinking_content,
                m.attachments,
                m.status,
                m.tool_calls,
                m.created_at,
            ],
        )?;
        Self::sync_fts(conn, &m.id, &m.content, &m.session_id, &m.role);
        Ok(())
    }

    /// finalize assistant 消息：一次性写入正文/状态/用量/思维链/工具调用
    ///
    /// `tool_calls` 为与前端 `ToolCall[]` 对齐的 JSON 字符串；`None` 表示本轮无工具调用。
    pub fn update_assistant_content(
        conn: &Connection,
        msg_id: &str,
        content: &str,
        thinking: Option<&str>,
        usage_json: Option<&str>,
        was_aborted: bool,
        tool_calls: Option<&str>,
    ) -> Result<()> {
        let status = if was_aborted { "aborted" } else { "complete" };

        conn.execute(
            "UPDATE messages SET content = ?1, status = ?2, token_usage = ?3,
                    thinking_content = ?4, tool_calls = ?5
             WHERE id = ?6",
            rusqlite::params![content, status, usage_json, thinking, tool_calls, msg_id],
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

    /// 单独更新某条消息的 `tool_calls` JSON（工具循环中途 flush 使用）
    pub fn update_tool_calls(conn: &Connection, msg_id: &str, tool_calls_json: &str) -> Result<()> {
        conn.execute(
            "UPDATE messages SET tool_calls = ?1 WHERE id = ?2",
            rusqlite::params![tool_calls_json, msg_id],
        )?;
        Ok(())
    }

    pub fn find_recent(conn: &Connection, session_id: &str, limit: u32) -> Result<Vec<Message>> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, tool_calls, created_at
             FROM messages
             WHERE session_id = ?1
             ORDER BY created_at DESC, rowid DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(rusqlite::params![session_id, limit], Self::map_row)?;

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
                    thinking_content, attachments, status, tool_calls, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at < ?2
             ORDER BY created_at DESC, rowid DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![session_id, created_at, limit],
            Self::map_row,
        )?;

        Self::collect_reversed(rows)
    }

    /// 加载 regeneration 上下文：目标消息之前最近一条用户消息（含附件）及更早的历史
    pub fn find_regeneration_context(
        conn: &Connection,
        session_id: &str,
        target_message_id: &str,
    ) -> Result<RegenerationContext> {
        let target_created_at: String = conn
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1 AND session_id = ?2",
                rusqlite::params![target_message_id, session_id],
                |row| row.get(0),
            )
            .context("Target message not found")?;

        let (user_content, user_msg_id, user_attachments): (String, String, Option<String>) = conn
            .query_row(
                "SELECT content, id, attachments FROM messages
                 WHERE session_id = ?1 AND role = 'user' AND created_at < ?2
                 ORDER BY created_at DESC, rowid DESC LIMIT 1",
                rusqlite::params![session_id, target_created_at],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .context("No user message found before target")?;

        let user_created_at: String = conn.query_row(
            "SELECT created_at FROM messages WHERE id = ?1",
            [&user_msg_id],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, tool_calls, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at < ?2
             ORDER BY created_at ASC, rowid ASC",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![session_id, user_created_at],
            Self::map_row,
        )?;

        let messages_before: Vec<Message> = rows.filter_map(|r| r.ok()).collect();

        Ok(RegenerationContext {
            user_content,
            user_msg_id,
            user_attachments,
            messages_before,
        })
    }

    pub fn update_status(conn: &Connection, msg_id: &str, status: &str) -> Result<()> {
        conn.execute(
            "UPDATE messages SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, msg_id],
        )?;
        Ok(())
    }

    pub fn update_usage(conn: &Connection, msg_id: &str, usage_json: &str) -> Result<()> {
        conn.execute(
            "UPDATE messages SET token_usage = ?1 WHERE id = ?2",
            rusqlite::params![usage_json, msg_id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, msg_id: &str) -> Result<()> {
        conn.execute("DELETE FROM messages WHERE id = ?1", [msg_id])?;
        Ok(())
    }

    pub fn delete_from(conn: &Connection, session_id: &str, message_id: &str) -> Result<()> {
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

    pub fn search_fts(
        conn: &Connection,
        query: &str,
        session_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<MessageSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let fts_query = build_fts_query(query);
        let (sql, params) = match session_id {
            Some(sid) => (
                "SELECT m.id, m.session_id, s.title, m.role,
                        snippet(messages_fts, 0, '<mark>', '</mark>', '…', 48) AS snip,
                        m.created_at
                 FROM messages_fts AS f
                 JOIN messages AS m ON m.rowid = f.rowid
                 JOIN sessions AS s ON s.id = m.session_id
                 WHERE messages_fts MATCH ?1 AND f.session_id = ?2
                 ORDER BY rank
                 LIMIT ?3"
                    .to_string(),
                vec![
                    Box::new(fts_query.clone()) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(sid.to_string()),
                    Box::new(limit),
                ],
            ),
            None => (
                "SELECT m.id, m.session_id, s.title, m.role,
                        snippet(messages_fts, 0, '<mark>', '</mark>', '…', 48) AS snip,
                        m.created_at
                 FROM messages_fts AS f
                 JOIN messages AS m ON m.rowid = f.rowid
                 JOIN sessions AS s ON s.id = m.session_id
                 WHERE messages_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2"
                    .to_string(),
                vec![
                    Box::new(fts_query.clone()) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit),
                ],
            ),
        };

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(MessageSearchResult {
                id: row.get(0)?,
                session_id: row.get(1)?,
                session_title: row.get(2)?,
                role: row.get(3)?,
                snippet: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn sync_fts(conn: &Connection, msg_id: &str, content: &str, session_id: &str, role: &str) {
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
            status: row
                .get::<_, Option<String>>(8)?
                .unwrap_or_else(|| "complete".to_string()),
            tool_calls: row.get(9)?,
            created_at: row.get(10)?,
        })
    }

    fn collect_reversed(
        rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row) -> rusqlite::Result<Message>>,
    ) -> Result<Vec<Message>> {
        let mut messages: Vec<Message> = rows.filter_map(|r| r.ok()).collect();
        messages.reverse();
        Ok(messages)
    }
}

/// 将用户输入转为 FTS5 查询表达式（每个词加 `*` 前缀匹配，用 AND 连接）
fn build_fts_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(|w| {
            let sanitized: String = w
                .chars()
                .filter(|c| !matches!(c, '"' | '\'' | '*'))
                .collect();
            format!("\"{}\"*", sanitized)
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}
