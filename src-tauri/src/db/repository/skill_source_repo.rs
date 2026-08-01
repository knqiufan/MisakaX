use std::collections::{HashMap, HashSet};

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::services::skills::types::{MessageSkillSelection, SkillMigrationStatus, SkillRecord};

pub struct SkillSourceRepo;

impl SkillSourceRepo {
    pub fn list(conn: &Connection) -> Result<Vec<SkillRecord>> {
        let mut statement = conn.prepare(
            "SELECT skill_id, slug, name, description, version, source_kind,
                    source_ref, source_url, artifact_hash, installed_path,
                    user_enabled, health, is_managed, security_state,
                    effective_rank, disabled_reason,
                    installed_at, updated_at
             FROM skill_sources
             ORDER BY slug COLLATE NOCASE, effective_rank DESC, updated_at DESC",
        )?;
        let rows = statement.query_map([], Self::map_row)?;
        let mut records = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)?;
        Self::decorate_conflicts(&mut records);
        Ok(records)
    }

    pub fn find(conn: &Connection, skill_id: &str) -> Result<Option<SkillRecord>> {
        conn.query_row(
            "SELECT skill_id, slug, name, description, version, source_kind,
                    source_ref, source_url, artifact_hash, installed_path,
                    user_enabled, health, is_managed, security_state,
                    effective_rank, disabled_reason,
                    installed_at, updated_at
             FROM skill_sources WHERE skill_id = ?1",
            [skill_id],
            Self::map_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn upsert_source(
        conn: &Connection,
        record: &SkillRecord,
        source_locator: &str,
        is_managed: bool,
    ) -> Result<(String, bool)> {
        let previous = conn
            .query_row(
                "SELECT skill_id, slug, name, description, version, artifact_hash,
                        installed_path, health, effective_rank
                 FROM skill_sources WHERE source_kind = ?1 AND source_locator = ?2",
                params![normalized_source_kind(record, is_managed), source_locator],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i32>(8)?,
                    ))
                },
            )
            .optional()?;
        let skill_id = previous
            .as_ref()
            .map(|value| value.0.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let source_kind = normalized_source_kind(record, is_managed);
        let rank = source_rank(&source_kind, is_managed);
        let initial_enabled = if is_managed { record.enabled } else { false };
        let initial_security = record.security_state.as_str();
        conn.execute(
            "INSERT INTO skill_sources (
                skill_id, slug, name, description, version, source_kind,
                source_locator, source_ref, source_url, artifact_hash,
                installed_path, is_managed, user_enabled, health,
                security_state, effective_rank, disabled_reason
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16, NULL
             )
             ON CONFLICT(source_kind, source_locator) DO UPDATE SET
                slug = excluded.slug, name = excluded.name,
                description = excluded.description, version = excluded.version,
                source_ref = excluded.source_ref, source_url = excluded.source_url,
                user_enabled = CASE
                    WHEN skill_sources.artifact_hash != excluded.artifact_hash
                      OR excluded.health != 'healthy' THEN 0
                    ELSE skill_sources.user_enabled
                END,
                security_state = CASE
                    WHEN skill_sources.artifact_hash != excluded.artifact_hash
                      THEN 'stale'
                    ELSE skill_sources.security_state
                END,
                disabled_reason = CASE
                    WHEN skill_sources.artifact_hash != excluded.artifact_hash
                      THEN 'artifact_changed'
                    WHEN excluded.health != 'healthy' THEN 'source_missing'
                    ELSE NULL
                END,
                artifact_hash = excluded.artifact_hash,
                installed_path = excluded.installed_path, health = excluded.health,
                effective_rank = excluded.effective_rank,
                updated_at = CURRENT_TIMESTAMP",
            params![
                skill_id,
                record.slug,
                record.name,
                record.description,
                record.version,
                source_kind,
                source_locator,
                record.source_ref,
                record.source_url,
                record.checksum,
                record.installed_path,
                is_managed,
                initial_enabled,
                record.health,
                initial_security,
                rank,
            ],
        )?;
        let changed = match previous {
            None => true,
            Some(old) => {
                old.1 != record.slug
                    || old.2 != record.name
                    || old.3 != record.description
                    || old.4 != record.version
                    || old.5 != record.checksum
                    || old.6 != record.installed_path
                    || old.7 != record.health
                    || old.8 != rank
            }
        };
        if changed {
            Self::bump_generation(conn)?;
        }
        Ok((skill_id, changed))
    }

    pub fn mark_missing_external_except(
        conn: &Connection,
        present_locators: &HashSet<String>,
    ) -> Result<bool> {
        let mut statement = conn.prepare(
            "SELECT source_locator FROM skill_sources
             WHERE is_managed = 0 AND health != 'missing'",
        )?;
        let candidates = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let missing = candidates
            .into_iter()
            .filter(|locator| !present_locators.contains(locator))
            .collect::<Vec<_>>();
        drop(statement);
        for locator in &missing {
            conn.execute(
                "UPDATE skill_sources
                 SET health = 'missing', user_enabled = 0,
                     disabled_reason = 'source_missing', updated_at = CURRENT_TIMESTAMP
                 WHERE is_managed = 0 AND source_locator = ?1",
                [locator],
            )?;
        }
        if !missing.is_empty() {
            Self::bump_generation(conn)?;
        }
        Ok(!missing.is_empty())
    }

    pub fn set_enabled(conn: &Connection, skill_id: &str, enabled: bool) -> Result<u64> {
        let record = Self::find(conn, skill_id)?.context("Skill source is not registered")?;
        if enabled && record.health != "healthy" {
            bail!("Cannot enable a missing or corrupted Skill source")
        }
        let security_state = record.security_state.as_str();
        let changed = conn.execute(
            "UPDATE skill_sources
             SET user_enabled = ?1, security_state = ?2,
                 disabled_reason = CASE WHEN ?1 THEN NULL ELSE 'user_disabled' END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = ?3 AND (user_enabled != ?1 OR security_state != ?2)",
            params![enabled, security_state, skill_id],
        )?;
        if changed > 0 {
            Self::bump_generation(conn)?;
        }
        Self::generation(conn)
    }

    pub fn delete_source(conn: &Connection, skill_id: &str) -> Result<()> {
        let changed = conn.execute("DELETE FROM skill_sources WHERE skill_id = ?1", [skill_id])?;
        if changed > 0 {
            Self::bump_generation(conn)?;
        }
        Ok(())
    }

    pub fn generation(conn: &Connection) -> Result<u64> {
        conn.query_row(
            "SELECT generation FROM skill_activation_state WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(Into::into)
    }

    pub fn bump_generation(conn: &Connection) -> Result<u64> {
        conn.execute(
            "UPDATE skill_activation_state
             SET generation = generation + 1, updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [],
        )?;
        Self::generation(conn)
    }

    pub fn replace_message_selection(
        conn: &Connection,
        message_id: &str,
        selections: &[MessageSkillSelection],
    ) -> Result<()> {
        let transaction = conn.unchecked_transaction()?;
        transaction.execute(
            "DELETE FROM message_skill_selections WHERE message_id = ?1",
            [message_id],
        )?;
        for (index, selection) in selections.iter().enumerate() {
            transaction.execute(
                "INSERT INTO message_skill_selections (
                    message_id, skill_id, skill_slug_snapshot,
                    artifact_hash_snapshot, sort_order
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    message_id,
                    selection.skill_id,
                    selection.slug_snapshot,
                    selection.artifact_hash_snapshot,
                    index as i64,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn message_selection(
        conn: &Connection,
        message_id: &str,
    ) -> Result<Vec<MessageSkillSelection>> {
        let mut statement = conn.prepare(
            "SELECT skill_id, skill_slug_snapshot, artifact_hash_snapshot
             FROM message_skill_selections
             WHERE message_id = ?1 ORDER BY sort_order",
        )?;
        let rows = statement.query_map([message_id], |row| {
            Ok(MessageSkillSelection {
                skill_id: row.get(0)?,
                slug_snapshot: row.get(1)?,
                artifact_hash_snapshot: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn migration_status(conn: &Connection) -> Result<SkillMigrationStatus> {
        conn.query_row(
            "SELECT state, total, completed, failed, current_skill_id, last_error
             FROM skill_security_migration WHERE singleton = 1",
            [],
            |row| {
                Ok(SkillMigrationStatus {
                    state: row.get(0)?,
                    total: row.get(1)?,
                    completed: row.get(2)?,
                    failed: row.get(3)?,
                    current_skill_id: row.get(4)?,
                    last_error: row.get(5)?,
                })
            },
        )
        .map_err(Into::into)
    }

    pub fn pending_migration_sources(conn: &Connection) -> Result<Vec<SkillRecord>> {
        let mut statement = conn.prepare(
            "SELECT skill_id, slug, name, description, version, source_kind,
                    source_ref, source_url, artifact_hash, installed_path,
                    user_enabled, health, is_managed, security_state,
                    effective_rank, disabled_reason,
                    source.installed_at, source.updated_at
             FROM skill_sources source
             JOIN skill_security_migration_items item USING(skill_id)
             WHERE item.state = 'pending'
             ORDER BY item.updated_at, skill_id",
        )?;
        let records = statement
            .query_map([], Self::map_row)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)?;
        Ok(records)
    }

    pub fn enroll_unscanned_migration_sources(conn: &Connection) -> Result<SkillMigrationStatus> {
        conn.execute(
            "INSERT OR IGNORE INTO skill_security_migration_items(skill_id, state)
             SELECT skill_id, 'pending' FROM skill_sources
             WHERE security_state = 'unscanned' AND health = 'healthy'",
            [],
        )?;
        conn.execute(
            "UPDATE skill_sources
             SET user_enabled = 0, disabled_reason = 'migration_scan_required',
                 updated_at = CURRENT_TIMESTAMP
             WHERE skill_id IN (
                 SELECT skill_id FROM skill_security_migration_items WHERE state = 'pending'
             )",
            [],
        )?;
        conn.execute(
            "UPDATE skill_security_migration
             SET total = (SELECT COUNT(*) FROM skill_security_migration_items),
                 state = CASE
                    WHEN (SELECT COUNT(*) FROM skill_security_migration_items
                          WHERE state = 'pending') > 0 THEN 'pending'
                    ELSE state END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [],
        )?;
        Self::migration_status(conn)
    }

    pub fn start_migration_scan(conn: &Connection, skill_id: &str) -> Result<()> {
        conn.execute(
            "UPDATE skill_security_migration_items
             SET state = 'running', error = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = ?1",
            [skill_id],
        )?;
        conn.execute(
            "UPDATE skill_security_migration
             SET state = 'running', current_skill_id = ?1, last_error = NULL,
                 started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
                 updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [skill_id],
        )?;
        Ok(())
    }

    pub fn record_migration_scan(
        conn: &Connection,
        skill_id: &str,
        error: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "UPDATE skill_security_migration_items
             SET state = CASE WHEN ?1 IS NULL THEN 'completed' ELSE 'failed' END,
                 error = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE skill_id = ?2",
            params![error, skill_id],
        )?;
        conn.execute(
            "UPDATE skill_security_migration
             SET total = (SELECT COUNT(*) FROM skill_security_migration_items),
                 completed = (SELECT COUNT(*) FROM skill_security_migration_items
                              WHERE state IN ('completed', 'failed')),
                 failed = (SELECT COUNT(*) FROM skill_security_migration_items
                           WHERE state = 'failed'),
                 last_error = ?1,
                 current_skill_id = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [error],
        )?;
        Ok(())
    }

    pub fn finish_migration_scan(conn: &Connection) -> Result<SkillMigrationStatus> {
        conn.execute(
            "UPDATE skill_security_migration
             SET total = (SELECT COUNT(*) FROM skill_security_migration_items),
                 completed = (SELECT COUNT(*) FROM skill_security_migration_items
                              WHERE state IN ('completed', 'failed')),
                 failed = (SELECT COUNT(*) FROM skill_security_migration_items
                           WHERE state = 'failed'),
                 state = CASE
                    WHEN (SELECT COUNT(*) FROM skill_security_migration_items
                          WHERE state = 'failed') > 0
                    THEN 'completed_with_errors' ELSE 'completed' END,
                 current_skill_id = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1
               AND NOT EXISTS (
                   SELECT 1 FROM skill_security_migration_items
                   WHERE state IN ('pending', 'running')
               )",
            [],
        )?;
        Self::migration_status(conn)
    }

    pub fn recover_migration_scan(conn: &Connection) -> Result<SkillMigrationStatus> {
        conn.execute(
            "UPDATE skill_security_migration_items
             SET state = 'pending', updated_at = CURRENT_TIMESTAMP
             WHERE state = 'running'",
            [],
        )?;
        conn.execute(
            "UPDATE skill_security_migration
             SET total = (SELECT COUNT(*) FROM skill_security_migration_items),
                 completed = (SELECT COUNT(*) FROM skill_security_migration_items
                              WHERE state IN ('completed', 'failed')),
                 failed = (SELECT COUNT(*) FROM skill_security_migration_items
                           WHERE state = 'failed'),
                 state = CASE
                    WHEN (SELECT COUNT(*) FROM skill_security_migration_items
                          WHERE state = 'pending') = 0
                    THEN CASE
                        WHEN (SELECT COUNT(*) FROM skill_security_migration_items
                              WHERE state = 'failed') > 0
                        THEN 'completed_with_errors' ELSE 'completed' END
                    ELSE 'pending'
                 END,
                 current_skill_id = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [],
        )?;
        Self::migration_status(conn)
    }

    pub fn retry_failed_migration_scans(conn: &Connection) -> Result<SkillMigrationStatus> {
        conn.execute(
            "UPDATE skill_security_migration_items
             SET state = 'pending', error = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE state = 'failed'",
            [],
        )?;
        conn.execute(
            "UPDATE skill_security_migration
             SET total = (SELECT COUNT(*) FROM skill_security_migration_items),
                 state = CASE WHEN failed > 0 THEN 'pending' ELSE state END,
                 completed = (SELECT COUNT(*) FROM skill_security_migration_items
                              WHERE state = 'completed'),
                 failed = 0, last_error = NULL, current_skill_id = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE singleton = 1",
            [],
        )?;
        Self::migration_status(conn)
    }

    fn decorate_conflicts(records: &mut [SkillRecord]) {
        let mut counts = HashMap::<String, usize>::new();
        for record in records.iter() {
            *counts.entry(record.slug.clone()).or_default() += 1;
        }
        let mut winners = HashSet::new();
        for record in records.iter_mut() {
            record.conflict = counts.get(&record.slug).copied().unwrap_or_default() > 1;
            let eligible = record.enabled
                && record.health == "healthy"
                && matches!(
                    record.security_state.as_str(),
                    "passed" | "warnings" | "approved"
                );
            record.effective_active = eligible && winners.insert(record.slug.clone());
            if record.conflict && eligible && !record.effective_active {
                record.disabled_reason = Some("source_conflict".to_string());
            }
        }
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillRecord> {
        let is_managed: bool = row.get(12)?;
        Ok(SkillRecord {
            skill_id: row.get(0)?,
            slug: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            version: row.get(4)?,
            source_kind: row.get(5)?,
            source_ref: row.get(6)?,
            source_url: row.get(7)?,
            checksum: row.get(8)?,
            installed_path: row.get(9)?,
            enabled: row.get(10)?,
            health: row.get(11)?,
            is_external: !is_managed,
            effective_active: false,
            effective_rank: row.get(14)?,
            conflict: false,
            disabled_reason: row.get(15)?,
            security_state: row.get(13)?,
            installed_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
    }
}

fn normalized_source_kind(record: &SkillRecord, is_managed: bool) -> String {
    if is_managed {
        "managed".to_string()
    } else {
        record.source_kind.clone()
    }
}

pub fn source_rank(source_kind: &str, is_managed: bool) -> i32 {
    if is_managed || source_kind == "managed" {
        return 400;
    }
    match source_kind {
        "codex" => 300,
        "claude" => 200,
        "cursor" => 100,
        _ => 50,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use rusqlite::Connection;

    use crate::db::migrations::run_migrations;
    use crate::services::skills::types::SkillRecord;

    use super::SkillSourceRepo;

    #[test]
    fn external_identity_and_user_state_survive_rediscovery() {
        let conn = database();
        let first = record("shared", "codex", "C:/codex/shared", true);
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &first, &first.installed_path, false).unwrap();
        let discovered = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert!(!discovered.enabled, "discovery must never imply activation");
        assert_eq!(discovered.security_state, "unscanned");

        SkillSourceRepo::set_enabled(&conn, &skill_id, true).unwrap();
        let mut changed = first;
        changed.description = "Updated description".to_string();
        let (rediscovered_id, _) =
            SkillSourceRepo::upsert_source(&conn, &changed, &changed.installed_path, false)
                .unwrap();
        let rediscovered = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert_eq!(rediscovered_id, skill_id);
        assert!(rediscovered.enabled);
        assert_eq!(rediscovered.security_state, "unscanned");
        assert_eq!(rediscovered.description, "Updated description");
    }

    #[test]
    fn conflict_resolution_activates_only_the_highest_ranked_source() {
        let conn = database();
        let managed = record("shared", "local", "C:/managed/shared", true);
        let mut codex = record("shared", "codex", "C:/codex/shared", true);
        codex.security_state = "passed".to_string();
        let (managed_id, _) =
            SkillSourceRepo::upsert_source(&conn, &managed, &managed.installed_path, true).unwrap();
        let (codex_id, _) =
            SkillSourceRepo::upsert_source(&conn, &codex, &codex.installed_path, false).unwrap();
        SkillSourceRepo::set_enabled(&conn, &codex_id, true).unwrap();

        let records = SkillSourceRepo::list(&conn).unwrap();
        let active = records
            .iter()
            .filter(|record| record.effective_active)
            .collect::<Vec<_>>();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].skill_id, managed_id);
        assert!(records.iter().all(|record| record.conflict));
        assert_eq!(
            records
                .iter()
                .find(|record| record.skill_id == codex_id)
                .unwrap()
                .disabled_reason
                .as_deref(),
            Some("source_conflict")
        );
    }

    #[test]
    fn generation_changes_are_monotonic_and_stale_values_are_observable() {
        let conn = database();
        let source = record("demo", "local", "C:/managed/demo", true);
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &source, &source.installed_path, true).unwrap();
        let before = SkillSourceRepo::generation(&conn).unwrap();
        let after = SkillSourceRepo::set_enabled(&conn, &skill_id, false).unwrap();
        assert!(after > before);
        assert_ne!(before, SkillSourceRepo::generation(&conn).unwrap());
    }

    #[test]
    fn migration_scan_progress_recovers_and_retries_failed_items() {
        let conn = database();
        let source = record("migrated", "codex", "C:/codex/migrated", true);
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &source, &source.installed_path, false).unwrap();

        let enrolled = SkillSourceRepo::enroll_unscanned_migration_sources(&conn).unwrap();
        assert_eq!(enrolled.total, 1);
        assert_eq!(enrolled.state, "pending");
        assert!(
            !SkillSourceRepo::find(&conn, &skill_id)
                .unwrap()
                .unwrap()
                .enabled
        );

        SkillSourceRepo::start_migration_scan(&conn, &skill_id).unwrap();
        let recovered = SkillSourceRepo::recover_migration_scan(&conn).unwrap();
        assert_eq!(recovered.state, "pending");
        assert_eq!(
            SkillSourceRepo::pending_migration_sources(&conn)
                .unwrap()
                .len(),
            1
        );

        SkillSourceRepo::start_migration_scan(&conn, &skill_id).unwrap();
        SkillSourceRepo::record_migration_scan(&conn, &skill_id, Some("fixture failure")).unwrap();
        let failed = SkillSourceRepo::finish_migration_scan(&conn).unwrap();
        assert_eq!(failed.state, "completed_with_errors");
        assert_eq!(failed.failed, 1);

        let retrying = SkillSourceRepo::retry_failed_migration_scans(&conn).unwrap();
        assert_eq!(retrying.state, "pending");
        assert_eq!(retrying.completed, 0);
        SkillSourceRepo::start_migration_scan(&conn, &skill_id).unwrap();
        SkillSourceRepo::record_migration_scan(&conn, &skill_id, None).unwrap();
        let completed = SkillSourceRepo::finish_migration_scan(&conn).unwrap();
        assert_eq!(completed.state, "completed");
        assert_eq!(completed.failed, 0);
    }

    #[test]
    fn migration_progress_reconciles_when_a_source_is_deleted_mid_scan() {
        let conn = database();
        let source = record("removed", "codex", "C:/codex/removed", true);
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &source, &source.installed_path, false).unwrap();

        let enrolled = SkillSourceRepo::enroll_unscanned_migration_sources(&conn).unwrap();
        assert_eq!(enrolled.total, 1);
        SkillSourceRepo::start_migration_scan(&conn, &skill_id).unwrap();
        SkillSourceRepo::delete_source(&conn, &skill_id).unwrap();
        SkillSourceRepo::record_migration_scan(&conn, &skill_id, None).unwrap();

        let completed = SkillSourceRepo::finish_migration_scan(&conn).unwrap();
        assert_eq!(completed.state, "completed");
        assert_eq!(completed.total, 0);
        assert_eq!(completed.completed, 0);
        assert_eq!(completed.failed, 0);
    }

    #[test]
    fn artifact_change_invalidates_previous_activation() {
        let conn = database();
        let source = record("demo", "codex", "C:/codex/demo", true);
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &source, &source.installed_path, false).unwrap();
        SkillSourceRepo::set_enabled(&conn, &skill_id, true).unwrap();

        let mut changed = source;
        changed.checksum = "different-artifact-hash".to_string();
        SkillSourceRepo::upsert_source(&conn, &changed, &changed.installed_path, false).unwrap();
        let invalidated = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert!(!invalidated.enabled);
        assert_eq!(invalidated.security_state, "stale");
        assert_eq!(
            invalidated.disabled_reason.as_deref(),
            Some("artifact_changed")
        );
    }

    #[test]
    fn external_delete_rename_and_restart_preserve_registry_history() {
        let temporary = tempfile::tempdir().unwrap();
        let db_path = temporary.path().join("registry.db");
        let original_dir = temporary.path().join("codex").join("demo");
        fs::create_dir_all(&original_dir).unwrap();
        fs::write(original_dir.join("SKILL.md"), "original").unwrap();
        let original_path = original_dir.display().to_string();

        let conn = Connection::open(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        let original = record("demo", "codex", &original_path, true);
        let (original_id, _) =
            SkillSourceRepo::upsert_source(&conn, &original, &original_path, false).unwrap();
        SkillSourceRepo::set_enabled(&conn, &original_id, true).unwrap();

        SkillSourceRepo::mark_missing_external_except(&conn, &HashSet::new()).unwrap();
        let missing = SkillSourceRepo::find(&conn, &original_id).unwrap().unwrap();
        assert_eq!(missing.health, "missing");
        assert!(!missing.enabled);
        assert_eq!(
            fs::read_to_string(original_dir.join("SKILL.md")).unwrap(),
            "original"
        );

        let renamed_path = temporary
            .path()
            .join("codex")
            .join("demo-renamed")
            .display()
            .to_string();
        let renamed = record("demo", "codex", &renamed_path, true);
        let (renamed_id, _) =
            SkillSourceRepo::upsert_source(&conn, &renamed, &renamed_path, false).unwrap();
        assert_ne!(renamed_id, original_id);
        drop(conn);

        let reopened = Connection::open(&db_path).unwrap();
        assert!(SkillSourceRepo::find(&reopened, &original_id)
            .unwrap()
            .is_some());
        assert!(SkillSourceRepo::find(&reopened, &renamed_id)
            .unwrap()
            .is_some());
    }

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn record(slug: &str, source: &str, path: &str, enabled: bool) -> SkillRecord {
        SkillRecord {
            skill_id: String::new(),
            slug: slug.to_string(),
            name: slug.to_string(),
            description: "Description".to_string(),
            version: None,
            source_kind: source.to_string(),
            source_ref: None,
            source_url: None,
            checksum: "artifact-hash".to_string(),
            installed_path: path.to_string(),
            enabled,
            health: "healthy".to_string(),
            is_external: source != "local",
            effective_active: false,
            effective_rank: 0,
            conflict: false,
            disabled_reason: None,
            security_state: if source == "local" {
                "passed".to_string()
            } else {
                "unscanned".to_string()
            },
            installed_at: String::new(),
            updated_at: String::new(),
        }
    }
}
