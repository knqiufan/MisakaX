use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
const BINARY_PROBE_SIZE: usize = 8 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

pub fn validate_under_root(root: &Path, target: &Path) -> Result<(), String> {
    let root = canonicalize_existing(root)?;
    let target = normalize_target(&root, target)?;

    if target.starts_with(&root) {
        Ok(())
    } else {
        Err(format!(
            "Path is outside working directory: {}",
            target.display()
        ))
    }
}

pub fn read_text_file_inner(root: &Path, target: &Path) -> Result<String, String> {
    validate_under_root(root, target)?;
    let metadata = fs::metadata(target).map_err(|err| err.to_string())?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!("File too large: {} bytes", metadata.len()));
    }

    let bytes = fs::read(target).map_err(|err| err.to_string())?;
    let probe = &bytes[..bytes.len().min(BINARY_PROBE_SIZE)];
    if probe.contains(&0) {
        return Err("File appears to be binary".to_string());
    }

    String::from_utf8(bytes).map_err(|err| format!("Invalid UTF-8: {err}"))
}

pub fn write_text_file_inner(root: &Path, target: &Path, content: &str) -> Result<(), String> {
    validate_under_root(root, target)?;
    fs::write(target, content).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn fs_list_dir(working_dir: String, path: String) -> Result<Vec<FsEntry>, String> {
    list_dir_inner(Path::new(&working_dir), Path::new(&path))
}

#[tauri::command]
pub fn fs_read_text_file(working_dir: String, path: String) -> Result<String, String> {
    read_text_file_inner(Path::new(&working_dir), Path::new(&path))
}

#[tauri::command]
pub fn fs_write_text_file(
    working_dir: String,
    path: String,
    content: String,
) -> Result<(), String> {
    write_text_file_inner(Path::new(&working_dir), Path::new(&path), &content)
}

fn list_dir_inner(root: &Path, target: &Path) -> Result<Vec<FsEntry>, String> {
    validate_under_root(root, target)?;
    let mut entries: Vec<FsEntry> = fs::read_dir(target)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .map(to_fs_entry)
        .collect();
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(entries)
}

fn to_fs_entry(entry: fs::DirEntry) -> FsEntry {
    let metadata = entry.metadata().ok();
    FsEntry {
        name: entry.file_name().to_string_lossy().to_string(),
        path: entry.path().to_string_lossy().to_string(),
        is_dir: metadata.as_ref().is_some_and(|m| m.is_dir()),
        size: metadata.map(|m| m.len()).unwrap_or(0),
    }
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|err| format!("Invalid path {}: {err}", path.display()))
}

fn normalize_target(root: &Path, target: &Path) -> Result<PathBuf, String> {
    if target.exists() {
        return canonicalize_existing(target);
    }
    if let Some(parent) = target.parent().filter(|parent| parent.exists()) {
        let parent = canonicalize_existing(parent)?;
        let Some(file_name) = target.file_name() else {
            return Ok(parent);
        };
        return Ok(parent.join(file_name));
    }
    let candidate = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root.join(target)
    };
    Ok(candidate)
}
