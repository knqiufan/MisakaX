use anyhow::Result;
use rusqlite::Connection;

use crate::db::models::{CreateCustomModel, CustomModel};

pub struct CustomModelRepo;

impl CustomModelRepo {
    pub fn list_by_router(conn: &Connection, router_config_id: &str) -> Result<Vec<CustomModel>> {
        let mut stmt = conn.prepare(
            "SELECT id, router_config_id, model_id, display_name, supports_vision,
                    supports_thinking, max_tokens, context_window, enabled, sort_order,
                    created_at
             FROM custom_models
             WHERE router_config_id = ?1
             ORDER BY sort_order ASC, created_at ASC",
        )?;

        let models = stmt
            .query_map([router_config_id], map_custom_model)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(models)
    }

    pub fn insert(
        conn: &Connection,
        id: &str,
        router_config_id: &str,
        model: &CreateCustomModel,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name,
             supports_vision, supports_thinking, max_tokens, context_window, enabled, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                id,
                router_config_id,
                model.model_id,
                model.display_name,
                model.supports_vision as i32,
                model.supports_thinking as i32,
                model.max_tokens,
                model.context_window,
                model.enabled as i32,
                model.sort_order,
            ],
        )?;
        Ok(())
    }

    pub fn replace_all(
        conn: &mut Connection,
        router_config_id: &str,
        models: &[CreateCustomModel],
    ) -> Result<()> {
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

fn map_custom_model(row: &rusqlite::Row) -> rusqlite::Result<CustomModel> {
    Ok(CustomModel {
        id: row.get(0)?,
        router_config_id: row.get(1)?,
        model_id: row.get(2)?,
        display_name: row.get(3)?,
        supports_vision: row.get::<_, i32>(4)? != 0,
        supports_thinking: row.get::<_, i32>(5)? != 0,
        max_tokens: row.get(6)?,
        context_window: row.get(7)?,
        enabled: row.get::<_, i32>(8)? != 0,
        sort_order: row.get(9)?,
        created_at: row.get(10)?,
    })
}
