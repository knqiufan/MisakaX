use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::config;
use crate::db::repository::{SkillRepo, SkillSourceRepo};

use super::installer::discover_external_skills;
use super::types::{MessageSkillSelection, SkillActivationMount, SkillActivationView, SkillRecord};

/// A single activation gate shared by enable, message selection, and Sidecar
/// mount construction. S3 extends this gate with scanner decisions; S1 never
/// labels legacy/user acknowledgement as a successful scan.
pub struct SkillSecurityGate;

impl SkillSecurityGate {
    pub fn ensure_enableable(conn: &Connection, record: &SkillRecord) -> Result<()> {
        if record.health != "healthy" {
            bail!("Skill '{}' is missing or corrupted", record.slug);
        }
        if !Path::new(&record.installed_path).join("SKILL.md").is_file() {
            bail!("Skill '{}' no longer contains SKILL.md", record.slug);
        }
        if record.security_state != "legacy_allowed" {
            super::security::ensure_scan_allows(conn, record)?;
        }
        Ok(())
    }

    pub fn ensure_effective(conn: &Connection, record: &SkillRecord) -> Result<()> {
        Self::ensure_enableable(conn, record)?;
        if !record.enabled {
            bail!("Skill '{}' is disabled", record.slug);
        }
        if !matches!(
            record.security_state.as_str(),
            "legacy_allowed" | "passed" | "warnings" | "approved"
        ) {
            bail!("Skill '{}' is awaiting security approval", record.slug);
        }
        if !record.effective_active {
            bail!(
                "Skill '{}' is shadowed by a higher-ranked source",
                record.slug
            );
        }
        Ok(())
    }
}

pub fn sync_inventory(conn: &Connection) -> Result<Vec<SkillRecord>> {
    crate::db::repository::SkillSecurityRepo::expire_approvals(conn)?;
    let managed_root = config::skills_dir()?;
    for mut record in SkillRepo::list(conn)? {
        let expected = managed_root.join(&record.slug);
        record.installed_path = expected.display().to_string();
        record.health = if expected.join("SKILL.md").is_file() {
            "healthy".to_string()
        } else {
            "missing".to_string()
        };
        if record.health == "healthy" {
            record.checksum = artifact_hash(&expected)?;
        }
        SkillSourceRepo::upsert_source(conn, &record, &record.installed_path, true)?;
    }

    let external = discover_external_skills()?;
    let present = external
        .iter()
        .map(|record| record.installed_path.clone())
        .collect::<HashSet<_>>();
    for mut record in external {
        record.checksum = artifact_hash(Path::new(&record.installed_path))?;
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(conn, &record, &record.installed_path, false)?;
        record.skill_id = skill_id;
        super::security::ensure_external_scan(conn, &record)?;
    }
    SkillSourceRepo::mark_missing_external_except(conn, &present)?;
    SkillSourceRepo::list(conn)
}

pub fn set_enabled(conn: &Connection, identifier: &str, enabled: bool) -> Result<(String, u64)> {
    sync_inventory(conn)?;
    let record = SkillSourceRepo::find_id_or_legacy_slug(conn, identifier)?
        .context("Skill source is not registered")?;
    if enabled {
        SkillSecurityGate::ensure_enableable(conn, &record)?;
    }
    let generation = SkillSourceRepo::set_enabled(conn, &record.skill_id, enabled)?;
    Ok((record.skill_id, generation))
}

pub fn activation_view(
    conn: &Connection,
    expected_generation: Option<u64>,
) -> Result<SkillActivationView> {
    sync_inventory(conn)?;
    activation_view_from_registry(conn, expected_generation)
}

