use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::db::repository::{ArtifactRepo, SessionRepo};

use super::preview::{ArtifactPreview, PreviewerRegistry};
use super::types::{
    ArtifactMetadata, ArtifactOrigin, ArtifactRecord, ContentSafetyPolicy, PreviewState,
    RetentionState,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportOutcome {
    pub status: String,
    pub file_name: Option<String>,
}

pub struct ArtifactService {
    root: PathBuf,
    policy: ContentSafetyPolicy,
}

impl ArtifactService {
    pub fn new(root: PathBuf, policy: ContentSafetyPolicy) -> Result<Self> {
        fs::create_dir_all(root.join("objects").join("sha256"))?;
        fs::create_dir_all(root.join("staging"))?;
        Ok(Self { root, policy })
    }

    pub fn policy(&self) -> &ContentSafetyPolicy {
        &self.policy
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_base64(
        &self,
        conn: &Connection,
        session_id: String,
        origin_message_id: Option<String>,
        origin_kind: ArtifactOrigin,
        display_name: String,
        declared_media_type: String,
        bytes_base64: &str,
    ) -> Result<ArtifactMetadata> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(bytes_base64)
            .map_err(|_| anyhow!("ARTIFACT_TYPE_BLOCKED"))?;
        self.register_bytes(
            conn,
            session_id,
            origin_message_id,
            origin_kind,
            display_name,
            declared_media_type,
            &bytes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_bytes(
        &self,
        conn: &Connection,
        session_id: String,
        origin_message_id: Option<String>,
        origin_kind: ArtifactOrigin,
        display_name: String,
        declared_media_type: String,
        bytes: &[u8],
    ) -> Result<ArtifactMetadata> {
        if session_id.trim().is_empty()
            || bytes.is_empty()
            || bytes.len() as u64 > self.policy.max_artifact_bytes
        {
            return Err(anyhow!("ARTIFACT_TOO_LARGE"));
        }
        // Authorize the owner before any file-system mutation. Artifact ingress never
        // accepts a caller supplied path or creates data for a nonexistent session.
        SessionRepo::find_by_id(conn, &session_id).context("ARTIFACT_ACCESS_DENIED")?;
        if ArtifactRepo::active_bytes(conn)? + bytes.len() as u64
            > self.policy.max_total_artifact_bytes
        {
            return Err(anyhow!("ARTIFACT_STORAGE_QUOTA_EXCEEDED"));
        }
        let display_name = sanitize_display_name(&display_name)?;
        let media_type = detect_media_type(bytes, &declared_media_type)?;
        validate_image_dimensions(bytes, &media_type, self.policy.max_image_pixels)?;

        let sha256 = hex_hash(bytes);
        let storage_key = format!(
            "objects/sha256/{}/{}/{}",
            &sha256[..2],
            &sha256[2..4],
            sha256
        );
        let path = self.resolve_storage_key(&storage_key)?;
        let created_file = !path.exists();
        if created_file {
            self.atomic_write(&path, bytes)?;
        }
        let now = chrono::Utc::now().to_rfc3339();
        let preview_state = match media_type.as_str() {
            "image/png"
            | "image/jpeg"
            | "image/webp"
            | "image/gif"
            | "text/plain"
            | "text/markdown"
            | "application/json"
            | "text/csv"
            | "application/pdf"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                PreviewState::None
            }
            _ => PreviewState::Unsupported,
        };
        let record = ArtifactRecord {
            artifact_id: uuid::Uuid::new_v4().to_string(),
            owner_session_id: session_id,
            origin_message_id,
            origin_kind,
            display_name,
            media_type,
            byte_size: bytes.len() as u64,
            sha256,
            storage_key,
            preview_state,
            preview_artifact_id: None,
            retention_state: RetentionState::Active,
            created_at: now,
            expires_at: None,
        };
        if let Err(error) = ArtifactRepo::insert(conn, &record) {
            if created_file {
                let _ = fs::remove_file(&path);
            }
            return Err(error);
        }
        Ok(ArtifactMetadata::from(&record))
    }

    pub fn metadata(
        &self,
        conn: &Connection,
        session_id: &str,
        artifact_id: &str,
    ) -> Result<ArtifactMetadata> {
        let record = self.authorize(conn, session_id, artifact_id)?;
        Ok(ArtifactMetadata::from(&record))
    }

    pub fn preview(
        &self,
        conn: &Connection,
        session_id: &str,
        artifact_id: &str,
    ) -> Result<ArtifactPreview> {
        let record = self.authorize(conn, session_id, artifact_id)?;
        let bytes = if record.byte_size <= self.policy.max_preview_bytes {
            Some(self.read_bytes(&record)?)
        } else {
            None
        };
        Ok(PreviewerRegistry::preview_for(
            &record,
            bytes.as_deref(),
            &self.policy,
        ))
    }

    pub fn read_preview_base64(
        &self,
        conn: &Connection,
        session_id: &str,
        artifact_id: &str,
    ) -> Result<String> {
        let record = self.authorize(conn, session_id, artifact_id)?;
        if record.byte_size > self.policy.max_preview_bytes {
            return Err(anyhow!("PREVIEW_RESOURCE_LIMIT"));
        }
        Ok(base64::engine::general_purpose::STANDARD.encode(self.read_bytes(&record)?))
    }

    pub fn expire(&self, conn: &Connection, session_id: &str, artifact_id: &str) -> Result<()> {
        let record = self.authorize(conn, session_id, artifact_id)?;
        self.expire_record(conn, &record)
    }

    pub fn expire_session(&self, conn: &Connection, session_id: &str) -> Result<()> {
        for record in ArtifactRepo::find_by_sessions(conn, &[session_id.to_string()])? {
            if record.retention_state == RetentionState::Active {
                self.expire_record(conn, &record)?;
            }
        }
        Ok(())
    }

    pub fn export_to_path(
        &self,
        conn: &Connection,
        session_id: &str,
        artifact_id: &str,
        destination: &Path,
    ) -> Result<ExportOutcome> {
        let record = self.authorize(conn, session_id, artifact_id)?;
        if destination.exists() {
            return Err(anyhow!("ARTIFACT_EXPORT_FAILED"));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| anyhow!("ARTIFACT_EXPORT_FAILED"))?;
        if !parent.is_dir() {
            return Err(anyhow!("ARTIFACT_EXPORT_FAILED"));
        }
        let source = self.resolve_storage_key(&record.storage_key)?;
        let mut input = File::open(source)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            output.write_all(&buffer[..count])?;
            hasher.update(&buffer[..count]);
        }
        output.sync_all()?;
        let actual = format!("{:x}", hasher.finalize());
        if actual != record.sha256 {
            let _ = fs::remove_file(destination);
            return Err(anyhow!("ARTIFACT_HASH_MISMATCH"));
        }
        Ok(ExportOutcome {
            status: "saved".to_string(),
            file_name: destination
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string),
        })
    }

    fn authorize(
        &self,
        conn: &Connection,
        session_id: &str,
        artifact_id: &str,
    ) -> Result<ArtifactRecord> {
        let record = ArtifactRepo::find_by_id(conn, artifact_id)?;
        if record.owner_session_id != session_id {
            return Err(anyhow!("ARTIFACT_ACCESS_DENIED"));
        }
        if record.retention_state != RetentionState::Active {
            return Err(anyhow!("ARTIFACT_NOT_FOUND"));
        }
        Ok(record)
    }

    fn read_bytes(&self, record: &ArtifactRecord) -> Result<Vec<u8>> {
        let path = self.resolve_storage_key(&record.storage_key)?;
        let bytes = fs::read(path).context("ARTIFACT_NOT_FOUND")?;
        if bytes.len() as u64 != record.byte_size || hex_hash(&bytes) != record.sha256 {
            return Err(anyhow!("ARTIFACT_HASH_MISMATCH"));
        }
        Ok(bytes)
    }

    fn expire_record(&self, conn: &Connection, record: &ArtifactRecord) -> Result<()> {
        ArtifactRepo::set_retention_state(conn, &record.artifact_id, RetentionState::Expired)?;
        if !ArtifactRepo::has_active_storage_reference(
            conn,
            &record.storage_key,
            &record.artifact_id,
        )? {
            let path = self.resolve_storage_key(&record.storage_key)?;
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn atomic_write(&self, final_path: &Path, bytes: &[u8]) -> Result<()> {
        let parent = final_path
            .parent()
            .ok_or_else(|| anyhow!("ARTIFACT_STORAGE_QUOTA_EXCEEDED"))?;
        fs::create_dir_all(parent)?;
        let staging = self
            .root
            .join("staging")
            .join(format!("{}.part", uuid::Uuid::new_v4()));
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        fs::rename(&staging, final_path).or_else(|error| {
            if final_path.exists() {
                let _ = fs::remove_file(&staging);
                Ok(())
            } else {
                Err(error)
            }
        })?;
        Ok(())
    }

    fn resolve_storage_key(&self, key: &str) -> Result<PathBuf> {
        let relative = Path::new(key);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(anyhow!("ARTIFACT_ACCESS_DENIED"));
        }
        let path = self.root.join(relative);
        if !path.starts_with(&self.root) {
            return Err(anyhow!("ARTIFACT_ACCESS_DENIED"));
        }
        Ok(path)
    }
}

fn sanitize_display_name(input: &str) -> Result<String> {
    let name = Path::new(input)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .trim()
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if name.is_empty() || name == "." || name == ".." || name.len() > 180 {
        return Err(anyhow!("ARTIFACT_TYPE_BLOCKED"));
    }
    Ok(name)
}

fn detect_media_type(bytes: &[u8], declared: &str) -> Result<String> {
    let declared = declared.trim().to_ascii_lowercase();
    let detected = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else if bytes.starts_with(b"%PDF-") {
        "application/pdf"
    } else if bytes.starts_with(b"PK\x03\x04") {
        if contains_macro_project(bytes) {
            return Err(anyhow!("ARTIFACT_TYPE_BLOCKED"));
        }
        match declared.as_str() {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                declared.as_str()
            }
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                declared.as_str()
            }
            _ => return Err(anyhow!("ARTIFACT_TYPE_BLOCKED")),
        }
    } else if is_safe_text(bytes) {
        match declared.as_str() {
            "text/plain"
            | "text/markdown"
            | "application/json"
            | "application/geo+json"
            | "text/csv" => declared.as_str(),
            _ => "text/plain",
        }
    } else {
        return Err(anyhow!("ARTIFACT_TYPE_BLOCKED"));
    };
    if detected != declared && !(detected == "text/plain" && declared.is_empty()) {
        return Err(anyhow!("ARTIFACT_TYPE_BLOCKED"));
    }
    Ok(detected.to_string())
}

