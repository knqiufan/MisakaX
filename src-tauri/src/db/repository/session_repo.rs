use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::Session;

pub struct SessionRepo;

impl SessionRepo {
    pub fn find_by_id(conn: &Connection, session_id: &str) -> Result<Session> {
        conn.query_row(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, total_input_tokens, total_output_tokens,
                    last_message_at, pinned, group_name, created_at, updated_at
             FROM sessions WHERE id = ?1",
            [session_id],
            Self::map_row,
        )
        .context("Session not found")
    }

    pub fn create(
        conn: &Connection,
        id: &str,
        title: Option<&str>,
        model: Option<&str>,
        working_directory: Option<&str>,
    ) -> Result<Session> {
        let project_name = working_directory.map(extract_project_name);

        conn.execute(
            "INSERT INTO sessions (id, title, model, working_directory, project_name, status, mode)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active', 'agent')",
            rusqlite::params![id, title, model, working_directory, project_name],
        )?;

        Self::find_by_id(conn, id)
    }

    /// 导入完整会话记录（保留所有字段，含 id/时间戳/统计）。
    pub fn import(conn: &Connection, s: &Session) -> Result<()> {
        conn.execute(
            "INSERT INTO sessions (id, title, model, system_prompt, working_directory, project_name,
                status, mode, total_input_tokens, total_output_tokens,
                last_message_at, pinned, group_name, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
            rusqlite::params![
                s.id,
                s.title,
                s.model,
                s.system_prompt,
                s.working_directory,
                s.project_name,
                s.status,
                s.mode,
                s.total_input_tokens,
                s.total_output_tokens,
                s.last_message_at,
                s.pinned as i32,
                s.group_name,
                s.created_at,
                s.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection, status: Option<&str>) -> Result<Vec<Session>> {
        let status_filter = status.unwrap_or("active");

        let mut stmt = conn.prepare(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, total_input_tokens, total_output_tokens,
                    last_message_at, pinned, group_name, created_at, updated_at
             FROM sessions
             WHERE status = ?1
             ORDER BY pinned DESC, COALESCE(last_message_at, updated_at) DESC",
        )?;

        let rows = stmt.query_map([status_filter], Self::map_row)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn update(
        conn: &Connection,
        session_id: &str,
        title: Option<&str>,
        model: Option<&str>,
        pinned: Option<bool>,
        status: Option<&str>,
    ) -> Result<()> {
        let mut set_clauses = vec!["updated_at = CURRENT_TIMESTAMP"];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = title {
            set_clauses.push("title = ?");
            params.push(Box::new(t.to_string()));
        }
        if let Some(m) = model {
            set_clauses.push("model = ?");
            params.push(Box::new(m.to_string()));
        }
        if let Some(p) = pinned {
            set_clauses.push("pinned = ?");
            params.push(Box::new(p as i32));
        }
        if let Some(s) = status {
            set_clauses.push("status = ?");
            params.push(Box::new(s.to_string()));
        }

        params.push(Box::new(session_id.to_string()));

        let param_start = 1;
        let mut numbered_clauses = Vec::new();
        let mut idx = param_start;
        for clause in &set_clauses {
            if clause.contains('?') {
                numbered_clauses.push(clause.replace('?', &format!("?{idx}")));
                idx += 1;
            } else {
                numbered_clauses.push(clause.to_string());
            }
        }
        let id_placeholder = format!("?{idx}");

        let sql = format!(
            "UPDATE sessions SET {} WHERE id = {}",
            numbered_clauses.join(", "),
            id_placeholder,
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())?;
        Ok(())
    }

    pub fn delete(conn: &Connection, session_id: &str) -> Result<()> {
        conn.execute("DELETE FROM messages WHERE session_id = ?1", [session_id])?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
        Ok(())
    }

    pub fn search(conn: &Connection, query: &str) -> Result<Vec<Session>> {
        let pattern = format!("%{query}%");
        let mut stmt = conn.prepare(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, total_input_tokens, total_output_tokens,
                    last_message_at, pinned, group_name, created_at, updated_at
             FROM sessions
             WHERE (title LIKE ?1 OR project_name LIKE ?1)
               AND status = 'active'
             ORDER BY COALESCE(last_message_at, updated_at) DESC
             LIMIT 50",
        )?;

        let rows = stmt.query_map([&pattern], Self::map_row)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn update_working_directory(
        conn: &Connection,
        session_id: &str,
        working_directory: Option<&str>,
    ) -> Result<()> {
        let project_name = working_directory.map(extract_project_name);

        conn.execute(
            "UPDATE sessions
             SET working_directory = ?1,
                 project_name = ?2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            rusqlite::params![working_directory, project_name, session_id],
        )?;
        Ok(())
    }

    pub fn update_stats(
        conn: &Connection,
        session_id: &str,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
    ) -> Result<()> {
        if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
            conn.execute(
                "UPDATE sessions
                 SET total_input_tokens = total_input_tokens + ?1,
                     total_output_tokens = total_output_tokens + ?2,
                     last_message_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?3",
                rusqlite::params![input, output, session_id],
            )?;
        } else {
            conn.execute(
                "UPDATE sessions
                 SET last_message_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                [session_id],
            )?;
        }
        Ok(())
    }

    pub fn pin_session(conn: &Connection, session_id: &str, pinned: bool) -> Result<()> {
        conn.execute(
            "UPDATE sessions SET pinned = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![pinned as i32, session_id],
        )?;
        Ok(())
    }

    pub fn archive_session(conn: &Connection, session_id: &str) -> Result<()> {
        conn.execute(
            "UPDATE sessions SET status = 'archived', updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [session_id],
        )?;
        Ok(())
    }

    pub fn unarchive_session(conn: &Connection, session_id: &str) -> Result<()> {
        conn.execute(
            "UPDATE sessions SET status = 'active', updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [session_id],
        )?;
        Ok(())
    }

    pub fn set_group(conn: &Connection, session_id: &str, group: Option<&str>) -> Result<()> {
        conn.execute(
            "UPDATE sessions SET group_name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![group, session_id],
        )?;
        Ok(())
    }

    pub fn list_groups(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT group_name FROM sessions
             WHERE group_name IS NOT NULL AND group_name != '' AND status = 'active'
             ORDER BY group_name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_all_for_export(conn: &Connection) -> Result<Vec<Session>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, total_input_tokens, total_output_tokens,
                    last_message_at, pinned, group_name, created_at, updated_at
             FROM sessions
             ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], Self::map_row)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Session> {
        Ok(Session {
            id: row.get(0)?,
            title: row.get(1)?,
            model: row.get(2)?,
            system_prompt: row.get(3)?,
            working_directory: row.get(4)?,
            project_name: row.get(5)?,
            status: row.get(6)?,
            mode: row.get(7)?,
            total_input_tokens: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
            total_output_tokens: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
            last_message_at: row.get(10)?,
            pinned: row.get::<_, Option<i32>>(11)?.unwrap_or(0) != 0,
            group_name: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }
}

fn extract_project_name(path: &str) -> String {
    // Split on both `/` and `\` so the result is the same regardless of host
    // OS. Otherwise a Windows-style path like `D:\code\Misaka-Tauri` yields the
    // whole string on a Unix host where `\` is not a separator.
    path.rsplit(['/', '\\'])
        .find(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}
