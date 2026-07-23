use std::collections::HashSet;
use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use super::manifest::{parse_manifest, validate_skill_directory};
use super::types::{ArchiveInspection, SkillFileNode, SkillRiskReport};

const MAX_ARCHIVE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 100 * 1024 * 1024;
const MAX_FILE_COUNT: usize = 500;
const MAX_COMPRESSION_RATIO: u64 = 100;

#[derive(Debug, Clone)]
pub struct ValidatedArchive {
    inspection: ArchiveInspection,
    root_prefix: Option<String>,
    entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Clone)]
struct ArchiveEntry {
    index: usize,
    path: PathBuf,
    size: u64,
}

pub fn inspect_archive(path: &Path) -> Result<ArchiveInspection> {
    Ok(validate_archive(path)?.inspection)
}

pub fn extract_archive(path: &Path, destination: &Path) -> Result<ArchiveInspection> {
    let validated = validate_archive(path)?;
    fs::create_dir_all(destination).context("Failed to create skill staging directory")?;
    copy_entries(path, destination, &validated)?;
    Ok(validated.inspection)
}

fn validate_archive(path: &Path) -> Result<ValidatedArchive> {
    let metadata = fs::metadata(path).context("Cannot read skills archive")?;
    if metadata.len() > MAX_ARCHIVE_BYTES {
        bail!("Skill archive exceeds the 10 MiB size limit")
    }

    let checksum = sha256_file(path)?;
    let mut archive = open_archive(path)?;
    let entries = collect_entries(&mut archive)?;
    let root_prefix = resolve_root_prefix(&entries)?;
    let skill_markdown = read_skill_markdown(&mut archive, &entries, root_prefix.as_deref())?;
    let manifest = parse_manifest(&skill_markdown)?;
    if let Some(root_dir) = root_prefix.as_deref() {
        validate_skill_directory(root_dir, &manifest)?;
    }

    Ok(ValidatedArchive {
        inspection: build_inspection(&entries, &manifest, checksum, metadata.len())?,
        root_prefix,
        entries,
    })
}

fn open_archive(path: &Path) -> Result<ZipArchive<File>> {
    let file = File::open(path).context("Cannot open skills archive")?;
    ZipArchive::new(file).context("Skill archive is not a valid ZIP file")
}

fn collect_entries(archive: &mut ZipArchive<File>) -> Result<Vec<ArchiveEntry>> {
    if archive.len() > MAX_FILE_COUNT {
        bail!("Skill archive exceeds the 500-file limit")
    }

    let mut paths = HashSet::new();
    let mut entries = Vec::new();
    let mut total_size = 0_u64;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .context("Cannot inspect archive entry")?;
        if file.is_dir() {
            continue;
        }
        reject_symlink(&file)?;
        let path = safe_relative_path(file.name())?;
        if !paths.insert(path.clone()) {
            bail!("Skill archive contains duplicate path '{}'", path.display())
        }
        total_size = total_size.saturating_add(file.size());
        check_entry_limits(file.size(), file.compressed_size(), total_size)?;
        entries.push(ArchiveEntry {
            index,
            path,
            size: file.size(),
        });
    }

    if entries.is_empty() {
        bail!("Skill archive does not contain any files")
    }
    Ok(entries)
}

fn reject_symlink(file: &zip::read::ZipFile<'_>) -> Result<()> {
    let is_symlink = file
        .unix_mode()
        .map(|mode| mode & 0o170000 == 0o120000)
        .unwrap_or(false);
    if is_symlink {
        bail!("Skill archive cannot contain symbolic links")
    }
    Ok(())
}

fn safe_relative_path(raw: &str) -> Result<PathBuf> {
    let path = Path::new(raw);
    if path.is_absolute() || raw.contains('\\') {
        bail!("Skill archive contains an unsafe path")
    }
    let safe = path
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    if !safe {
        bail!("Skill archive contains a path traversal entry")
    }
    Ok(path.to_path_buf())
}

fn check_entry_limits(size: u64, compressed: u64, total: u64) -> Result<()> {
    if total > MAX_EXTRACTED_BYTES {
        bail!("Skill archive exceeds the 100 MiB extraction limit")
    }
    if size > 0 && compressed > 0 && size / compressed > MAX_COMPRESSION_RATIO {
        bail!("Skill archive compression ratio is too high")
    }
    Ok(())
}

fn resolve_root_prefix(entries: &[ArchiveEntry]) -> Result<Option<String>> {
    if entries
        .iter()
        .any(|entry| entry.path == Path::new("SKILL.md"))
    {
        return Ok(None);
    }

    let roots: HashSet<String> = entries
        .iter()
        .filter_map(|entry| first_component(&entry.path))
        .collect();
    if roots.len() != 1
        || entries
            .iter()
            .any(|entry| entry.path.components().count() < 2)
    {
        bail!("Skill ZIP must contain SKILL.md at root or under one root directory")
    }
    Ok(roots.into_iter().next())
}

fn first_component(path: &Path) -> Option<String> {
    path.components()
        .next()
        .and_then(|component| match component {
            Component::Normal(value) => value.to_str().map(str::to_owned),
            _ => None,
        })
}

