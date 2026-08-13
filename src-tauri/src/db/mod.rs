pub mod migrations;
pub mod models;
pub mod repository;

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

type SqliteExtensionInit = unsafe extern "C" fn(
    *mut rusqlite::ffi::sqlite3,
    *mut *mut i8,
    *const rusqlite::ffi::sqlite3_api_routines,
) -> i32;

/// Initialize the database: create file, load sqlite-vec, run migrations.
pub fn init_database(db_path: &Path) -> Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;

    backup_before_migration(&conn, db_path, 16)?;

    // Load sqlite-vec extension
    unsafe {
        let ext = std::mem::transmute::<usize, SqliteExtensionInit>(
            sqlite_vec::sqlite3_vec_init as *const () as usize,
        );
        rusqlite::auto_extension::register_auto_extension(ext)?;
    }

    // Run schema migrations
    migrations::run_migrations(&conn)?;
    repository::ProfileRepo::ensure_default(&conn)?;
    match crate::services::usage::backfill::backfill_legacy_usage(&conn) {
        Ok(diagnostics) => tracing::info!(?diagnostics, "Legacy usage backfill complete"),
        Err(error) => tracing::warn!(error = %error, "Legacy usage backfill failed"),
    }

    tracing::info!("Database initialized at: {}", db_path.display());
    Ok(conn)
}

/// Create one SQLite-consistent backup before a schema upgrade. `VACUUM INTO`
/// includes committed WAL content and leaves the original database untouched.
pub fn backup_before_migration(
    conn: &Connection,
    db_path: &Path,
    target_version: i64,
) -> Result<Option<std::path::PathBuf>> {
    let has_version_table: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if has_version_table.is_none() {
        return Ok(None);
    }
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
        [],
        |row| row.get(0),
    )?;
    if current >= target_version {
        return Ok(None);
    }
    let backup = db_path.with_extension(format!("pre-v{target_version}.sqlite3"));
    if !backup.exists() {
        conn.execute("VACUUM INTO ?1", [backup.display().to_string()])?;
    }
    Ok(Some(backup))
}
