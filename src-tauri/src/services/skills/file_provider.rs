use std::collections::BTreeMap;
use std::fs::{self, File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::config;
use crate::db::repository::SkillSourceRepo;

use super::manifest::parse_manifest;
use super::types::{
    SkillFileEntry, SkillFilePage, SkillFilePreview, SkillManifest, SkillRecord, SkillScanSummary,
    SkillSummary,
};

pub const MAX_READ_BYTES: usize = 200 * 1024;
pub const MAX_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;
const MAX_DIRECTORY_PAGE: usize = 500;
const DEFAULT_DIRECTORY_PAGE: usize = 200;
const MAX_HASH_BYTES: u64 = 8 * 1024 * 1024;
const MAX_CONCURRENT_READS: usize = 8;
const MANIFEST_HEADER_BYTES: usize = 64 * 1024;
const WINDOWS_REPARSE_POINT_ATTRIBUTE: u32 = 0x400;

static ACTIVE_READS: AtomicUsize = AtomicUsize::new(0);

/// Resolves every file operation from a trusted Skill source root. The UI only
/// supplies a stable Skill id and a relative path; it never chooses a host root.
pub struct SkillFileProvider {
    skill_id: String,
    root: PathBuf,
}

impl SkillFileProvider {
    pub fn from_registered(conn: &Connection, identifier: &str) -> Result<(Self, SkillRecord)> {
        let record =
            SkillSourceRepo::find(conn, identifier)?.context("Skill source is not registered")?;
        let root = registered_root(&record)?;
        Ok((
            Self {
                skill_id: record.skill_id.clone(),
                root,
            },
            record,
        ))
    }

    /// Adapter for a future explicitly populated remote preview cache. Merely
    /// opening remote detail never creates or downloads this directory.
    #[allow(dead_code)]
    pub fn from_cached_remote(provider: &str, slug: &str) -> Result<Self> {
        validate_cache_component(provider)?;
        validate_cache_component(slug)?;
        let cache_root = fs::canonicalize(config::skills_preview_cache_dir()?)
            .context("Cannot resolve the remote Skill preview cache")?;
        let requested = cache_root.join(provider).join(slug);
        let root = fs::canonicalize(&requested).with_context(|| {
            format!(
                "Remote Skill preview is not cached: {}",
                requested.display()
            )
        })?;
        if !root.starts_with(&cache_root) || !root.is_dir() {
            bail!("Remote Skill preview cache escaped its managed root")
        }
        Ok(Self {
            skill_id: format!("remote:{provider}:{slug}"),
            root,
        })
    }

    pub fn list_files(
        &self,
        generation: u64,
        parent: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<SkillFilePage> {
        let parent_path = match parent.filter(|value| !value.is_empty()) {
            Some(value) => validate_relative_path(value)?,
            None => PathBuf::new(),
        };
        let directory = self.resolve_existing(&parent_path, true)?;
        let offset = parse_cursor(cursor)?;
        let limit = limit
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_DIRECTORY_PAGE)
            .clamp(1, MAX_DIRECTORY_PAGE);
        let mut items = fs::read_dir(&directory)
            .context("Cannot enumerate Skill directory")?
            .map(|entry| entry.context("Cannot enumerate Skill directory entry"))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|entry| self.file_entry(&parent_path, entry.path()))
            .collect::<Result<Vec<_>>>()?;
        items.sort_by(|left, right| {
            right
                .is_directory
                .cmp(&left.is_directory)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.name.cmp(&right.name))
        });
        let total = items.len();
        let page = items.into_iter().skip(offset).take(limit).collect();
        let next = offset
            .saturating_add(limit)
            .lt(&total)
            .then(|| offset.saturating_add(limit).to_string());
        Ok(SkillFilePage {
            skill_id: self.skill_id.clone(),
            generation,
            parent: parent.filter(|value| !value.is_empty()).map(str::to_string),
            items: page,
            next_cursor: next,
        })
    }

    pub fn read_file(
        &self,
        generation: u64,
        path: &str,
        offset: u64,
        limit: Option<u32>,
    ) -> Result<SkillFilePreview> {
        let _permit = ReadPermit::acquire()?;
        let relative = validate_relative_path(path)?;
        let requested_limit = limit.unwrap_or(MAX_READ_BYTES as u32) as usize;
        if requested_limit == 0 || requested_limit > MAX_READ_BYTES {
            bail!("Skill file read limit must be between 1 byte and 200 KiB")
        }
        if offset >= MAX_PREVIEW_BYTES {
            bail!("Skill file preview is limited to 2 MiB per file")
        }
        let allowed = (MAX_PREVIEW_BYTES - offset).min(requested_limit as u64) as usize;
        let target = self.resolve_existing(&relative, false)?;
        let before = FileSnapshot::capture(&target)?;
        let mut file = File::open(&target).context("Cannot open Skill file")?;
        let opened = file
            .metadata()
            .context("Cannot inspect opened Skill file")?;
        before.verify_metadata(&opened)?;
        if offset > before.len {
            bail!("Skill file offset is beyond the end of the file")
        }

        let classification = classify_file(&mut file, before.len)?;
        let mut preview = match classification {
            FileClassification::Text { bom_len } => {
                read_text_preview(&mut file, path, offset, allowed, before.len, bom_len)?
            }
            FileClassification::Binary { encoding } => SkillFilePreview {
                skill_id: self.skill_id.clone(),
                generation,
                path: path.to_string(),
                kind: "binary".to_string(),
                encoding,
                content: None,
                offset,
                next_offset: None,
                total_size_bytes: before.len,
                sha256: hash_small_file(&mut file, before.len)?,
            },
            FileClassification::UnsupportedEncoding { encoding } => SkillFilePreview {
                skill_id: self.skill_id.clone(),
                generation,
                path: path.to_string(),
                kind: "unsupported_encoding".to_string(),
                encoding: Some(encoding),
                content: None,
                offset,
                next_offset: None,
                total_size_bytes: before.len,
                sha256: None,
            },
        };
        preview.skill_id = self.skill_id.clone();
        preview.generation = generation;

        let after_path = self.resolve_existing(&relative, false)?;
        if after_path != before.canonical_path {
            bail!("Skill file changed location while it was being read")
        }
        before.verify_metadata(&fs::metadata(&after_path)?)?;
        Ok(preview)
    }

    fn resolve_existing(&self, relative: &Path, directory: bool) -> Result<PathBuf> {
        ensure_no_links(&self.root, relative)?;
        let candidate = self.root.join(relative);
        let canonical = fs::canonicalize(&candidate)
            .with_context(|| format!("Cannot resolve Skill path {}", candidate.display()))?;
        if !canonical.starts_with(&self.root) {
            bail!("Skill path escapes its registered source root")
        }
        let metadata = fs::metadata(&canonical).context("Cannot inspect Skill path")?;
        if directory && !metadata.is_dir() {
            bail!("Requested Skill path is not a directory")
        }
        if !directory && !metadata.is_file() {
            bail!("Requested Skill path is not a regular file")
        }
        Ok(canonical)
    }

    fn file_entry(&self, parent: &Path, path: PathBuf) -> Result<SkillFileEntry> {
        let metadata = fs::symlink_metadata(&path).context("Cannot inspect Skill file entry")?;
        let is_link = is_link_like(&metadata);
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .context("Skill file entry has no name")?;
        let relative = parent.join(&name);
        let relative_text = relative.to_string_lossy().replace('\\', "/");
        let is_directory = metadata.is_dir() && !is_link;
        let size_bytes = if metadata.is_file() && !is_link {
            metadata.len()
        } else {
            0
        };
        Ok(SkillFileEntry {
            path: relative_text.clone(),
            name,
            kind: if is_link {
                "link".to_string()
            } else if is_directory {
                "directory".to_string()
            } else {
                file_kind(&relative_text)
            },
            size_bytes,
            is_directory,
            is_text_candidate: !is_link && !is_directory && is_text_candidate(&relative_text),
            is_link,
        })
    }

    #[cfg(test)]
    fn from_test_root(root: &Path) -> Result<Self> {
        Ok(Self {
            skill_id: "test-skill-id".to_string(),
            root: fs::canonicalize(root)?,
        })
    }
}

