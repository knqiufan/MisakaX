#[cfg(test)]
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::config;
use crate::db::repository::SkillSourceRepo;

use super::manifest::parse_manifest;
use super::security::ApprovedArtifactId;
use super::types::{ArchiveInspection, InstallSource, SkillInstallResult, SkillRecord};

pub fn install_local_archive(
    conn: &Connection,
    archive_path: &Path,
    source: InstallSource,
) -> Result<SkillInstallResult> {
    ensure_zip_file(archive_path)?;
    super::security::install_archive_compat(conn, archive_path, source)
}

pub(crate) fn install_approved_artifact(
    conn: &Connection,
    approved: ApprovedArtifactId,
    source: InstallSource,
    force_disabled: bool,
) -> Result<SkillInstallResult> {
    let (_scan_id, artifact_hash, extracted_path, inspection) = approved.into_parts();
    if inspection.checksum != artifact_hash {
        bail!("Approved artifact capability failed hash validation")
    }
    install_staged_skill(conn, extracted_path, inspection, source, force_disabled)
}

pub fn list_installed(conn: &Connection) -> Result<Vec<SkillRecord>> {
    super::registry::sync_inventory(conn)
}

pub fn set_enabled(conn: &Connection, identifier: &str, enabled: bool) -> Result<(String, u64)> {
    super::registry::set_enabled(conn, identifier, enabled)
}

pub fn uninstall(conn: &Connection, identifier: &str) -> Result<(String, u64)> {
    super::registry::sync_inventory(conn)?;
    let record = SkillSourceRepo::find(conn, identifier)?.context("Skill is not installed")?;
    if record.is_external {
        bail!("External Skill sources are never modified or removed by MisakaX")
    }
    let skill_dir = checked_skill_dir(&record)?;
    if skill_dir.exists() {
        fs::remove_dir_all(&skill_dir).context("Cannot remove installed skill files")?;
    }
    SkillSourceRepo::delete_source(conn, &record.skill_id)?;
    Ok((record.skill_id, SkillSourceRepo::generation(conn)?))
}

pub fn installed_selection(conn: &Connection, identifiers: &[String]) -> Result<Vec<SkillRecord>> {
    let (_, selections) = super::registry::resolve_selection(conn, identifiers, None)?;
    selections
        .iter()
        .map(|selection| {
            SkillSourceRepo::find(conn, &selection.skill_id)?
                .context("Selected Skill disappeared from the registry")
        })
        .collect()
}

fn install_staged_skill(
    conn: &Connection,
    stage: PathBuf,
    inspection: ArchiveInspection,
    source: InstallSource,
    force_disabled: bool,
) -> Result<SkillInstallResult> {
    let target = config::skills_dir()?.join(&inspection.manifest.name);
    let replaced_existing = target.exists();
    let backup = replace_target_atomically(&stage, &target)?;
    let record = build_record(&target, &inspection, source, force_disabled);
    let transaction = conn.unchecked_transaction()?;
    let persisted = (|| -> Result<SkillRecord> {
        let mut current = record.clone();
        current.checksum = super::registry::artifact_hash(&target)?;
        let (skill_id, _) =
            SkillSourceRepo::upsert_source(&transaction, &current, &current.installed_path, true)?;
        SkillSourceRepo::find(&transaction, &skill_id)?.context("Cannot read installed skill")
    })();
    let persisted = match persisted {
        Ok(persisted) => persisted,
        Err(error) => {
            drop(transaction);
            restore_backup(&target, backup.as_deref());
            return Err(error);
        }
    };
    if let Err(error) = transaction.commit() {
        restore_backup(&target, backup.as_deref());
        return Err(error.into());
    }
    if let Some(path) = backup {
        let _ = fs::remove_dir_all(path);
    }
    Ok(SkillInstallResult {
        skill: persisted,
        replaced_existing,
    })
}

