use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use walkdir::WalkDir;

use super::super::types::ArchiveInspection;
use super::archive_validator;

const QUARANTINE_QUOTA_BYTES: u64 = 500 * 1024 * 1024;
const EXPIRY: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug)]
pub struct QuarantinedArchive {
    pub scan_id: String,
    pub root: PathBuf,
    pub archive_path: PathBuf,
    pub extracted_path: PathBuf,
    pub inspection: ArchiveInspection,
}

impl QuarantinedArchive {
    pub fn load(root: PathBuf, scan_id: &str) -> Result<Self> {
        let canonical_root = fs::canonicalize(&root).context("Cannot resolve quarantine item")?;
        let expected_root = fs::canonicalize(crate::config::skills_quarantine_dir()?)
            .context("Cannot resolve quarantine root")?;
        if !canonical_root.starts_with(&expected_root)
            || canonical_root.file_name().and_then(|value| value.to_str()) != Some(scan_id)
        {
            bail!("Quarantine item is outside the managed root")
        }
        Self::load_unchecked(canonical_root, scan_id)
    }

    fn load_unchecked(root: PathBuf, scan_id: &str) -> Result<Self> {
        let inspection: ArchiveInspection = serde_json::from_slice(
            &fs::read(root.join("inspection.json")).context("Missing quarantine inspection")?,
        )
        .context("Invalid quarantine inspection")?;
        let archive_path = root.join("artifact.zip");
        let extracted_path = root.join("extracted");
        if !archive_path.is_file() || !extracted_path.join("SKILL.md").is_file() {
            bail!("Quarantine item is incomplete")
        }
        Ok(Self {
            scan_id: scan_id.to_string(),
            root,
            archive_path,
            extracted_path,
            inspection,
        })
    }
}

pub fn stage_archive(source: &Path, scan_id: &str) -> Result<QuarantinedArchive> {
    stage_archive_at(&crate::config::skills_quarantine_dir()?, source, scan_id)
}

fn stage_archive_at(root: &Path, source: &Path, scan_id: &str) -> Result<QuarantinedArchive> {
    validate_scan_id(scan_id)?;
    fs::create_dir_all(root).context("Cannot create Skill quarantine root")?;
    cleanup_expired_at(root, SystemTime::now())?;
    let source_metadata = fs::symlink_metadata(source).context("Cannot inspect Skill archive")?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_file() {
        bail!("Skill archive must be a regular file")
    }
    let used = quarantine_usage(root)?;
    if used.saturating_add(source_metadata.len()) > QUARANTINE_QUOTA_BYTES {
        bail!("SKILL_QUARANTINE_QUOTA: Skill quarantine quota exceeded")
    }
    let item_root = root.join(scan_id);
    fs::create_dir(&item_root).context("Cannot allocate quarantine item")?;
    let archive_path = item_root.join("artifact.zip");
    let result = (|| -> Result<QuarantinedArchive> {
        copy_regular_file(source, &archive_path)?;
        let extracted_path = item_root.join("extracted");
        let inspection = archive_validator::validate_and_extract(&archive_path, &extracted_path)?;
        fs::write(
            item_root.join("inspection.json"),
            serde_json::to_vec(&inspection)?,
        )?;
        fs::write(item_root.join("ready"), b"ready")?;
        Ok(QuarantinedArchive {
            scan_id: scan_id.to_string(),
            root: item_root.clone(),
            archive_path,
            extracted_path,
            inspection,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&item_root);
    }
    result
}

pub fn cleanup_expired() -> Result<usize> {
    cleanup_expired_at(&crate::config::skills_quarantine_dir()?, SystemTime::now())
}

fn cleanup_expired_at(root: &Path, now: SystemTime) -> Result<usize> {
    if !root.is_dir() {
        return Ok(0);
    }
    let canonical_root = fs::canonicalize(root).context("Cannot resolve quarantine root")?;
    let mut removed = 0;
    for entry in fs::read_dir(&canonical_root).context("Cannot enumerate quarantine root")? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if !metadata.is_dir() || entry.file_type()?.is_symlink() {
            continue;
        }
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if now.duration_since(modified).unwrap_or_default() < EXPIRY {
            continue;
        }
        let candidate = fs::canonicalize(entry.path())?;
        if candidate.parent() != Some(canonical_root.as_path()) {
            bail!("Quarantine cleanup target escaped its root")
        }
        fs::remove_dir_all(candidate)?;
        removed += 1;
    }
    Ok(removed)
}

fn quarantine_usage(root: &Path) -> Result<u64> {
    if !root.is_dir() {
        return Ok(0);
    }
    let mut total = 0u64;
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total = total.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(total)
}

fn copy_regular_file(source: &Path, destination: &Path) -> Result<()> {
    let mut input = File::open(source).context("Cannot open Skill archive")?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .context("Cannot create quarantine artifact")?;
    io::copy(&mut input, &mut output).context("Cannot copy Skill archive into quarantine")?;
    output.sync_all()?;
    Ok(())
}

fn validate_scan_id(scan_id: &str) -> Result<()> {
    if uuid::Uuid::parse_str(scan_id).is_err() {
        bail!("Invalid scan identifier")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, SystemTime};

    use zip::write::SimpleFileOptions;

    use super::{cleanup_expired_at, stage_archive_at, EXPIRY};

    #[test]
    fn archive_is_copied_and_extracted_inside_scan_scoped_quarantine() {
        let temporary = tempfile::tempdir().unwrap();
        let archive = temporary.path().join("skill.zip");
        let file = fs::File::create(&archive).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("SKILL.md", SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, b"---\nname: safe-skill\ndescription: Safe\n---\n")
            .unwrap();
        zip.finish().unwrap();
        let root = temporary.path().join("quarantine");
        let scan_id = uuid::Uuid::new_v4().to_string();

        let staged = stage_archive_at(&root, &archive, &scan_id).unwrap();
        assert_eq!(staged.root.parent(), Some(root.as_path()));
        assert!(staged.archive_path.is_file());
        assert!(staged.extracted_path.join("SKILL.md").is_file());
    }

    #[test]
    fn expired_scan_directories_are_removed_but_root_files_are_preserved() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("quarantine");
        let scan = root.join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&scan).unwrap();
        fs::write(scan.join("partial"), b"partial").unwrap();
        fs::write(root.join("do-not-delete"), b"root file").unwrap();

        let future = SystemTime::now() + EXPIRY + Duration::from_secs(1);
        assert_eq!(cleanup_expired_at(&root, future).unwrap(), 1);
        assert!(!scan.exists());
        assert!(root.join("do-not-delete").is_file());
    }
}
