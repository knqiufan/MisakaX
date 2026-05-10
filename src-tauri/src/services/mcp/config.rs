use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::types::{McpServerConfig, McpTransport};

/// mcp.json 文件格式
#[derive(Debug, serde::Deserialize)]
pub struct McpConfigFile {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

/// mcp.json 中单个 server 的配置
#[derive(Debug, serde::Deserialize)]
pub struct McpServerEntry {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(rename = "transportType")]
    pub transport_type: Option<String>,
    #[serde(rename = "autoConnect", default = "default_true")]
    pub auto_connect: bool,
}

fn default_true() -> bool {
    true
}

impl McpServerEntry {
    fn to_config(&self, id: &str) -> Option<McpServerConfig> {
        let transport = if let Some(url) = &self.url {
            let t_type = self
                .transport_type
                .as_deref()
                .unwrap_or("http");
            match t_type {
                "sse" => McpTransport::Sse {
                    url: url.clone(),
                    headers: self.headers.clone(),
                },
                _ => McpTransport::Http {
                    url: url.clone(),
                    headers: self.headers.clone(),
                },
            }
        } else if let Some(command) = &self.command {
            McpTransport::Stdio {
                command: command.clone(),
                args: self.args.clone(),
            }
        } else {
            return None;
        };

        Some(McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            transport,
            auto_connect: self.auto_connect,
            env: self.env.clone(),
        })
    }
}

/// MCP 配置加载器
pub struct McpConfigLoader;

impl McpConfigLoader {
    /// 从 mcp.json 文件加载配置
    pub fn load_from_file(path: &Path) -> Result<Vec<McpServerConfig>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(path)
            .context("Failed to read mcp.json")?;

        let config_file: McpConfigFile = serde_json::from_str(&content)
            .context("Failed to parse mcp.json")?;

        let configs: Vec<McpServerConfig> = config_file
            .mcp_servers
            .iter()
            .filter_map(|(id, entry)| entry.to_config(id))
            .collect();

        tracing::info!(count = configs.len(), "Loaded MCP configs from file");
        Ok(configs)
    }

    /// 从数据库加载配置
    pub fn load_from_db(conn: &Connection) -> Result<Vec<McpServerConfig>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, transport_json, auto_connect, env_json
             FROM mcp_servers",
        )?;

        let configs = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let transport_json: String = row.get(2)?;
                let auto_connect: bool = row.get(3)?;
                let env_json: Option<String> = row.get(4)?;
                Ok((id, name, transport_json, auto_connect, env_json))
            })?
            .filter_map(|row| {
                let (id, name, transport_json, auto_connect, env_json) =
                    row.ok()?;

                let transport: McpTransport =
                    serde_json::from_str(&transport_json).ok()?;
                let env: HashMap<String, String> = env_json
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                Some(McpServerConfig {
                    id,
                    name,
                    transport,
                    auto_connect,
                    env,
                })
            })
            .collect::<Vec<_>>();

        tracing::info!(count = configs.len(), "Loaded MCP configs from database");
        Ok(configs)
    }

    /// 合并 file 和 db 两个来源的配置（file 优先）
    pub fn load_all(
        config_dir: &Path,
        conn: &Connection,
    ) -> Result<Vec<McpServerConfig>> {
        let mcp_json_path = config_dir.join("mcp.json");
        let file_configs = Self::load_from_file(&mcp_json_path)?;
        let db_configs = Self::load_from_db(conn).unwrap_or_default();

        let mut seen = std::collections::HashSet::new();
        let mut merged = Vec::new();

        for config in file_configs {
            seen.insert(config.id.clone());
            merged.push(config);
        }

        for config in db_configs {
            if !seen.contains(&config.id) {
                merged.push(config);
            }
        }

        tracing::info!(
            total = merged.len(),
            "Merged MCP configs (file takes priority)"
        );
        Ok(merged)
    }

    /// 创建默认 mcp.json 模板
    pub fn ensure_default_config(config_dir: &Path) -> Result<()> {
        let path = config_dir.join("mcp.json");
        if path.exists() {
            return Ok(());
        }

        let template = serde_json::json!({
            "mcpServers": {}
        });

        let content = serde_json::to_string_pretty(&template)
            .context("Failed to serialize mcp.json template")?;

        std::fs::write(&path, content)
            .context("Failed to write default mcp.json")?;

        tracing::info!(path = %path.display(), "Created default mcp.json");
        Ok(())
    }
}