pub fn get_summary(conn: &Connection, identifier: &str) -> Result<SkillSummary> {
    let (provider, record) = SkillFileProvider::from_registered(conn, identifier)?;
    let generation = SkillSourceRepo::generation(conn)?;
    let manifest = provider
        .read_file(
            generation,
            "SKILL.md",
            0,
            Some(MANIFEST_HEADER_BYTES as u32),
        )
        .ok()
        .and_then(|preview| preview.content)
        .and_then(|text| parse_manifest(&text).ok())
        .unwrap_or_else(|| manifest_fallback(&record));
    Ok(SkillSummary {
        generation,
        skill: record,
        manifest,
        body_bytes_transferred: 0,
    })
}

pub fn get_scan_summary(conn: &Connection, identifier: &str) -> Result<SkillScanSummary> {
    let record =
        SkillSourceRepo::find(conn, identifier)?.context("Skill source is not registered")?;
    crate::db::repository::SkillSecurityRepo::scan_summary(
        conn,
        &record.skill_id,
        SkillSourceRepo::generation(conn)?,
    )
}

fn registered_root(record: &SkillRecord) -> Result<PathBuf> {
    let requested = PathBuf::from(&record.installed_path);
    let canonical = fs::canonicalize(&requested)
        .with_context(|| format!("Cannot resolve Skill root {}", requested.display()))?;
    if !canonical.is_dir() {
        bail!("Registered Skill root is not a directory")
    }
    let allowed = if record.is_external {
        let home = dirs::home_dir().context("Failed to resolve the home directory")?;
        match record.source_kind.as_str() {
            "codex" => home.join(".codex").join("skills"),
            "claude" => home.join(".claude").join("skills"),
            "cursor" => home.join(".cursor").join("skills"),
            _ => bail!("Unknown external Skill source '{}'", record.source_kind),
        }
    } else {
        config::skills_dir()?
    };
    let canonical_allowed = fs::canonicalize(&allowed)
        .with_context(|| format!("Cannot resolve allowed Skill root {}", allowed.display()))?;
    if !canonical.starts_with(&canonical_allowed) {
        bail!("Registered Skill root is outside its allowed source directory")
    }
    if !record.is_external {
        let expected = fs::canonicalize(allowed.join(&record.slug))
            .context("Cannot resolve the managed Skill's expected directory")?;
        if canonical != expected {
            bail!("Managed Skill registry root does not match its stable source record")
        }
    }
    Ok(canonical)
}

