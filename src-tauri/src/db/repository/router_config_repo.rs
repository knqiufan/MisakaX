use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::models::AdvancedConfig;
use crate::db::models::RouterConfig;

/// 前端展示用的脱敏视图
#[derive(Debug, serde::Serialize)]
pub struct RouterConfigView {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub vendor: Option<String>,
    pub api_key_masked: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_compat: Option<String>,
    pub config_json: Option<String>,
    pub advanced: AdvancedConfig,
    pub is_active: bool,
    pub created_at: String,
}

pub type RouterConnectionInfo = (Option<String>, Option<String>, String, Option<String>);

pub struct RouterConfigRepo;

impl RouterConfigRepo {
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<RouterConfig> {
        conn.query_row(
            "SELECT id, name, provider, vendor, api_key_encrypted, model, base_url,
                    config_json, advanced_json, is_active, created_at, api_compat
             FROM router_configs WHERE id = ?1",
            [id],
            Self::map_row,
        )
        .context("Router config not found")
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<RouterConfig>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, vendor, api_key_encrypted, model, base_url,
                    config_json, advanced_json, is_active, created_at, api_compat
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

    #[allow(clippy::too_many_arguments)]
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
        Self::insert_with_metadata(
            conn,
            id,
            name,
            provider,
            None,
            api_key_encrypted,
            model,
            base_url,
            config_json,
            None,
            is_active,
            api_compat,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_with_metadata(
        conn: &Connection,
        id: &str,
        name: &str,
        provider: &str,
        vendor: Option<&str>,
        api_key_encrypted: &str,
        model: Option<&str>,
        base_url: Option<&str>,
        config_json: Option<&str>,
        advanced_json: Option<&str>,
        is_active: bool,
        api_compat: Option<&str>,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO router_configs (id, name, provider, vendor, api_key_encrypted,
             model, base_url, config_json, advanced_json, is_active, api_compat)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id,
                name,
                provider,
                vendor,
                api_key_encrypted,
                model,
                base_url,
                config_json,
                advanced_json,
                is_active as i32,
                api_compat,
            ],
        )?;
        Ok(())
    }

    pub fn update(
        conn: &Connection,
        id: &str,
        updates: &[(&str, Box<dyn rusqlite::types::ToSql>)],
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
        let affected = conn.execute("DELETE FROM router_configs WHERE id = ?1", [id])?;

        if affected == 0 {
            anyhow::bail!("Router config not found: {}", id);
        }
        Ok(())
    }

    pub fn verify_exists(conn: &Connection, id: &str) -> Result<()> {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM router_configs WHERE id = ?1",
            [id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )?;

        if !exists {
            anyhow::bail!("Router config not found: {}", id);
        }
        Ok(())
    }

    pub fn find_connection_info(conn: &Connection, id: &str) -> Result<RouterConnectionInfo> {
        conn.query_row(
            "SELECT api_key_encrypted, base_url, provider, vendor
             FROM router_configs WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
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
            vendor: row.get(3)?,
            api_key_encrypted: row.get(4)?,
            model: row.get(5)?,
            base_url: row.get(6)?,
            config_json: row.get(7)?,
            advanced_json: row.get(8)?,
            is_active: row.get::<_, i32>(9)? != 0,
            created_at: row.get(10)?,
            api_compat: row.get(11)?,
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
            vendor: config.vendor,
            api_key_masked,
            model: config.model,
            base_url: config.base_url,
            api_compat: config.api_compat,
            config_json: config.config_json,
            advanced: parse_advanced_config(config.advanced_json.as_deref()),
            is_active: config.is_active,
            created_at: config.created_at,
        }
    }
}

fn parse_advanced_config(raw: Option<&str>) -> AdvancedConfig {
    raw.and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default()
}

/// 轻量 SQL UPDATE 字段构建器 — 为 [`RouterConfigRepo::update`] 构造 `(column, value)` 列表。
pub struct UpdateBuilder {
    fields: Vec<(&'static str, Box<dyn rusqlite::types::ToSql>)>,
}

impl UpdateBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn set_opt(&mut self, column: &'static str, value: &Option<String>) -> &mut Self {
        if let Some(v) = value {
            self.fields.push((column, Box::new(v.clone())));
        }
        self
    }

    pub fn set<T: rusqlite::types::ToSql + 'static>(
        &mut self,
        column: &'static str,
        value: T,
    ) -> &mut Self {
        self.fields.push((column, Box::new(value)));
        self
    }

    pub fn build(self) -> Vec<(&'static str, Box<dyn rusqlite::types::ToSql>)> {
        self.fields
    }
}

impl Default for UpdateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
