use std::path::PathBuf;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;

static WATCHER: OnceLock<Mutex<Option<RecommendedWatcher>>> = OnceLock::new();

pub fn start(app: AppHandle) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })?;
    for root in watch_roots()? {
        if root.is_dir() {
            watcher
                .watch(&root, RecursiveMode::Recursive)
                .with_context(|| format!("Cannot watch Skill root {}", root.display()))?;
        }
    }
    let cell = WATCHER.get_or_init(|| Mutex::new(None));
    *cell
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))? = Some(watcher);

    std::thread::Builder::new()
        .name("skill-watch-debounce".to_string())
        .spawn(move || debounce_loop(app, receiver))?;
    Ok(())
}

fn debounce_loop(app: AppHandle, receiver: mpsc::Receiver<notify::Result<notify::Event>>) {
    loop {
        let first = match receiver.recv() {
            Ok(event) => event,
            Err(_) => return,
        };
        if let Err(error) = first {
            tracing::warn!(error = %error, "Skill filesystem watcher error");
            continue;
        }
        while receiver.recv_timeout(Duration::from_millis(500)).is_ok() {}
        let state = app.state::<AppState>();
        let Ok(conn) = state.db.lock() else {
            continue;
        };
        match crate::services::skills::installer::list_installed(&conn) {
            Ok(_) => {
                let generation = crate::db::repository::SkillSourceRepo::generation(&conn).ok();
                let _ = app.emit(
                    "skills:changed",
                    serde_json::json!({
                        "action": "filesystem_changed",
                        "skill_id": null,
                        "generation": generation,
                    }),
                );
            }
            Err(error) => {
                tracing::warn!(error = %error, "Failed to reconcile Skill filesystem changes")
            }
        }
    }
}

fn watch_roots() -> Result<Vec<PathBuf>> {
    let mut roots = vec![crate::config::skills_dir()?];
    if let Some(home) = dirs::home_dir() {
        roots.extend([
            home.join(".codex").join("skills"),
            home.join(".claude").join("skills"),
            home.join(".cursor").join("skills"),
        ]);
    }
    Ok(roots)
}