fn ensure_zip_file(path: &Path) -> Result<()> {
    if !path.is_file() {
        bail!("Selected skill archive does not exist")
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("zip") {
        bail!("Only .zip skill archives are supported")
    }
    Ok(())
}

fn replace_target_atomically(stage: &Path, target: &Path) -> Result<Option<PathBuf>> {
    let target_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .context("Skill target directory is invalid")?;
    let backup = target.with_file_name(format!(".{target_name}.backup-{}", uuid::Uuid::new_v4()));
    let backup = target.exists().then_some(backup);
    if let Some(path) = &backup {
        fs::rename(target, path).context("Cannot preserve existing skill before replacement")?;
    }
    if let Err(error) = fs::rename(stage, target) {
        if let Some(path) = &backup {
            let _ = fs::rename(path, target);
        }
        return Err(error).context("Cannot activate staged skill");
    }
    Ok(backup)
}

fn restore_backup(target: &Path, backup: Option<&Path>) {
    let _ = fs::remove_dir_all(target);
    if let Some(path) = backup {
        let _ = fs::rename(path, target);
    }
}

fn build_record(
    target: &Path,
    inspection: &ArchiveInspection,
    source: InstallSource,
    force_disabled: bool,
) -> SkillRecord {
    SkillRecord {
        skill_id: String::new(),
        slug: inspection.manifest.name.clone(),
        name: inspection.manifest.name.clone(),
        description: inspection.manifest.description.clone(),
        version: source.version,
        source_kind: source.kind,
        source_ref: source.reference,
        source_url: source.url,
        checksum: inspection.checksum.clone(),
        installed_path: target.display().to_string(),
        enabled: !force_disabled,
        health: "healthy".to_string(),
        is_external: false,
        effective_active: false,
        effective_rank: 400,
        conflict: false,
        disabled_reason: force_disabled.then(|| "review_approved".to_string()),
        security_state: "unscanned".to_string(),
        installed_at: String::new(),
        updated_at: String::new(),
    }
}

fn checked_skill_dir(record: &SkillRecord) -> Result<PathBuf> {
    let root = config::skills_dir()?;
    let expected = root.join(&record.slug);
    if Path::new(&record.installed_path) != expected {
        bail!("Skill installation path is outside the managed skills directory")
    }
    Ok(expected)
}

const EXTERNAL_SKILL_LIMIT: usize = 250;
const EXTERNAL_SKILL_MAX_DEPTH: usize = 4;

/// Discover Skills in the standard user-level directories of other Agents.
///
/// Only frontmatter and file metadata are read at inventory time. Full Skill
/// instructions remain on disk until a user explicitly selects the Skill for a
/// turn. A deterministic precedence order avoids duplicate names appearing in
/// the UI: Codex, Claude, then Cursor.
pub fn discover_external_skills() -> Result<Vec<SkillRecord>> {
    let home = dirs::home_dir().context("Failed to resolve the home directory")?;
    let roots = [
        ("codex", home.join(".codex").join("skills")),
        ("claude", home.join(".claude").join("skills")),
        ("cursor", home.join(".cursor").join("skills")),
    ];
    discover_external_from_roots(&roots)
}

fn discover_external_from_roots(roots: &[(&str, PathBuf)]) -> Result<Vec<SkillRecord>> {
    let mut found = Vec::new();
    for (source, root) in roots {
        if found.len() >= EXTERNAL_SKILL_LIMIT {
            break;
        }
        found.extend(discover_from_root(
            source,
            root,
            EXTERNAL_SKILL_LIMIT - found.len(),
        )?);
    }
    Ok(found)
}

#[cfg(test)]
fn merge_installed_sources(
    managed: Vec<SkillRecord>,
    external: Vec<SkillRecord>,
) -> Vec<SkillRecord> {
    let managed_slugs = managed
        .iter()
        .map(|skill| skill.slug.clone())
        .collect::<HashSet<_>>();
    let mut skills = managed;
    skills.extend(
        external
            .into_iter()
            .filter(|skill| !managed_slugs.contains(&skill.slug)),
    );
    skills
}

fn discover_from_root(source: &str, root: &Path, remaining: usize) -> Result<Vec<SkillRecord>> {
    if remaining == 0 || !root.is_dir() {
        return Ok(Vec::new());
    }
    let canonical_root = fs::canonicalize(root)
        .with_context(|| format!("Cannot resolve external Skill root {}", root.display()))?;
    let mut found = Vec::new();
    for entry in WalkDir::new(&canonical_root)
        .follow_links(false)
        .min_depth(2)
        .max_depth(EXTERNAL_SKILL_MAX_DEPTH)
    {
        let entry = entry.context("Cannot enumerate external Skills")?;
        if !entry.file_type().is_file() || entry.file_name() != "SKILL.md" {
            continue;
        }
        let Some(directory) = entry.path().parent() else {
            continue;
        };
        let canonical_dir = match fs::canonicalize(directory) {
            Ok(path) if path.starts_with(&canonical_root) => path,
            _ => continue,
        };
        let markdown = match fs::read_to_string(canonical_dir.join("SKILL.md")) {
            Ok(markdown) => markdown,
            Err(_) => continue,
        };
        let manifest = match parse_manifest(&markdown) {
            Ok(manifest) => manifest,
            Err(_) => continue,
        };
        let Some(directory_name) = canonical_dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if super::manifest::validate_skill_directory(directory_name, &manifest).is_err() {
            continue;
        }
        found.push(external_record(source, &canonical_dir, manifest, &markdown));
        if found.len() >= remaining {
            break;
        }
    }
    Ok(found)
}

fn external_record(
    source: &str,
    directory: &Path,
    manifest: super::types::SkillManifest,
    markdown: &str,
) -> SkillRecord {
    let mut hasher = Sha256::new();
    hasher.update(markdown.as_bytes());
    let checksum = format!("{:x}", hasher.finalize());
    SkillRecord {
        skill_id: String::new(),
        slug: manifest.name.clone(),
        name: manifest.name,
        description: manifest.description,
        version: manifest
            .metadata
            .get("version")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string),
        source_kind: source.to_string(),
        source_ref: Some("external".to_string()),
        source_url: None,
        checksum,
        installed_path: directory.display().to_string(),
        enabled: false,
        health: "healthy".to_string(),
        is_external: true,
        effective_active: false,
        effective_rank: crate::db::repository::skill_source_repo::source_rank(source, false),
        conflict: false,
        disabled_reason: Some("unscanned".to_string()),
        security_state: "unscanned".to_string(),
        installed_at: String::new(),
        updated_at: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::services::skills::types::SkillRecord;

    use super::{discover_external_from_roots, discover_from_root, merge_installed_sources};

    #[test]
    fn discovers_only_valid_external_skill_directories() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("skills");
        let valid = root.join("example-skill");
        let invalid = root.join("different-directory");
        fs::create_dir_all(&valid).unwrap();
        fs::create_dir_all(&invalid).unwrap();
        fs::write(
            valid.join("SKILL.md"),
            "---\nname: example-skill\ndescription: A valid external skill\n---\n",
        )
        .unwrap();
        fs::write(
            invalid.join("SKILL.md"),
            "---\nname: another-skill\ndescription: This must not be indexed\n---\n",
        )
        .unwrap();

        let found = discover_from_root("claude", &root, 10).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].slug, "example-skill");
        assert!(found[0].is_external);
        assert_eq!(found[0].source_kind, "claude");
        assert_eq!(
            found[0].installed_path,
            fs::canonicalize(valid).unwrap().display().to_string()
        );
    }

    #[test]
    fn external_discovery_uses_codex_claude_cursor_precedence() {
        let temporary = tempfile::tempdir().unwrap();
        let roots = [
            ("codex", temporary.path().join("codex")),
            ("claude", temporary.path().join("claude")),
            ("cursor", temporary.path().join("cursor")),
        ];
        for (_, root) in &roots {
            write_skill(root, "shared-skill", "Shared Skill");
        }

        let found = discover_external_from_roots(&roots).unwrap();

        assert_eq!(found.len(), 3);
        assert_eq!(found[0].source_kind, "codex");
        assert_eq!(found[1].source_kind, "claude");
        assert_eq!(found[2].source_kind, "cursor");
    }

    #[test]
    fn managed_inventory_wins_same_slug_source_conflicts() {
        let managed = record("shared-skill", "managed", true, Path::new("managed"));
        let mut external = record("shared-skill", "codex", true, Path::new("external"));
        external.is_external = true;

        let merged = merge_installed_sources(vec![managed], vec![external]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source_kind, "managed");
    }

    fn write_skill(root: &Path, slug: &str, description: &str) {
        let directory = root.join(slug);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("SKILL.md"),
            format!("---\nname: {slug}\ndescription: {description}\n---\n"),
        )
        .unwrap();
    }

    fn record(slug: &str, source: &str, enabled: bool, installed_path: &Path) -> SkillRecord {
        SkillRecord {
            skill_id: String::new(),
            slug: slug.to_string(),
            name: slug.to_string(),
            description: "Skill description".to_string(),
            version: Some("1.0.0".to_string()),
            source_kind: source.to_string(),
            source_ref: None,
            source_url: None,
            checksum: "checksum".to_string(),
            installed_path: installed_path.display().to_string(),
            enabled,
            health: "healthy".to_string(),
            is_external: source != "managed" && source != "local",
            effective_active: enabled,
            effective_rank: if source == "managed" || source == "local" {
                400
            } else {
                100
            },
            conflict: false,
            disabled_reason: None,
            security_state: "passed".to_string(),
            installed_at: String::new(),
            updated_at: String::new(),
        }
    }
}