fn is_safe_text(bytes: &[u8]) -> bool {
    let text = std::str::from_utf8(bytes).ok();
    text.is_some_and(|value| {
        let lower = value.trim_start().to_ascii_lowercase();
        !lower.starts_with("<svg")
            && !lower.starts_with("<!doctype html")
            && !lower.starts_with("<html")
    })
}

fn validate_image_dimensions(bytes: &[u8], media_type: &str, max_pixels: u64) -> Result<()> {
    if !matches!(
        media_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) {
        return Ok(());
    }
    let dimensions = match media_type {
        "image/png" if bytes.len() >= 24 => Some((
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as u64,
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()) as u64,
        )),
        "image/gif" if bytes.len() >= 10 => Some((
            u16::from_le_bytes(bytes[6..8].try_into().unwrap()) as u64,
            u16::from_le_bytes(bytes[8..10].try_into().unwrap()) as u64,
        )),
        "image/jpeg" => jpeg_dimensions(bytes),
        "image/webp" => webp_dimensions(bytes),
        _ => None,
    };
    let Some((width, height)) = dimensions else {
        return Err(anyhow!("ARTIFACT_TYPE_BLOCKED"));
    };
    if width == 0 || height == 0 || width.saturating_mul(height) > max_pixels {
        return Err(anyhow!("ARTIFACT_TOO_LARGE"));
    }
    Ok(())
}