fn read_skill_markdown(
    archive: &mut ZipArchive<File>,
    entries: &[ArchiveEntry],
    root_prefix: Option<&str>,
) -> Result<String> {
    let skill_path = root_prefix
        .map(|prefix| Path::new(prefix).join("SKILL.md"))
        .unwrap_or_else(|| PathBuf::from("SKILL.md"));
    let entry = entries
        .iter()
        .find(|entry| entry.path == skill_path)
        .context("Skill ZIP must contain a root SKILL.md file")?;
    let mut file = archive
        .by_index(entry.index)
        .context("Cannot read SKILL.md")?;
    let mut markdown = String::new();
    io::Read::read_to_string(&mut file, &mut markdown).context("SKILL.md must be UTF-8")?;
    Ok(markdown)
}

fn build_inspection(
    entries: &[ArchiveEntry],
    manifest: &super::types::SkillManifest,
    checksum: String,
    compressed_size: u64,
) -> Result<ArchiveInspection> {
    let files = entries
        .iter()
        .map(|entry| SkillFileNode {
            path: entry.path.display().to_string(),
            kind: file_kind(&entry.path),
            size_bytes: entry.size,
        })
        .collect::<Vec<_>>();
    let uncompressed_size = entries.iter().map(|entry| entry.size).sum();
    let risk = build_risk(&files, manifest);
    Ok(ArchiveInspection {
        manifest: manifest.clone(),
        files,
        risk,
        checksum,
        compressed_size_bytes: compressed_size,
        uncompressed_size_bytes: uncompressed_size,
    })
}

fn build_risk(files: &[SkillFileNode], manifest: &super::types::SkillManifest) -> SkillRiskReport {
    let has_scripts = files.iter().any(|file| is_script(&file.path));
    let has_binary_files = files.iter().any(|file| is_binary(&file.path));
    let mut notes = Vec::new();
    if has_scripts {
        notes
            .push("This skill includes executable scripts; installation does not run them.".into());
    }
    if has_binary_files {
        notes.push("This skill includes binary assets; inspect them before use.".into());
    }
    if manifest.allowed_tools.is_some() {
        notes.push("This skill declares allowed tools that require a runtime review.".into());
    }
    SkillRiskReport {
        has_scripts,
        has_binary_files,
        has_allowed_tools: manifest.allowed_tools.is_some(),
        remote_scan_status: None,
        notes,
    }
}

fn file_kind(path: &Path) -> String {
    if path.file_name().is_some_and(|name| name == "SKILL.md") {
        return "skill-manifest".to_string();
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("file")
        .to_string()
}

fn is_script(path: &str) -> bool {
    [
        ".py", ".js", ".mjs", ".cjs", ".ts", ".sh", ".ps1", ".bat", ".cmd",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
}

fn is_binary(path: &str) -> bool {
    [".exe", ".dll", ".so", ".dylib", ".bin"]
        .iter()
        .any(|suffix| path.ends_with(suffix))
}

fn copy_entries(path: &Path, destination: &Path, validated: &ValidatedArchive) -> Result<()> {
    let mut archive = open_archive(path)?;
    for entry in &validated.entries {
        let target = destination.join(relative_path(entry, validated.root_prefix.as_deref())?);
        copy_entry(&mut archive, entry.index, &target)?;
    }
    Ok(())
}

fn relative_path(entry: &ArchiveEntry, root_prefix: Option<&str>) -> Result<PathBuf> {
    let Some(prefix) = root_prefix else {
        return Ok(entry.path.clone());
    };
    entry
        .path
        .strip_prefix(prefix)
        .map(Path::to_path_buf)
        .context("Skill archive root prefix is invalid")
}

fn copy_entry(archive: &mut ZipArchive<File>, index: usize, target: &Path) -> Result<()> {
    let mut source = archive
        .by_index(index)
        .context("Cannot extract skill archive entry")?;
    let parent = target
        .parent()
        .context("Skill file destination is invalid")?;
    fs::create_dir_all(parent).context("Cannot create skill directory")?;
    let mut output = File::create(target).context("Cannot create extracted skill file")?;
    io::copy(&mut source, &mut output).context("Cannot extract skill archive entry")?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).context("Cannot checksum skills archive")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read =
            io::Read::read(&mut file, &mut buffer).context("Cannot checksum skills archive")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{resolve_root_prefix, safe_relative_path, ArchiveEntry};

    #[test]
    fn rejects_absolute_and_traversal_archive_paths() {
        assert!(safe_relative_path("../SKILL.md").is_err());
        assert!(safe_relative_path("/SKILL.md").is_err());
        assert!(safe_relative_path("nested\\SKILL.md").is_err());
    }

    #[test]
    fn accepts_a_single_root_directory_layout() {
        let entries = vec![
            ArchiveEntry {
                index: 0,
                path: PathBuf::from("code-review/SKILL.md"),
                size: 1,
            },
            ArchiveEntry {
                index: 1,
                path: PathBuf::from("code-review/scripts/check.py"),
                size: 1,
            },
        ];
        assert_eq!(
            resolve_root_prefix(&entries).unwrap(),
            Some("code-review".to_string())
        );
    }
}