fn activation_view_from_registry(
    conn: &Connection,
    expected_generation: Option<u64>,
) -> Result<SkillActivationView> {
    let records = SkillSourceRepo::list(conn)?;
    let generation = SkillSourceRepo::generation(conn)?;
    if let Some(expected) = expected_generation {
        if expected != generation {
            bail!("Stale Skill activation generation: expected {expected}, current {generation}");
        }
    }
    let skills = records
        .iter()
        .filter(|record| record.effective_active)
        .map(|record| {
            SkillSecurityGate::ensure_effective(conn, record)?;
            Ok(SkillActivationMount {
                skill_id: record.skill_id.clone(),
                slug: record.slug.clone(),
                path: record.installed_path.clone(),
                artifact_hash: record.checksum.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(SkillActivationView { generation, skills })
}

pub fn resolve_selection(
    conn: &Connection,
    identifiers: &[String],
    expected_generation: Option<u64>,
) -> Result<(SkillActivationView, Vec<MessageSkillSelection>)> {
    if identifiers.len() > 20 {
        bail!("At most 20 Skills can be selected for one message");
    }
    let view = activation_view(conn, expected_generation)?;
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for identifier in identifiers {
        let raw = identifier.trim();
        if raw.is_empty() {
            bail!("Skill identifier cannot be empty");
        }
        let record = SkillSourceRepo::find_id_or_legacy_slug(conn, raw)?
            .with_context(|| format!("Selected Skill '{raw}' is not registered"))?;
        SkillSecurityGate::ensure_effective(conn, &record)?;
        let mount = view
            .skills
            .iter()
            .find(|mount| mount.skill_id == record.skill_id)
            .with_context(|| format!("Selected Skill '{}' is not active", record.slug))?;
        if mount.artifact_hash != record.checksum {
            bail!("Selected Skill '{}' changed during activation", record.slug);
        }
        if seen.insert(record.skill_id.clone()) {
            selected.push(MessageSkillSelection {
                skill_id: record.skill_id,
                slug_snapshot: record.slug,
                artifact_hash_snapshot: record.checksum,
            });
        }
    }
    Ok((view, selected))
}

pub fn resolve_snapshot_selection(
    conn: &Connection,
    snapshots: &[MessageSkillSelection],
) -> Result<(SkillActivationView, Vec<MessageSkillSelection>)> {
    let ids = snapshots
        .iter()
        .map(|selection| selection.skill_id.clone())
        .collect::<Vec<_>>();
    let (view, current) = resolve_selection(conn, &ids, None)?;
    for snapshot in snapshots {
        let active = current
            .iter()
            .find(|item| item.skill_id == snapshot.skill_id)
            .context("Previously selected Skill is no longer active")?;
        if active.artifact_hash_snapshot != snapshot.artifact_hash_snapshot {
            bail!(
                "Previously selected Skill '{}' changed after the message was sent",
                snapshot.slug_snapshot
            );
        }
    }
    Ok((view, current))
}

pub fn artifact_hash(root: &Path) -> Result<String> {
    let canonical = fs::canonicalize(root)
        .with_context(|| format!("Cannot resolve Skill directory {}", root.display()))?;
    let mut paths = WalkDir::new(&canonical)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect::<Vec<PathBuf>>();
    paths.sort();
    let mut hasher = Sha256::new();
    for path in paths {
        let relative = path.strip_prefix(&canonical)?;
        hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(&path)
                .with_context(|| format!("Cannot hash Skill artifact file {}", path.display()))?,
        );
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::Connection;

    use crate::db::migrations::run_migrations;
    use crate::db::repository::{SkillSecurityRepo, SkillSourceRepo};
    use crate::services::skills::types::{SkillRecord, SkillRiskReport};

    use super::{activation_view_from_registry, artifact_hash, SkillSecurityGate};

    #[test]
    fn stale_activation_generation_is_rejected_before_mounting() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let temporary = tempfile::tempdir().unwrap();
        fs::write(
            temporary.path().join("SKILL.md"),
            "---\nname: demo\ndescription: Demo\n---\n",
        )
        .unwrap();
        let record = SkillRecord {
            skill_id: String::new(),
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            description: "Demo".to_string(),
            version: None,
            source_kind: "local".to_string(),
            source_ref: None,
            source_url: None,
            checksum: artifact_hash(temporary.path()).unwrap(),
            installed_path: temporary.path().display().to_string(),
            enabled: true,
            health: "healthy".to_string(),
            is_external: false,
            effective_active: false,
            effective_rank: 400,
            conflict: false,
            disabled_reason: None,
            security_state: "legacy_allowed".to_string(),
            risk: SkillRiskReport::default(),
            installed_at: String::new(),
            updated_at: String::new(),
        };
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &record, &record.installed_path, true).unwrap();
        let stale = SkillSourceRepo::generation(&conn).unwrap();
        SkillSourceRepo::set_enabled(&conn, &skill_id, false).unwrap();

        let error = activation_view_from_registry(&conn, Some(stale)).unwrap_err();
        assert!(error
            .to_string()
            .contains("Stale Skill activation generation"));
    }

    #[test]
    fn scan_gate_rejects_unscanned_and_stale_artifacts_without_a_legacy_bypass() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let temporary = tempfile::tempdir().unwrap();
        fs::write(
            temporary.path().join("SKILL.md"),
            "---\nname: scanned-skill\ndescription: Demo\n---\n",
        )
        .unwrap();
        let hash = artifact_hash(temporary.path()).unwrap();
        let record = SkillRecord {
            skill_id: String::new(),
            slug: "scanned-skill".to_string(),
            name: "Scanned".to_string(),
            description: "Demo".to_string(),
            version: None,
            source_kind: "managed".to_string(),
            source_ref: None,
            source_url: None,
            checksum: hash.clone(),
            installed_path: temporary.path().display().to_string(),
            enabled: false,
            health: "healthy".to_string(),
            is_external: false,
            effective_active: false,
            effective_rank: 400,
            conflict: false,
            disabled_reason: Some("unscanned".to_string()),
            security_state: "unscanned".to_string(),
            risk: SkillRiskReport::default(),
            installed_at: String::new(),
            updated_at: String::new(),
        };
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&conn, &record, &record.installed_path, true).unwrap();
        let unscanned = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert!(SkillSecurityGate::ensure_enableable(&conn, &unscanned).is_err());

        SkillSecurityRepo::upsert_artifact(&conn, &hash, 1, "{}", None, "approved").unwrap();
        SkillSecurityRepo::create_scan(
            &conn,
            "scan-gate",
            &hash,
            &serde_json::json!({
                "builtin": super::super::security::scanner::ENGINE_VERSION
            })
            .to_string(),
            super::super::security::policy::POLICY_VERSION,
            "corr",
        )
        .unwrap();
        SkillSecurityRepo::begin_scan(&conn, "scan-gate").unwrap();
        SkillSecurityRepo::complete_scan(&conn, "scan-gate", "passed", "allow", None, &[]).unwrap();
        SkillSecurityRepo::attach_scan_to_source(&conn, &skill_id, "scan-gate", "passed").unwrap();
        let passed = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        SkillSecurityGate::ensure_enableable(&conn, &passed).unwrap();

        let mut changed = passed.clone();
        changed.checksum = "changed".to_string();
        assert!(SkillSecurityGate::ensure_enableable(&conn, &changed)
            .unwrap_err()
            .to_string()
            .contains("SKILL_SCAN_STALE"));

        conn.execute(
            "UPDATE skill_scan_runs
             SET state = 'passed', decision = 'allow', engine_versions_json = '{\"builtin\":\"old\"}'
             WHERE scan_id = 'scan-gate'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE skill_sources SET security_state = 'passed' WHERE skill_id = ?1",
            [&skill_id],
        )
        .unwrap();
        let version_stale = SkillSourceRepo::find(&conn, &skill_id).unwrap().unwrap();
        assert!(SkillSecurityGate::ensure_enableable(&conn, &version_stale)
            .unwrap_err()
            .to_string()
            .contains("SKILL_SCAN_STALE"));
        assert_eq!(
            SkillSourceRepo::find(&conn, &skill_id)
                .unwrap()
                .unwrap()
                .security_state,
            "stale"
        );
    }
}