fn validate_relative_path(value: &str) -> Result<PathBuf> {
    if value.is_empty()
        || value.contains('\0')
        || value.contains(':')
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains("//")
        || value.contains("\\\\")
    {
        bail!("Skill file path must be a non-empty relative path without streams or devices")
    }
    let normalized = value.replace('\\', "/");
    let candidate = Path::new(&normalized);
    if candidate.is_absolute() {
        bail!("Absolute Skill file paths are not allowed")
    }
    let mut safe = PathBuf::new();
    for component in candidate.components() {
        let Component::Normal(segment) = component else {
            bail!("Skill file path contains dot or parent traversal")
        };
        let text = segment.to_string_lossy();
        if text.is_empty()
            || text.ends_with('.')
            || text.ends_with(' ')
            || is_windows_device_name(&text)
        {
            bail!("Skill file path contains an unsafe platform path component")
        }
        safe.push(segment);
    }
    if safe.as_os_str().is_empty() {
        bail!("Skill file path cannot resolve to the source root")
    }
    Ok(safe)
}

fn validate_cache_component(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || value == "."
        || value == ".."
    {
        bail!("Remote Skill cache key is invalid")
    }
    Ok(())
}

fn is_windows_device_name(value: &str) -> bool {
    let base = value
        .trim_end_matches(['.', ' '])
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || base
            .strip_prefix("COM")
            .or_else(|| base.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

fn ensure_no_links(root: &Path, relative: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).with_context(|| {
            format!("Cannot inspect Skill path component {}", current.display())
        })?;
        if is_link_like(&metadata) {
            bail!("Skill file paths cannot traverse symlinks or junctions")
        }
    }
    Ok(())
}

