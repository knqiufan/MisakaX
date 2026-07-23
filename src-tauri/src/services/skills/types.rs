use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub allowed_tools: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub source_kind: String,
    pub source_ref: Option<String>,
    pub source_url: Option<String>,
    pub checksum: String,
    pub installed_path: String,
    pub enabled: bool,
    pub health: String,
    pub risk: SkillRiskReport,
    pub installed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillRiskReport {
    #[serde(default)]
    pub has_scripts: bool,
    #[serde(default)]
    pub has_binary_files: bool,
    #[serde(default)]
    pub has_allowed_tools: bool,
    #[serde(default)]
    pub remote_scan_status: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileNode {
    pub path: String,
    pub kind: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub skill: SkillRecord,
    pub manifest: SkillManifest,
    pub files: Vec<SkillFileNode>,
    pub skill_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub provider: String,
    pub slug: String,
    pub display_name: String,
    pub summary: String,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub source_url: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub suspicious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkillDetail {
    pub skill: RemoteSkill,
    pub changelog: Option<String>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub risk: SkillRiskReport,
    pub manifest: Option<SkillManifest>,
    #[serde(default)]
    pub files: Vec<SkillFileNode>,
    pub skill_markdown: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchPage {
    pub items: Vec<RemoteSkill>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInspection {
    pub manifest: SkillManifest,
    pub files: Vec<SkillFileNode>,
    pub risk: SkillRiskReport,
    pub checksum: String,
    pub compressed_size_bytes: u64,
    pub uncompressed_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallResult {
    pub skill: SkillRecord,
    pub replaced_existing: bool,
}

#[derive(Debug, Clone)]
pub struct InstallSource {
    pub kind: String,
    pub reference: Option<String>,
    pub url: Option<String>,
    pub version: Option<String>,
    pub remote_risk: SkillRiskReport,
}
