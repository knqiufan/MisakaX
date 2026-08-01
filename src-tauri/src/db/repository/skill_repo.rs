use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::services::skills::types::MessageSkillSelection;
use crate::services::skills::types::{SkillRecord, SkillRiskReport};

use super::SkillSourceRepo;

pub struct SkillRepo;

impl SkillRepo {
    pub fn list(conn: &Connection) -> Result<Vec<SkillRecord>> {
        let mut statement = conn.prepare(
            "SELECT slug, name, description, version, source_kind, source_ref, source_url,
                    checksum, installed_path, enabled, health, risk_json, installed_at, updated_at
             FROM skills ORDER BY updated_at DESC, name COLLATE NOCASE",
        )?;
        let rows = statement.query_map([], Self::map_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn find(conn: &Connection, slug: &str) -> Result<Option<SkillRecord>> {
        let mut statement = conn.prepare(
            "SELECT slug, name, description, version, source_kind, source_ref, source_url,
                    checksum, installed_path, enabled, health, risk_json, installed_at, updated_at
             FROM skills WHERE slug = ?1",
        )?;
        let mut rows = statement.query([slug])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn upsert(conn: &Connection, skill: &SkillRecord) -> Result<()> {
        let risk_json =
            serde_json::to_string(&skill.risk).context("Cannot serialize skill risk")?;
        conn.execute(
            "INSERT INTO skills (
                slug, name, description, version, source_kind, source_ref, source_url,
                checksum, installed_path, enabled, health, risk_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(slug) DO UPDATE SET
                name = excluded.name, description = excluded.description, version = excluded.version,
                source_kind = excluded.source_kind, source_ref = excluded.source_ref,
                source_url = excluded.source_url, checksum = excluded.checksum,
                installed_path = excluded.installed_path, enabled = excluded.enabled,
                health = excluded.health, risk_json = excluded.risk_json,
                updated_at = CURRENT_TIMESTAMP",
            rusqlite::params![
                skill.slug,
                skill.name,
                skill.description,
                skill.version,
                skill.source_kind,
                skill.source_ref,
                skill.source_url,
                skill.checksum,
                skill.installed_path,
                skill.enabled,
                skill.health,
                risk_json,
            ],
        )
        .context("Cannot save skill inventory record")?;
        Ok(())
    }

    pub fn set_enabled(conn: &Connection, slug: &str, enabled: bool) -> Result<()> {
        conn.execute(
            "UPDATE skills SET enabled = ?1, updated_at = CURRENT_TIMESTAMP WHERE slug = ?2",
            rusqlite::params![enabled, slug],
        )
        .context("Cannot update skill status")?;
        Ok(())
    }

    pub fn delete(conn: &Connection, slug: &str) -> Result<()> {
        conn.execute("DELETE FROM skills WHERE slug = ?1", [slug])
            .context("Cannot delete skill inventory record")?;
        Ok(())
    }

    pub fn replace_message_selection(
        conn: &Connection,
        message_id: &str,
        slugs: &[String],
    ) -> Result<()> {
        let mut selections = Vec::with_capacity(slugs.len());
        for slug in slugs {
            let source = SkillSourceRepo::find_id_or_legacy_slug(conn, slug)?
                .with_context(|| format!("Skill selection '{slug}' is not registered"))?;
            selections.push(MessageSkillSelection {
                skill_id: source.skill_id,
                slug_snapshot: source.slug,
                artifact_hash_snapshot: source.checksum,
            });
        }
        SkillSourceRepo::replace_message_selection(conn, message_id, &selections)
    }

    pub fn message_selection(conn: &Connection, message_id: &str) -> Result<Vec<String>> {
        SkillSourceRepo::message_selection(conn, message_id).map(|items| {
            items
                .into_iter()
                .map(|selection| selection.skill_id)
                .collect()
        })
    }

    pub fn enabled_healthy(conn: &Connection, slugs: &[String]) -> Result<Vec<SkillRecord>> {
        let mut records = Vec::new();
        for slug in slugs {
            let Some(record) = Self::find(conn, slug)? else {
                anyhow::bail!("Selected skill '{}' is not installed", slug);
            };
            if !record.enabled || record.health != "healthy" {
                anyhow::bail!("Selected skill '{}' is not enabled and healthy", slug);
            }
            records.push(record);
        }
        Ok(records)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillRecord> {
        let risk_json: String = row.get(11)?;
        let risk = serde_json::from_str(&risk_json).unwrap_or_else(|_| SkillRiskReport::default());
        Ok(SkillRecord {
            skill_id: String::new(),
            slug: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            version: row.get(3)?,
            source_kind: row.get(4)?,
            source_ref: row.get(5)?,
            source_url: row.get(6)?,
            checksum: row.get(7)?,
            installed_path: row.get(8)?,
            enabled: row.get(9)?,
            health: row.get(10)?,
            is_external: false,
            effective_active: false,
            effective_rank: 400,
            conflict: false,
            disabled_reason: None,
            security_state: "legacy_allowed".to_string(),
            risk,
            installed_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }
}