fn is_link_like(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        return metadata.file_attributes() & WINDOWS_REPARSE_POINT_ATTRIBUTE != 0;
    }
    #[cfg(not(windows))]
    {
        let _ = WINDOWS_REPARSE_POINT_ATTRIBUTE;
        false
    }
}

fn parse_cursor(cursor: Option<&str>) -> Result<usize> {
    cursor
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<usize>()
                .context("Skill file cursor is invalid")
        })
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn file_kind(path: &str) -> String {
    if path.eq_ignore_ascii_case("SKILL.md") {
        return "skill-manifest".to_string();
    }
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "file".to_string())
}

fn is_text_candidate(path: &str) -> bool {
    let kind = file_kind(path);
    matches!(
        kind.as_str(),
        "skill-manifest"
            | "md"
            | "txt"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "rs"
            | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "css"
            | "html"
            | "xml"
            | "csv"
            | "sh"
            | "ps1"
            | "bat"
            | "cmd"
            | "sql"
            | "ini"
            | "cfg"
            | "conf"
    )
}

enum FileClassification {
    Text { bom_len: usize },
    Binary { encoding: Option<String> },
    UnsupportedEncoding { encoding: String },
}

fn classify_file(file: &mut File, size: u64) -> Result<FileClassification> {
    file.seek(SeekFrom::Start(0))?;
    let mut probe = vec![0_u8; size.min(8192) as usize];
    file.read_exact(&mut probe)?;
    if probe.starts_with(&[0xFF, 0xFE]) {
        return Ok(FileClassification::UnsupportedEncoding {
            encoding: "utf-16le".to_string(),
        });
    }
    if probe.starts_with(&[0xFE, 0xFF]) {
        return Ok(FileClassification::UnsupportedEncoding {
            encoding: "utf-16be".to_string(),
        });
    }
    let bom_len = usize::from(probe.starts_with(&[0xEF, 0xBB, 0xBF])) * 3;
    let content = &probe[bom_len.min(probe.len())..];
    if content.iter().any(|byte| *byte == 0) {
        return Ok(FileClassification::Binary {
            encoding: Some("binary".to_string()),
        });
    }
    if std::str::from_utf8(content).is_err() {
        return Ok(FileClassification::UnsupportedEncoding {
            encoding: "unknown".to_string(),
        });
    }
    Ok(FileClassification::Text { bom_len })
}

fn read_text_preview(
    file: &mut File,
    path: &str,
    offset: u64,
    limit: usize,
    total: u64,
    bom_len: usize,
) -> Result<SkillFilePreview> {
    file.seek(SeekFrom::Start(offset))?;
    let remaining = total.saturating_sub(offset).min(limit as u64) as usize;
    let mut bytes = vec![0_u8; remaining];
    file.read_exact(&mut bytes)?;
    let prefix = if offset == 0 && bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        bom_len.min(bytes.len())
    } else {
        0
    };
    let text_bytes = &bytes[prefix..];
    let valid_len = match std::str::from_utf8(text_bytes) {
        Ok(_) => text_bytes.len(),
        Err(error) if error.error_len().is_none() => error.valid_up_to(),
        Err(_) => bail!("Skill file offset does not begin on a UTF-8 boundary"),
    };
    if valid_len == 0 && !text_bytes.is_empty() {
        bail!("Skill file preview limit splits the first UTF-8 character")
    }
    let content = std::str::from_utf8(&text_bytes[..valid_len])?.to_string();
    let consumed = prefix.saturating_add(valid_len) as u64;
    let loaded_to = offset.saturating_add(consumed);
    let next_offset = (loaded_to < total && loaded_to < MAX_PREVIEW_BYTES).then_some(loaded_to);
    Ok(SkillFilePreview {
        skill_id: String::new(),
        generation: 0,
        path: path.to_string(),
        kind: "text".to_string(),
        encoding: Some("utf-8".to_string()),
        content: Some(content),
        offset,
        next_offset,
        total_size_bytes: total,
        sha256: None,
    })
}

