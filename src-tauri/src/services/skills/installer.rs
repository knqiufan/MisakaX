use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::config;
use crate::db::repository::SkillRepo;

use super::archive::{extract_archive, inspect_archive};
use super::manifest::parse_manifest;
use super::types::{
    ArchiveInspection, InstallSource, SkillDetail, SkillFileNode, SkillInstallResult, SkillRecord,
    SkillRiskReport,
};

pub fn install_local_archive(
    conn: &Connection,
    archive_path: &Path,
    source: InstallSource,
) -> Result<SkillInstallResult> {
    ensure_zip_file(archive_path)?;
    let inspection = inspect_archive(archive_path)?;
    let stage = prepare_staging_dir()?;
    let extract_result = extract_archive(archive_path, &stage);
    if let Err(error) = extract_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    let result = install_staged_skill(conn, stage.clone(), inspection, source);
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

pub fn list_installed(conn: &Connection) -> Result<Vec<SkillRecord>> {
    let root = config::skills_dir()?;
    let managed = SkillRepo::list(conn)?
        .into_iter()
        .map(|record| Ok(with_current_health(record, &root)))
        .collect::<Result<Vec<_>>>()?;
    let mut skills = merge_installed_sources(managed, discover_external_skills()?);
    skills.sort_by(|left, right| {
        left.is_external
            .cmp(&right.is_external)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(skills)
}

pub fn get_detail(conn: &Connection, slug: &str) -> Result<SkillDetail> {
    let record = match SkillRepo::find(conn, slug)? {
        Some(record) => record,
        None => find_external_skill(slug)?.context("Skill is not installed")?,
    };
    let skill_dir = skill_dir_for_record(&record)?;
    let markdown =
        fs::read_to_string(skill_dir.join("SKILL.md")).context("Cannot read SKILL.md")?;
    let manifest = parse_manifest(&markdown)?;
    let files = list_skill_files(&skill_dir)?;
    Ok(SkillDetail {
        skill: with_current_health(record, &config::skills_dir()?),
        manifest,
        files,
        skill_markdown: markdown,
    })
}

pub fn set_enabled(conn: &Connection, slug: &str, enabled: bool) -> Result<()> {
    let record = SkillRepo::find(conn, slug)?.context("Skill is not installed")?;
    if health_for_record(&record, &config::skills_dir()?) != "healthy" {
        bail!("Cannot enable a missing or corrupted skill")
    }
    SkillRepo::set_enabled(conn, slug, enabled)
}

pub fn uninstall(conn: &Connection, slug: &str) -> Result<()> {
    let record = SkillRepo::find(conn, slug)?.context("Skill is not installed")?;
    let skill_dir = checked_skill_dir(&record)?;
    if skill_dir.exists() {
        fs::remove_dir_all(&skill_dir).context("Cannot remove installed skill files")?;
    }
    SkillRepo::delete(conn, slug)
}

pub fn installed_selection(conn: &Connection, slugs: &[String]) -> Result<Vec<SkillRecord>> {
    let root = config::skills_dir()?;
    installed_selection_with(conn, slugs, &root, find_external_skill)
}

fn installed_selection_with(
    conn: &Connection,
    slugs: &[String],
    root: &Path,
    mut find_external: impl FnMut(&str) -> Result<Option<SkillRecord>>,
) -> Result<Vec<SkillRecord>> {
    let mut records = Vec::with_capacity(slugs.len());
    for slug in slugs {
        let record = match SkillRepo::find(conn, slug)? {
            Some(record) => record,
            None => {
                find_external(slug)?.context(format!("Selected skill '{slug}' is not installed"))?
            }
        };
        if !record.enabled || health_for_record(&record, &root) != "healthy" {
            bail!("Selected skill '{}' is missing or corrupted", record.slug)
        }
        records.push(record);
    }
    Ok(records)
}

fn install_staged_skill(
    conn: &Connection,
    stage: PathBuf,
    inspection: ArchiveInspection,
    source: InstallSource,
) -> Result<SkillInstallResult> {
    let target = config::skills_dir()?.join(&inspection.manifest.name);
    let replaced_existing = target.exists();
    let backup = replace_target_atomically(&stage, &target)?;
    let record = build_record(&target, &inspection, source);
    if let Err(error) = SkillRepo::upsert(conn, &record) {
        restore_backup(&target, backup.as_deref());
        return Err(error);
    }
    if let Some(path) = backup {
        let _ = fs::remove_dir_all(path);
    }
    let persisted = SkillRepo::find(conn, &record.slug)?.context("Cannot read installed skill")?;
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

fn prepare_staging_dir() -> Result<PathBuf> {
    let directory = config::skills_staging_dir()?.join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&directory).context("Cannot create skills staging directory")?;
    Ok(directory)
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
) -> SkillRecord {
    SkillRecord {
        slug: inspection.manifest.name.clone(),
        name: inspection.manifest.name.clone(),
        description: inspection.manifest.description.clone(),
        version: source.version,
        source_kind: source.kind,
        source_ref: source.reference,
        source_url: source.url,
        checksum: inspection.checksum.clone(),
        installed_path: target.display().to_string(),
        enabled: true,
        health: "healthy".to_string(),
        is_external: false,
        risk: merge_risk(&inspection.risk, &source.remote_risk),
        installed_at: String::new(),
        updated_at: String::new(),
    }
}

fn merge_risk(archive: &SkillRiskReport, remote: &SkillRiskReport) -> SkillRiskReport {
    let mut notes = archive.notes.clone();
    notes.extend(remote.notes.iter().cloned());
    SkillRiskReport {
        has_scripts: archive.has_scripts || remote.has_scripts,
        has_binary_files: archive.has_binary_files || remote.has_binary_files,
        has_allowed_tools: archive.has_allowed_tools || remote.has_allowed_tools,
        remote_scan_status: remote.remote_scan_status.clone(),
        notes,
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

fn skill_dir_for_record(record: &SkillRecord) -> Result<PathBuf> {
    if record.is_external {
        return external_skill_dir(record);
    }
    checked_skill_dir(record)
}

fn with_current_health(mut record: SkillRecord, root: &Path) -> SkillRecord {
    record.health = health_for_record(&record, root);
    record
}

fn health_for_record(record: &SkillRecord, root: &Path) -> String {
    if record.is_external {
        return external_skill_dir(record)
            .ok()
            .filter(|path| path.join("SKILL.md").is_file())
            .map(|_| "healthy".to_string())
            .unwrap_or_else(|| "missing".to_string());
    }
    let expected = root.join(&record.slug).join("SKILL.md");
    if expected.is_file() && Path::new(&record.installed_path) == root.join(&record.slug) {
        "healthy".to_string()
    } else {
        "missing".to_string()
    }
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
    let mut by_slug = BTreeMap::new();
    for (source, root) in roots {
        if by_slug.len() >= EXTERNAL_SKILL_LIMIT {
            break;
        }
        for skill in discover_from_root(source, root, EXTERNAL_SKILL_LIMIT - by_slug.len())? {
            by_slug.entry(skill.slug.clone()).or_insert(skill);
        }
    }
    Ok(by_slug.into_values().collect())
}

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

fn find_external_skill(slug: &str) -> Result<Option<SkillRecord>> {
    Ok(discover_external_skills()?
        .into_iter()
        .find(|skill| skill.slug == slug))
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
        enabled: true,
        health: "healthy".to_string(),
        is_external: true,
        risk: SkillRiskReport {
            notes: vec!["Discovered from another Agent's local Skills directory.".to_string()],
            ..SkillRiskReport::default()
        },
        installed_at: String::new(),
        updated_at: String::new(),
    }
}

fn external_skill_dir(record: &SkillRecord) -> Result<PathBuf> {
    let path = PathBuf::from(&record.installed_path);
    let canonical = fs::canonicalize(&path)
        .with_context(|| format!("Cannot resolve external Skill directory {}", path.display()))?;
    let home = dirs::home_dir().context("Failed to resolve the home directory")?;
    let root = match record.source_kind.as_str() {
        "codex" => home.join(".codex").join("skills"),
        "claude" => home.join(".claude").join("skills"),
        "cursor" => home.join(".cursor").join("skills"),
        _ => bail!("Unknown external Skill source '{}'", record.source_kind),
    };
    let canonical_root = fs::canonicalize(&root)
        .with_context(|| format!("Cannot resolve external Skill root {}", root.display()))?;
    if !canonical.starts_with(canonical_root) || !canonical.join("SKILL.md").is_file() {
        bail!("External Skill path is outside its allowed source directory")
    }
    Ok(canonical)
}

fn list_skill_files(root: &Path) -> Result<Vec<SkillFileNode>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.context("Cannot enumerate installed skill files")?;
        if !entry.file_type().is_file() {
            continue;
        }
        files.push(to_file_node(root, entry.path())?);
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn to_file_node(root: &Path, path: &Path) -> Result<SkillFileNode> {
    let relative = path
        .strip_prefix(root)
        .context("Skill file is outside its root")?;
    let path_string = relative.to_string_lossy().replace('\\', "/");
    let size = fs::metadata(path)
        .context("Cannot inspect installed skill file")?
        .len();
    let kind = if path_string == "SKILL.md" {
        "skill-manifest".to_string()
    } else {
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("file")
            .to_string()
    };
    Ok(SkillFileNode {
        path: path_string,
        kind,
        size_bytes: size,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use rusqlite::Connection;

    use crate::{
        db::{migrations::run_migrations, repository::SkillRepo},
        services::skills::types::{SkillRecord, SkillRiskReport},
    };

    use super::{
        discover_external_from_roots, discover_from_root, installed_selection_with,
        merge_installed_sources,
    };

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

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].source_kind, "codex");
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

    #[test]
    fn managed_enable_disable_and_overwrite_behavior_is_stable() {
        let conn = database();
        let root = tempfile::tempdir().unwrap();
        let first = record("demo-skill", "local", true, root.path());
        SkillRepo::upsert(&conn, &first).unwrap();
        SkillRepo::set_enabled(&conn, "demo-skill", false).unwrap();
        assert!(
            !SkillRepo::find(&conn, "demo-skill")
                .unwrap()
                .unwrap()
                .enabled
        );

        let mut replacement = first;
        replacement.description = "Replacement".to_string();
        replacement.enabled = true;
        SkillRepo::upsert(&conn, &replacement).unwrap();
        let persisted = SkillRepo::find(&conn, "demo-skill").unwrap().unwrap();
        assert_eq!(persisted.description, "Replacement");
        assert!(persisted.enabled);
    }

    #[test]
    fn installed_selection_rejects_disabled_and_corrupted_managed_skills() {
        let conn = database();
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let skill_dir = root.join("demo-skill");
        write_skill(root, "demo-skill", "Demo Skill");
        let skill = record("demo-skill", "local", true, &skill_dir);
        SkillRepo::upsert(&conn, &skill).unwrap();

        let selected =
            installed_selection_with(&conn, &["demo-skill".to_string()], root, |_| Ok(None))
                .unwrap();
        assert_eq!(selected.len(), 1);

        SkillRepo::set_enabled(&conn, "demo-skill", false).unwrap();
        assert!(
            installed_selection_with(&conn, &["demo-skill".to_string()], root, |_| Ok(None),)
                .is_err()
        );

        SkillRepo::set_enabled(&conn, "demo-skill", true).unwrap();
        fs::remove_file(skill_dir.join("SKILL.md")).unwrap();
        assert!(
            installed_selection_with(&conn, &["demo-skill".to_string()], root, |_| Ok(None),)
                .is_err()
        );
    }

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
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
            risk: SkillRiskReport::default(),
            installed_at: String::new(),
            updated_at: String::new(),
        }
    }
}
