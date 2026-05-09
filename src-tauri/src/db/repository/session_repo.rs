use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::Session;

pub struct SessionRepo;

impl SessionRepo {
    pub fn find_by_id(conn: &Connection, session_id: &str) -> Result<Session> {
        conn.query_row(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, created_at, updated_at
             FROM sessions WHERE id = ?1",
            [session_id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model: row.get(2)?,
                    system_prompt: row.get(3)?,
                    working_directory: row.get(4)?,
                    project_name: row.get(5)?,
                    status: row.get(6)?,
                    mode: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
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
}

fn extract_project_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}
