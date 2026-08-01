use anyhow::Result;

use crate::services::skills::types::SkillManifest;

pub fn parse(markdown: &str) -> Result<SkillManifest> {
    super::super::super::manifest::parse_manifest(markdown)
}
