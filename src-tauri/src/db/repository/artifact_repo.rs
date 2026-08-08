use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::services::artifacts::{ArtifactOrigin, ArtifactRecord, PreviewState, RetentionState};

pub struct ArtifactRepo;

impl ArtifactRepo {
    pub fn insert(conn: &Connection, record: &ArtifactRecord) -> Result<()> {
        conn.execute(
            "INSERT INTO artifacts (
                artifact_id, owner_session_id, origin_message_id, origin_kind, display_name,
                media_type, byte_size, sha256, storage_key, preview_state, preview_artifact_id,
                retention_state, created_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                record.artifact_id,
                record.owner_session_id,
                record.origin_message_id,
                serde_json::to_string(&record.origin_kind)?,
                record.display_name,
                record.media_type,
                record.byte_size,
                record.sha256,
                record.storage_key,
                serde_json::to_string(&record.preview_state)?,
                record.preview_artifact_id,
                serde_json::to_string(&record.retention_state)?,
                record.created_at,
                record.expires_at,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, artifact_id: &str) -> Result<ArtifactRecord> {
        conn.query_row(
            "SELECT artifact_id, owner_session_id, origin_message_id, origin_kind, display_name,
                    media_type, byte_size, sha256, storage_key, preview_state, preview_artifact_id,
                    retention_state, created_at, expires_at
             FROM artifacts WHERE artifact_id = ?1",
            [artifact_id],
            Self::map_row,
        )
        .context("ARTIFACT_NOT_FOUND")
    }

    pub fn active_bytes(conn: &Connection) -> Result<u64> {
        let bytes = conn.query_row(
            "SELECT COALESCE(SUM(byte_size), 0) FROM artifacts WHERE retention_state = '\"active\"'",
            [],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(bytes.max(0) as u64)
    }

    pub fn find_by_sessions(
        conn: &Connection,
        session_ids: &[String],
    ) -> Result<Vec<ArtifactRecord>> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = std::iter::repeat("?")
            .take(session_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT artifact_id, owner_session_id, origin_message_id, origin_kind, display_name,
                    media_type, byte_size, sha256, storage_key, preview_state, preview_artifact_id,
                    retention_state, created_at, expires_at
             FROM artifacts WHERE owner_session_id IN ({placeholders}) ORDER BY created_at ASC"
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(session_ids), Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn set_retention_state(
        conn: &Connection,
        artifact_id: &str,
        state: RetentionState,
    ) -> Result<()> {
        conn.execute(
            "UPDATE artifacts SET retention_state = ?1 WHERE artifact_id = ?2",
            rusqlite::params![serde_json::to_string(&state)?, artifact_id],
        )?;
        Ok(())
    }

    pub fn has_active_storage_reference(
        conn: &Connection,
        storage_key: &str,
        exclude_artifact_id: &str,
    ) -> Result<bool> {
        let count = conn.query_row(
            "SELECT COUNT(*) FROM artifacts
             WHERE storage_key = ?1 AND artifact_id != ?2 AND retention_state = '\"active\"'",
            rusqlite::params![storage_key, exclude_artifact_id],
            |row| row.get::<_, i64>(0),
        )?;
        Ok(count > 0)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactRecord> {
        let parse = |column: usize| -> rusqlite::Result<String> { row.get(column) };
        let origin_kind = serde_json::from_str::<ArtifactOrigin>(&parse(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let preview_state = serde_json::from_str::<PreviewState>(&parse(9)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let retention_state =
            serde_json::from_str::<RetentionState>(&parse(11)?).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
        Ok(ArtifactRecord {
            artifact_id: row.get(0)?,
            owner_session_id: row.get(1)?,
            origin_message_id: row.get(2)?,
            origin_kind,
            display_name: row.get(4)?,
            media_type: row.get(5)?,
            byte_size: row.get::<_, i64>(6)?.max(0) as u64,
            sha256: row.get(7)?,
            storage_key: row.get(8)?,
            preview_state,
            preview_artifact_id: row.get(10)?,
            retention_state,
            created_at: row.get(12)?,
            expires_at: row.get(13)?,
        })
    }
}