fn hash_small_file(file: &mut File, size: u64) -> Result<Option<String>> {
    if size > MAX_HASH_BYTES {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(Some(format!("{:x}", hasher.finalize())))
}

fn manifest_fallback(record: &SkillRecord) -> SkillManifest {
    SkillManifest {
        name: record.slug.clone(),
        description: record.description.clone(),
        license: None,
        compatibility: None,
        allowed_tools: None,
        metadata: BTreeMap::new(),
    }
}

struct FileSnapshot {
    canonical_path: PathBuf,
    len: u64,
    modified: Option<SystemTime>,
}

impl FileSnapshot {
    fn capture(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            bail!("Requested Skill path is not a regular file")
        }
        Ok(Self {
            canonical_path: fs::canonicalize(path)?,
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }

    fn verify_metadata(&self, metadata: &Metadata) -> Result<()> {
        if !metadata.is_file()
            || metadata.len() != self.len
            || metadata.modified().ok() != self.modified
        {
            bail!("Skill file changed while it was being read")
        }
        Ok(())
    }
}

struct ReadPermit;

impl ReadPermit {
    fn acquire() -> Result<Self> {
        ACTIVE_READS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_CONCURRENT_READS).then_some(active + 1)
            })
            .map_err(|_| anyhow::anyhow!("Too many concurrent Skill file reads"))?;
        Ok(Self)
    }
}

impl Drop for ReadPermit {
    fn drop(&mut self) {
        ACTIVE_READS.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant};

    use super::{validate_relative_path, FileSnapshot, SkillFileProvider, MAX_READ_BYTES};

    #[test]
    fn rejects_traversal_absolute_ads_device_and_ambiguous_paths() {
        for value in [
            "../secret.txt",
            "./secret.txt",
            "/etc/passwd",
            r"C:\secret.txt",
            r"\\?\C:\secret.txt",
            r"\\server\share\secret.txt",
            "safe.txt:stream",
            "CON",
            "aux.log",
            "folder./file.txt",
            "folder /file.txt",
        ] {
            assert!(validate_relative_path(value).is_err(), "accepted {value}");
        }
        assert!(validate_relative_path("文档/说明.md").is_ok());
    }

    #[test]
    fn lists_unicode_names_in_pages_without_reading_bodies() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(temporary.path().join("说明.md"), "正文").unwrap();
        fs::create_dir(temporary.path().join("scripts")).unwrap();
        let provider = SkillFileProvider::from_test_root(temporary.path()).unwrap();

