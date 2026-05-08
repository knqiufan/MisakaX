use anyhow::Result;
use rusqlite::Connection;

use crate::db::models::CreateCustomModel;

pub struct CustomModelRepo;

impl CustomModelRepo {
    pub fn insert(
        conn: &Connection,
        id: &str,
        router_config_id: &str,
        model: &CreateCustomModel,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name,
             supports_vision, supports_thinking, max_tokens, context_window)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                id,
                router_config_id,
                model.model_id,
                model.display_name,
                model.supports_vision as i32,
                model.supports_thinking as i32,
                model.max_tokens,
                model.context_window,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let affected = conn.execute(
            "DELETE FROM custom_models WHERE id = ?1",
            [id],
        )?;

        if affected == 0 {
            anyhow::bail!("Custom model not found: {}", id);
        }
        Ok(())
    }
}
