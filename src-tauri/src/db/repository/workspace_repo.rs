use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;

/// 目录验证信息（返回给前端）
#[derive(Debug, Clone, Serialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub name: String,
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub file_count: Option<u32>,
}

/// 最近使用的工作目录（从 SQLite 读取）
#[derive(Debug, Clone, Serialize)]
pub struct RecentDirectory {
    pub id: String,
    pub path: String,
    pub display_name: Option<String>,
    pub last_used_at: String,
    pub use_count: u32,
}

/// Per-workspace presentation state for the task sidebar.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePreference {
    pub workspace_key: String,
    pub pinned: bool,
    pub hidden: bool,
}

pub struct WorkspaceRepo;

impl WorkspaceRepo {
    /// Must stay aligned with the renderer's workspace grouping key.
    pub fn normalize_workspace_key(path: &str) -> String {
        path.replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase()
    }

    pub fn list_preferences(conn: &Connection) -> Result<Vec<WorkspacePreference>> {
        let mut stmt = conn.prepare(
            "SELECT workspace_key, pinned, hidden
             FROM workspace_preferences
             ORDER BY workspace_key",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WorkspacePreference {
                workspace_key: row.get(0)?,
                pinned: row.get::<_, i64>(1)? != 0,
                hidden: row.get::<_, i64>(2)? != 0,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn update_preference(
        conn: &Connection,
        workspace_key: &str,
        pinned: Option<bool>,
        hidden: Option<bool>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO workspace_preferences (workspace_key, pinned, hidden, updated_at)
             VALUES (?1, COALESCE(?2, 0), COALESCE(?3, 0), CURRENT_TIMESTAMP)
             ON CONFLICT(workspace_key) DO UPDATE SET
                pinned = COALESCE(?2, workspace_preferences.pinned),
                hidden = COALESCE(?3, workspace_preferences.hidden),
                updated_at = CURRENT_TIMESTAMP",
            rusqlite::params![
                workspace_key,
                pinned.map(|value| value as i64),
                hidden.map(|value| value as i64),
            ],
        )?;
        Ok(())
    }

    pub fn restore_workspace(conn: &Connection, path: &str) -> Result<()> {
        let workspace_key = Self::normalize_workspace_key(path);
        conn.execute(
            "UPDATE workspace_preferences
             SET hidden = 0, updated_at = CURRENT_TIMESTAMP
             WHERE workspace_key = ?1 AND hidden != 0",
            [workspace_key],
        )?;
        Ok(())
    }

    /// 查询最近使用的工作目录（按最后使用时间倒序）
    pub fn find_recent(conn: &Connection, limit: u32) -> Result<Vec<RecentDirectory>> {
        let mut stmt = conn.prepare(
            "SELECT id, path, display_name, last_used_at, use_count
             FROM recent_directories
             ORDER BY last_used_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit], |row| {
            Ok(RecentDirectory {
                id: row.get(0)?,
                path: row.get(1)?,
                display_name: row.get(2)?,
                last_used_at: row.get(3)?,
                use_count: row.get::<_, u32>(4).unwrap_or(1),
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 记录目录使用（upsert：存在则更新计数+时间，不存在则插入）
    pub fn record_usage(
        conn: &Connection,
        id: &str,
        path: &str,
        display_name: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO recent_directories (id, path, display_name, last_used_at, use_count)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, 1)
             ON CONFLICT(path) DO UPDATE SET
                 use_count = use_count + 1,
                 last_used_at = CURRENT_TIMESTAMP,
                 display_name = COALESCE(?3, display_name)",
            rusqlite::params![id, path, display_name],
        )?;
        Ok(())
    }

    /// 删除指定路径的目录记录
    pub fn delete_by_path(conn: &Connection, path: &str) -> Result<bool> {
        let affected = conn.execute("DELETE FROM recent_directories WHERE path = ?1", [path])?;
        Ok(affected > 0)
    }

    /// 查询指定路径是否已存在
    pub fn exists(conn: &Connection, path: &str) -> Result<bool> {
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM recent_directories WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .context("Failed to check directory existence")?;
        Ok(count > 0)
    }
}
