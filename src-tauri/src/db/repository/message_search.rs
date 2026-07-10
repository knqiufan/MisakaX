//! Message full-text search (FTS5) — read-side search queries over
//! `messages_fts`.
//!
//! Split out of [`super::message_repo`] for focus: the search queries live here,
//! while index *maintenance* ([`MessageRepo::sync_fts`]-driven) stays with the
//! write path in `message_repo`. The API surface is unchanged —
//! [`MessageRepo::search_fts`] is still the entry point.

use anyhow::Result;
use rusqlite::Connection;

use crate::db::models::MessageSearchResult;
use crate::db::repository::MessageRepo;

impl MessageRepo {
    /// 全文检索消息（按 session 可选过滤），返回带高亮摘要的命中结果。
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
