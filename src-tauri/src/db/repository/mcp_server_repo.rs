use anyhow::{Context, Result};
use rusqlite::Connection;

/// MCP Server 数据库记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerRecord {
    pub id: String,
    pub name: String,
    pub transport_json: String,
    pub auto_connect: bool,
    pub env_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct McpServerRepo;

impl McpServerRepo {
    pub fn insert(conn: &Connection, record: &McpServerRecord) -> Result<()> {
        conn.execute(
            "INSERT INTO mcp_servers (id, name, transport_json, auto_connect, env_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                record.id,
                record.name,
                record.transport_json,
                record.auto_connect,
                record.env_json,
            ],
        )
        .context("Failed to insert MCP server")?;
        Ok(())
    }

    pub fn find_by_id(
        conn: &Connection,
        id: &str,
    ) -> Result<McpServerRecord> {
        conn.query_row(
            "SELECT id, name, transport_json, auto_connect, env_json,
                    created_at, updated_at
             FROM mcp_servers WHERE id = ?1",
            [id],
            Self::map_row,
        )
        .context("MCP server not found")
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<McpServerRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, transport_json, auto_connect, env_json,
                    created_at, updated_at
             FROM mcp_servers ORDER BY created_at",
        )?;

        let records = stmt
            .query_map([], Self::map_row)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(records)
    }

    pub fn update(conn: &Connection, record: &McpServerRecord) -> Result<()> {
        conn.execute(
            "UPDATE mcp_servers
             SET name = ?1, transport_json = ?2, auto_connect = ?3,
                 env_json = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?5",
            rusqlite::params![
                record.name,
                record.transport_json,
                record.auto_connect,
                record.env_json,
                record.id,
            ],
        )
        .context("Failed to update MCP server")?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM mcp_servers WHERE id = ?1",
            [id],
        )
        .context("Failed to delete MCP server")?;
        Ok(())
    }

    fn map_row(
        row: &rusqlite::Row<'_>,
    ) -> rusqlite::Result<McpServerRecord> {
        Ok(McpServerRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            transport_json: row.get(2)?,
            auto_connect: row.get(3)?,
            env_json: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}
