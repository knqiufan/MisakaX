use std::thread;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::repository::SkillSourceRepo;
use crate::services::skills::types::SkillMigrationStatus;
use crate::AppState;

const EVENT_NAME: &str = "skills:migration-progress";

pub fn start(app: AppHandle) -> Result<()> {
    thread::Builder::new()
        .name("skill-security-migration".to_string())
        .spawn(move || {
            if let Err(error) = run(&app) {
                tracing::warn!(error = %error, "Skill security migration scan stopped");
            }
        })?;
    Ok(())
}

pub fn status(app: &AppHandle) -> Result<SkillMigrationStatus> {
    let state = app.state::<AppState>();
    let conn = state
        .db
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    SkillSourceRepo::migration_status(&conn)
}

pub fn retry(app: AppHandle) -> Result<SkillMigrationStatus> {
    let status = {
        let state = app.state::<AppState>();
        let conn = state
            .db
            .lock()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        SkillSourceRepo::retry_failed_migration_scans(&conn)?
    };
    emit_status(&app, &status);
    start(app)?;
    Ok(status)
}

fn run(app: &AppHandle) -> Result<()> {
    let records = {
        let state = app.state::<AppState>();
        let conn = state
            .db
            .lock()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        super::super::registry::sync_inventory_without_scanning(&conn)?;
        SkillSourceRepo::recover_migration_scan(&conn)?;
        let status = SkillSourceRepo::enroll_unscanned_migration_sources(&conn)?;
        emit_status(app, &status);
        SkillSourceRepo::pending_migration_sources(&conn)?
    };

    for record in records {
        let status = {
            let state = app.state::<AppState>();
            let conn = state
                .db
                .lock()
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            SkillSourceRepo::start_migration_scan(&conn, &record.skill_id)?;
            SkillSourceRepo::migration_status(&conn)?
        };
        emit_status(app, &status);

        let result = {
            let state = app.state::<AppState>();
            let conn = state
                .db
                .lock()
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            super::scan_existing_source(&conn, &record, |scan_id, progress| {
                let _ = app.emit(
                    "skills:scan-progress",
                    serde_json::json!({ "scan_id": scan_id, "progress": progress }),
                );
            })
        };
        let error = result.as_ref().err().map(ToString::to_string);
        let status = {
            let state = app.state::<AppState>();
            let conn = state
                .db
                .lock()
                .map_err(|lock_error| anyhow::anyhow!(lock_error.to_string()))?;
            SkillSourceRepo::record_migration_scan(&conn, &record.skill_id, error.as_deref())?;
            SkillSourceRepo::migration_status(&conn)?
        };
        emit_status(app, &status);
        let _ = app.emit(
            "skills:changed",
            serde_json::json!({
                "action": "migration_scanned",
                "skill_id": record.skill_id,
                "generation": null,
            }),
        );
    }

    let status = {
        let state = app.state::<AppState>();
        let conn = state
            .db
            .lock()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        SkillSourceRepo::finish_migration_scan(&conn)
            .context("Cannot finalize Skill migration scan")?
    };
    emit_status(app, &status);
    Ok(())
}

fn emit_status(app: &AppHandle, status: &SkillMigrationStatus) {
    let _ = app.emit(EVENT_NAME, status);
}
