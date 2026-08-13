use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use base64::Engine;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageFormat, ImageReader};
use sha2::{Digest, Sha256};

pub const MAX_AVATAR_BYTES: u64 = 5 * 1024 * 1024;
pub const MAX_AVATAR_PIXELS: u64 = 40_000_000;
pub const MAX_AVATAR_DIMENSION: u32 = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAvatar {
    pub storage_key: String,
    pub sha256: String,
}

fn validate_storage_key(storage_key: &str) -> Result<()> {
    let path = Path::new(storage_key);
    if storage_key.is_empty()
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some("webp")
        || !storage_key.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("invalid avatar storage key");
    }
    Ok(())
}

pub fn avatar_path(storage_root: &Path, storage_key: &str) -> Result<PathBuf> {
    validate_storage_key(storage_key)?;
    Ok(storage_root.join(storage_key))
}

pub fn store_avatar(source_path: &Path, storage_root: &Path) -> Result<StoredAvatar> {
    let metadata = fs::metadata(source_path).context("avatar source is not readable")?;
    if !metadata.is_file() {
        bail!("avatar source must be a file");
    }
    if metadata.len() == 0 || metadata.len() > MAX_AVATAR_BYTES {
        bail!("avatar source must be between 1 byte and 5 MiB");
    }

    let mut source = File::open(source_path).context("failed to open avatar source")?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::take(&mut source, MAX_AVATAR_BYTES + 1)
        .read_to_end(&mut bytes)
        .context("failed to read avatar source")?;
    if bytes.len() as u64 > MAX_AVATAR_BYTES {
        bail!("avatar source exceeds 5 MiB");
    }

    let reader = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .context("failed to inspect avatar format")?;
    let format = reader.format().context("avatar format is unknown")?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        bail!("avatar must be PNG, JPEG, or WebP");
    }
    let (width, height) = reader
        .into_dimensions()
        .context("failed to read avatar dimensions")?;
    if width == 0
        || height == 0
        || u64::from(width)
            .checked_mul(u64::from(height))
            .is_none_or(|pixels| pixels > MAX_AVATAR_PIXELS)
    {
        bail!("avatar pixel dimensions exceed the safety limit");
    }

    let decoded = image::load_from_memory_with_format(&bytes, format)
        .context("failed to decode avatar pixels")?;
    let normalized = decoded
        .thumbnail(MAX_AVATAR_DIMENSION, MAX_AVATAR_DIMENSION)
        .to_rgba8();
    let mut encoded = Vec::new();
    WebPEncoder::new_lossless(&mut encoded)
        .encode(
            normalized.as_raw(),
            normalized.width(),
            normalized.height(),
            ExtendedColorType::Rgba8,
        )
        .context("failed to encode normalized avatar")?;

    let sha256 = format!("{:x}", Sha256::digest(&encoded));
    let storage_key = format!("avatar-{}.webp", uuid::Uuid::new_v4());
    fs::create_dir_all(storage_root).context("failed to create avatar storage")?;
    let final_path = avatar_path(storage_root, &storage_key)?;
    let temporary_path = storage_root.join(format!(".avatar-{}.tmp", uuid::Uuid::new_v4()));
    let write_result = (|| -> Result<()> {
        let mut temporary = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
            .context("failed to create avatar temporary file")?;
        temporary
            .write_all(&encoded)
            .context("failed to write avatar temporary file")?;
        temporary
            .sync_all()
            .context("failed to sync avatar temporary file")?;
        drop(temporary);
        fs::rename(&temporary_path, &final_path).context("failed to publish normalized avatar")?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary_path);
        let _ = fs::remove_file(&final_path);
    }
    write_result?;

    Ok(StoredAvatar {
        storage_key,
        sha256,
    })
}

pub fn read_avatar_data_url(storage_root: &Path, storage_key: &str) -> Result<String> {
    let path = avatar_path(storage_root, storage_key)?;
    let bytes = fs::read(path).context("stored avatar is unavailable")?;
    Ok(format!(
        "data:image/webp;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

pub fn remove_avatar(storage_root: &Path, storage_key: &str) -> Result<()> {
    let path = avatar_path(storage_root, storage_key)?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("failed to remove stored avatar"),
    }
}

pub fn cleanup_orphaned_avatars(
    storage_root: &Path,
    active_storage_keys: &HashSet<String>,
) -> Result<usize> {
    fs::create_dir_all(storage_root).context("failed to create avatar storage")?;
    let mut removed = 0;
    for entry in fs::read_dir(storage_root).context("failed to list avatar storage")? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_temporary = name.starts_with(".avatar-") && name.ends_with(".tmp");
        let is_orphaned_avatar = name.starts_with("avatar-")
            && name.ends_with(".webp")
            && !active_storage_keys.contains(&name);
        if is_temporary || is_orphaned_avatar {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}
