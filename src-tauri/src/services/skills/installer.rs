use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
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
    SkillRepo::list(conn)?
        .into_iter()
        .map(|record| Ok(with_current_health(record, &root)))
        .collect()
}

pub fn get_detail(conn: &Connection, slug: &str) -> Result<SkillDetail> {
    let record = SkillRepo::find(conn, slug)?.context("Skill is not installed")?;
    let skill_dir = checked_skill_dir(&record)?;
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
    let records = SkillRepo::enabled_healthy(conn, slugs)?;
    let root = config::skills_dir()?;
    for record in &records {
        if health_for_record(record, &root) != "healthy" {
            bail!("Selected skill '{}' is missing or corrupted", record.slug)
        }
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

fn with_current_health(mut record: SkillRecord, root: &Path) -> SkillRecord {
    record.health = health_for_record(&record, root);
    record
}

fn health_for_record(record: &SkillRecord, root: &Path) -> String {
    let expected = root.join(&record.slug).join("SKILL.md");
    if expected.is_file() && Path::new(&record.installed_path) == root.join(&record.slug) {
        "healthy".to_string()
    } else {
        "missing".to_string()
    }
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
