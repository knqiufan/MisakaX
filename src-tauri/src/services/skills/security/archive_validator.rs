use std::path::Path;

use anyhow::Result;

use super::super::archive::{extract_archive, inspect_archive};
use super::super::types::ArchiveInspection;

/// Security boundary for archive intake. The legacy archive module remains the
/// format/parser implementation; callers publish nothing until this validator
/// has inspected and extracted into a scan-scoped quarantine directory.
pub fn validate_and_extract(archive: &Path, destination: &Path) -> Result<ArchiveInspection> {
    let inspection = inspect_archive(archive)?;
    extract_archive(archive, destination)?;
    Ok(inspection)
}
