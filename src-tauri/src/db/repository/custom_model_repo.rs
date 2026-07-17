use anyhow::Result;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

use crate::db::models::{CreateCustomModel, CustomModel};

pub struct CustomModelRepo;

impl CustomModelRepo {
    pub fn list_by_router(conn: &Connection, router_config_id: &str) -> Result<Vec<CustomModel>> {
        let mut stmt = conn.prepare(
            "SELECT id, router_config_id, model_id, display_name, supports_vision,
                    supports_thinking, model_types_json, max_tokens, context_window, enabled, sort_order,
                    thinking_off_model_id, created_at
             FROM custom_models
             WHERE router_config_id = ?1
             ORDER BY sort_order ASC, created_at ASC",
        )?;

        let models = stmt
            .query_map([router_config_id], map_custom_model)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(models)
    }

    /// Find an enabled model by provider model_id within a router config.
    pub fn find_enabled(
        conn: &Connection,
        router_config_id: &str,
        model_id: &str,
    ) -> Result<Option<CustomModel>> {
        let mut stmt = conn.prepare(
            "SELECT id, router_config_id, model_id, display_name, supports_vision,
                    supports_thinking, model_types_json, max_tokens, context_window, enabled, sort_order,
                    thinking_off_model_id, created_at
             FROM custom_models
             WHERE router_config_id = ?1 AND model_id = ?2 AND enabled = 1
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map(
            rusqlite::params![router_config_id, model_id],
            map_custom_model,
        )?;
        Ok(rows.next().transpose()?)
    }

    /// Whether a given model is enabled for a router config.
    pub fn is_enabled(conn: &Connection, router_config_id: &str, model_id: &str) -> Result<bool> {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM custom_models
                WHERE router_config_id = ?1 AND model_id = ?2 AND enabled = 1
            )",
            rusqlite::params![router_config_id, model_id],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    pub fn insert(
        conn: &Connection,
        id: &str,
        router_config_id: &str,
        model: &CreateCustomModel,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name,
             supports_vision, supports_thinking, model_types_json, max_tokens, context_window, enabled,
             sort_order, thinking_off_model_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                id,
                router_config_id,
                model.model_id,
                model.display_name,
                model.supports_vision as i32,
                model.supports_thinking as i32,
                serde_json::to_string(&model.model_types)?,
                model.max_tokens,
                model.context_window,
                model.enabled as i32,
                model.sort_order,
                model.thinking_off_model_id,
            ],
        )?;
        Ok(())
    }

    pub fn replace_all(
        conn: &mut Connection,
        router_config_id: &str,
        models: &[CreateCustomModel],
    ) -> Result<()> {
        validate_thinking_off_mappings(models)?;

        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM custom_models WHERE router_config_id = ?1",
            [router_config_id],
        )?;

        for model in models {
            let id = uuid::Uuid::new_v4().to_string();
            Self::insert(&tx, &id, router_config_id, model)?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let affected = conn.execute("DELETE FROM custom_models WHERE id = ?1", [id])?;

        if affected == 0 {
            anyhow::bail!("Custom model not found: {}", id);
        }
        Ok(())
    }
}

/// Validate thinking_off_model_id references against the incoming list before
/// replace_all deletes existing rows. Failures leave the prior config intact.
pub fn validate_thinking_off_mappings(models: &[CreateCustomModel]) -> Result<()> {
    let by_id: HashMap<&str, &CreateCustomModel> =
        models.iter().map(|m| (m.model_id.as_str(), m)).collect();

    if by_id.len() != models.len() {
        anyhow::bail!("Duplicate model_id in custom model list");
    }

    let mut seen_targets = HashSet::new();
    for model in models {
        let Some(ref target_id) = model.thinking_off_model_id else {
            continue;
        };
        if target_id.is_empty() {
            anyhow::bail!(
                "thinking_off_model_id for '{}' must not be empty",
                model.model_id
            );
        }
        if target_id == &model.model_id {
            anyhow::bail!(
                "thinking_off_model_id for '{}' cannot reference itself",
                model.model_id
            );
        }
        let Some(target) = by_id.get(target_id.as_str()) else {
            anyhow::bail!(
                "thinking_off_model_id '{}' for '{}' is not in the same provider list",
                target_id,
                model.model_id
            );
        };
        if !target.enabled {
            anyhow::bail!(
                "thinking_off_model_id '{}' for '{}' must be enabled",
                target_id,
                model.model_id
            );
        }
        if target.supports_thinking {
            anyhow::bail!(
                "thinking_off_model_id '{}' for '{}' must be a non-thinking model",
                target_id,
                model.model_id
            );
        }
        seen_targets.insert(target_id.as_str());
    }
    let _ = seen_targets;
    Ok(())
}

fn map_custom_model(row: &rusqlite::Row) -> rusqlite::Result<CustomModel> {
    Ok(CustomModel {
        id: row.get(0)?,
        router_config_id: row.get(1)?,
        model_id: row.get(2)?,
        display_name: row.get(3)?,
        supports_vision: row.get::<_, i32>(4)? != 0,
        supports_thinking: row.get::<_, i32>(5)? != 0,
        model_types: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
        max_tokens: row.get(7)?,
        context_window: row.get(8)?,
        enabled: row.get::<_, i32>(9)? != 0,
        sort_order: row.get(10)?,
        thinking_off_model_id: row.get(11)?,
        created_at: row.get(12)?,
    })
}
