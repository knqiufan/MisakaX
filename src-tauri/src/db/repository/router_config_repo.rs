use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::RouterConfig;

/// 前端展示用的脱敏视图
#[derive(Debug, serde::Serialize)]
pub struct RouterConfigView {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key_masked: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

pub struct RouterConfigRepo;

impl RouterConfigRepo {
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<RouterConfig> {
        conn.query_row(
            "SELECT id, name, provider, api_key_encrypted, model, base_url,
                    config_json, is_active, created_at, api_compat
             FROM router_configs WHERE id = ?1",
            [id],
            Self::map_row,
        )
        .context("Router config not found")
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<RouterConfig>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, api_key_encrypted, model, base_url,
                    config_json, is_active, created_at, api_compat
             FROM router_configs ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn list_views(conn: &Connection) -> Result<Vec<RouterConfigView>> {
        let configs = Self::list_all(conn)?;
        Ok(configs.into_iter().map(Self::to_view).collect())
    }

    pub fn find_view_by_id(conn: &Connection, id: &str) -> Result<RouterConfigView> {
        let config = Self::find_by_id(conn, id)?;
        Ok(Self::to_view(config))
    }

    pub fn find_views_by_optional_id(
        conn: &Connection,
        config_id: Option<&str>,
    ) -> Result<Vec<RouterConfigView>> {
        match config_id {
            Some(id) => Ok(vec![Self::find_view_by_id(conn, id)?]),
            None => Self::list_views(conn),
        }
    }

    pub fn insert(
        conn: &Connection,
        id: &str,
        name: &str,
        provider: &str,
        api_key_encrypted: &str,
        model: Option<&str>,
        base_url: Option<&str>,
        config_json: Option<&str>,
        is_active: bool,
        api_compat: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO router_configs (id, name, provider, api_key_encrypted, model,
             base_url, config_json, is_active, api_compat)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                id,
                name,
                provider,
                api_key_encrypted,
                model,
                base_url,
                config_json,
                is_active as i32,
                api_compat,
            ],
        )?;
        Ok(())
    }

    pub fn update(
        conn: &Connection,
        id: &str,
        updates: &[(& str, Box<dyn rusqlite::types::ToSql>)],
    ) -> Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        Self::verify_exists(conn, id)?;

        let set_clauses: Vec<String> = updates
            .iter()
            .enumerate()
            .map(|(i, (col, _))| format!("{} = ?{}", col, i + 1))
            .collect();

        let sql = format!(
            "UPDATE router_configs SET {} WHERE id = ?{}",
            set_clauses.join(", "),
            updates.len() + 1
        );

        let mut params: Vec<&dyn rusqlite::types::ToSql> =
            updates.iter().map(|(_, v)| v.as_ref()).collect();
        let id_owned = id.to_string();
        params.push(&id_owned);

        conn.execute(&sql, params.as_slice())?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        let affected = conn.execute(
            "DELETE FROM router_configs WHERE id = ?1",
            [id],
        )?;

        if affected == 0 {
            anyhow::bail!("Router config not found: {}", id);
        }
        Ok(())
    }

    pub fn verify_exists(conn: &Connection, id: &str) -> Result<()> {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM router_configs WHERE id = ?1",
                [id],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )?;

        if !exists {
            anyhow::bail!("Router config not found: {}", id);
        }
        Ok(())
    }

    pub fn find_connection_info(
        conn: &Connection,
        id: &str,
    ) -> Result<(Option<String>, Option<String>, String)> {
        conn.query_row(
            "SELECT api_key_encrypted, base_url, provider FROM router_configs WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .context("Router config not found")
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<RouterConfig> {
        Ok(RouterConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            provider: row.get(2)?,
            api_key_encrypted: row.get(3)?,
            model: row.get(4)?,
            base_url: row.get(5)?,
            config_json: row.get(6)?,
            is_active: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
            api_compat: row.get(9)?,
        })
    }

    fn to_view(config: RouterConfig) -> RouterConfigView {
        let api_key_masked = config
            .api_key_encrypted
            .as_deref()
            .and_then(|enc| crate::crypto::decrypt(enc).ok())
            .map(|plain| crate::crypto::mask_api_key(&plain))
            .unwrap_or_else(|| "***".to_string());

        RouterConfigView {
            id: config.id,
            name: config.name,
            provider: config.provider,
            api_key_masked,
            model: config.model,
            base_url: config.base_url,
            is_active: config.is_active,
            created_at: config.created_at,
        }
    }
}
