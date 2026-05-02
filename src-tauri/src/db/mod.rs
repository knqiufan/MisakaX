pub mod migrations;
pub mod models;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

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

    // Load sqlite-vec extension
    unsafe {
        let ext = std::mem::transmute(sqlite_vec::sqlite3_vec_init as *const () as usize);
        rusqlite::auto_extension::register_auto_extension(ext)?;
    }

    // Run schema migrations
    migrations::run_migrations(&conn)?;

    tracing::info!("Database initialized at: {}", db_path.display());
    Ok(conn)
}
