use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub server_id: String,
    pub tool_name: String,
    pub policy: String,
    pub updated_at: String,
}

pub struct ToolPermissionRepo;

impl ToolPermissionRepo {
    pub fn find_policy(
        conn: &Connection,
        server_id: &str,
        tool_name: &str,
    ) -> Result<Option<String>> {
        let result = conn.query_row(
            "SELECT policy FROM tool_permissions WHERE server_id = ?1 AND tool_name = ?2",
            rusqlite::params![server_id, tool_name],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(policy) => Ok(Some(policy)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query tool permission"),
        }
    }

    pub fn upsert_policy(
        conn: &Connection,
        server_id: &str,
        tool_name: &str,
        policy: &str,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO tool_permissions (server_id, tool_name, policy, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(server_id, tool_name) DO UPDATE SET
                policy = excluded.policy,
                updated_at = excluded.updated_at",
            rusqlite::params![server_id, tool_name, policy],
        )
        .context("Failed to upsert tool permission")?;
        Ok(())
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<ToolPermission>> {
        let mut stmt = conn.prepare(
            "SELECT server_id, tool_name, policy, updated_at
             FROM tool_permissions ORDER BY updated_at DESC",
        )?;

        let records = stmt
            .query_map([], |row| {
                Ok(ToolPermission {
                    server_id: row.get(0)?,
                    tool_name: row.get(1)?,
                    policy: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(records)
    }

    pub fn reset(
        conn: &Connection,
        server_id: &str,
        tool_name: &str,
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM tool_permissions WHERE server_id = ?1 AND tool_name = ?2",
            rusqlite::params![server_id, tool_name],
        )
        .context("Failed to reset tool permission")?;
        Ok(())
    }
}