fn contains_macro_project(bytes: &[u8]) -> bool {
    bytes
        .windows(b"vbaProject.bin".len())
        .any(|window| window.eq_ignore_ascii_case(b"vbaProject.bin"))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u64, u64)> {
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    let mut index = 2usize;
    while index + 9 <= bytes.len() {
        while bytes.get(index) == Some(&0xff) {
            index += 1;
        }
        let marker = *bytes.get(index)?;
        index += 1;
        if matches!(marker, 0xd8 | 0xd9) || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let segment_length =
            u16::from_be_bytes([*bytes.get(index)?, *bytes.get(index + 1)?]) as usize;
        if segment_length < 7 || index + segment_length > bytes.len() {
            return None;
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            let height = u16::from_be_bytes([bytes[index + 3], bytes[index + 4]]) as u64;
            let width = u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]) as u64;
            return Some((width, height));
        }
        index += segment_length;
    }
    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u64, u64)> {
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WEBP") {
        return None;
    }
    match bytes.get(12..16)? {
        b"VP8X" if bytes.len() >= 30 => {
            let width = 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]) as u64;
            let height = 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]) as u64;
            Some((width, height))
        }
        b"VP8 " if bytes.len() >= 30 && bytes.get(23..26) == Some(&[0x9d, 0x01, 0x2a]) => {
            let width = (u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3fff) as u64;
            let height = (u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3fff) as u64;
            Some((width, height))
        }
        b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => {
            let width = 1 + (bytes[21] as u16 | ((bytes[22] as u16 & 0x3f) << 8)) as u64;
            let height = 1
                + ((bytes[22] as u16 >> 6)
                    | ((bytes[23] as u16) << 2)
                    | ((bytes[24] as u16 & 0x0f) << 10)) as u64;
            Some((width, height))
        }
        _ => None,
    }
}