        let first = provider.list_files(7, None, None, Some(1)).unwrap();
        assert_eq!(first.items.len(), 1);
        assert!(first.items[0].is_directory);
        assert_eq!(first.next_cursor.as_deref(), Some("1"));
        let second = provider
            .list_files(7, None, first.next_cursor.as_deref(), Some(1))
            .unwrap();
        assert_eq!(second.items[0].name, "说明.md");
    }

    #[test]
    fn reads_utf8_in_bounded_segments_and_reports_binary_and_encoding_metadata() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(
            temporary.path().join("long.md"),
            "A".repeat(MAX_READ_BYTES + 10),
        )
        .unwrap();
        fs::write(temporary.path().join("binary.bin"), [0, 1, 2, 3]).unwrap();
        fs::write(temporary.path().join("utf16.txt"), [0xFF, 0xFE, b'A', 0]).unwrap();
        let provider = SkillFileProvider::from_test_root(temporary.path()).unwrap();

        let first = provider.read_file(9, "long.md", 0, None).unwrap();
        assert_eq!(first.content.as_ref().unwrap().len(), MAX_READ_BYTES);
        assert_eq!(first.next_offset, Some(MAX_READ_BYTES as u64));
        let second = provider
            .read_file(9, "long.md", first.next_offset.unwrap(), None)
            .unwrap();
        assert_eq!(second.content.as_ref().unwrap().len(), 10);
        assert_eq!(second.next_offset, None);

        let binary = provider.read_file(9, "binary.bin", 0, None).unwrap();
        assert_eq!(binary.kind, "binary");
        assert!(binary.content.is_none());
        assert!(binary.sha256.is_some());

        let unsupported = provider.read_file(9, "utf16.txt", 0, None).unwrap();
        assert_eq!(unsupported.kind, "unsupported_encoding");
        assert_eq!(unsupported.encoding.as_deref(), Some("utf-16le"));
        assert!(unsupported.content.is_none());
    }

    #[test]
    fn rejects_link_escape_and_detects_snapshot_changes() {
        let temporary = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "secret").unwrap();
        fs::write(temporary.path().join("stable.txt"), "one").unwrap();
        let provider = SkillFileProvider::from_test_root(temporary.path()).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            temporary.path().join("escape.txt"),
        )
        .unwrap();
        #[cfg(windows)]
        assert!(std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                temporary.path().join("escape").to_str().unwrap(),
                outside.path().to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success());
        #[cfg(unix)]
        assert!(provider.read_file(1, "escape.txt", 0, None).is_err());
        #[cfg(windows)]
        assert!(provider.read_file(1, "escape/secret.txt", 0, None).is_err());

        let stable = temporary.path().join("stable.txt");
        let before = FileSnapshot::capture(&stable).unwrap();
        fs::write(&stable, "changed-size").unwrap();
        assert!(before
            .verify_metadata(&fs::metadata(&stable).unwrap())
            .is_err());
    }

    #[test]
    fn enforces_read_size_and_total_preview_limits() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(temporary.path().join("file.txt"), "content").unwrap();
        let provider = SkillFileProvider::from_test_root(temporary.path()).unwrap();
        assert!(provider
            .read_file(1, "file.txt", 0, Some(MAX_READ_BYTES as u32 + 1))
            .is_err());
        assert!(provider
            .read_file(1, "file.txt", 2 * 1024 * 1024, None)
            .is_err());
    }

    #[test]
    fn meets_local_summary_and_500_entry_tree_budgets() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(
            temporary.path().join("SKILL.md"),
            "---\nname: demo-skill\ndescription: Demo\n---\n",
        )
        .unwrap();
        for index in 0..499 {
            fs::write(
                temporary.path().join(format!("file-{index:03}.txt")),
                "metadata",
            )
            .unwrap();
        }
        let provider = SkillFileProvider::from_test_root(temporary.path()).unwrap();

        let tree_started = Instant::now();
        let page = provider.list_files(1, None, None, Some(500)).unwrap();
        let tree_elapsed = tree_started.elapsed();
        assert_eq!(page.items.len(), 500);
        assert!(
            tree_elapsed < Duration::from_millis(200),
            "tree took {tree_elapsed:?}"
        );

        let mut summary_samples = Vec::with_capacity(100);
        for _ in 0..100 {
            let started = Instant::now();
            let preview = provider
                .read_file(1, "SKILL.md", 0, Some(64 * 1024))
                .unwrap();
            assert_eq!(preview.kind, "text");
            summary_samples.push(started.elapsed());
        }
        summary_samples.sort_unstable();
        let p95 = summary_samples[94];
        assert!(p95 < Duration::from_millis(150), "summary P95 was {p95:?}");
    }
}