fn hex_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes.extend([8, 6, 0, 0, 0]);
        bytes
    }

    #[test]
    fn blocks_cross_session_and_path_escape_access() {
        let temp = tempfile::tempdir().unwrap();
        let service =
            ArtifactService::new(temp.path().to_path_buf(), ContentSafetyPolicy::default())
                .unwrap();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute("INSERT INTO sessions (id) VALUES ('session-a')", [])
            .unwrap();
        let record = service
            .register_bytes(
                &conn,
                "session-a".into(),
                None,
                ArtifactOrigin::Agent,
                "chart.png".into(),
                "image/png".into(),
                &png(1, 1),
            )
            .unwrap();
        assert!(service
            .metadata(&conn, "session-b", &record.artifact_id)
            .is_err());
        assert!(service.resolve_storage_key("../secret").is_err());
    }

    #[test]
    fn rejects_svg_disguised_as_png() {
        let temp = tempfile::tempdir().unwrap();
        let service =
            ArtifactService::new(temp.path().to_path_buf(), ContentSafetyPolicy::default())
                .unwrap();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute("INSERT INTO sessions (id) VALUES ('session')", [])
            .unwrap();
        assert!(service
            .register_bytes(
                &conn,
                "session".into(),
                None,
                ArtifactOrigin::Agent,
                "bad.png".into(),
                "image/png".into(),
                b"<svg onload=alert(1)></svg>",
            )
            .is_err());
    }

    #[test]
    fn preserves_safe_geojson_media_type() {
        assert_eq!(
            detect_media_type(
                br#"{"type":"FeatureCollection","features":[]}"#,
                "application/geo+json",
            )
            .unwrap(),
            "application/geo+json"
        );
    }

    #[test]
    fn rejects_an_oversized_pixel_header_before_preview() {
        let temp = tempfile::tempdir().unwrap();
        let service =
            ArtifactService::new(temp.path().to_path_buf(), ContentSafetyPolicy::default())
                .unwrap();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute("INSERT INTO sessions (id) VALUES ('session')", [])
            .unwrap();
        assert!(service
            .register_bytes(
                &conn,
                "session".into(),
                None,
                ArtifactOrigin::Agent,
                "large.png".into(),
                "image/png".into(),
                &png(100_000, 100_000),
            )
            .is_err());
    }

    #[test]
    fn expiration_removes_unreferenced_bytes_but_keeps_the_audit_record() {
        let temp = tempfile::tempdir().unwrap();
        let service =
            ArtifactService::new(temp.path().to_path_buf(), ContentSafetyPolicy::default())
                .unwrap();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute("INSERT INTO sessions (id) VALUES ('session')", [])
            .unwrap();
        let record = service
            .register_bytes(
                &conn,
                "session".into(),
                None,
                ArtifactOrigin::Agent,
                "chart.png".into(),
                "image/png".into(),
                &png(1, 1),
            )
            .unwrap();
        let stored = service
            .resolve_storage_key(
                &ArtifactRepo::find_by_id(&conn, &record.artifact_id)
                    .unwrap()
                    .storage_key,
            )
            .unwrap();
        assert!(stored.exists());

        service
            .expire(&conn, "session", &record.artifact_id)
            .unwrap();

        assert!(!stored.exists());
        assert!(service
            .metadata(&conn, "session", &record.artifact_id)
            .is_err());
        assert!(ArtifactRepo::find_by_id(&conn, &record.artifact_id).is_ok());
    }
}
