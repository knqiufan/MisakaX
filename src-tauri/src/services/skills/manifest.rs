use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::types::SkillManifest;

#[derive(Debug, Deserialize)]
struct RawManifest {
    name: String,
    description: String,
    license: Option<String>,
    compatibility: Option<String>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, serde_yaml::Value>,
}

pub fn parse_manifest(markdown: &str) -> Result<SkillManifest> {
    let yaml = frontmatter(markdown)?;
    let raw: RawManifest =
        serde_yaml::from_str(yaml).context("SKILL.md YAML frontmatter is invalid")?;
    validate_slug(&raw.name)?;
    validate_description(&raw.description)?;

    Ok(SkillManifest {
        name: raw.name,
        description: raw.description.trim().to_string(),
        license: normalize_optional(raw.license),
        compatibility: normalize_optional(raw.compatibility),
        allowed_tools: normalize_optional(raw.allowed_tools),
        metadata: raw.metadata,
    })
}

pub fn validate_skill_directory(dir_name: &str, manifest: &SkillManifest) -> Result<()> {
    if dir_name == manifest.name {
        return Ok(());
    }
    bail!(
        "Skill directory '{}' must match SKILL.md name '{}'",
        dir_name,
        manifest.name
    )
}

pub fn validate_slug(slug: &str) -> Result<()> {
    let valid_chars = slug
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    let has_bad_hyphen = slug.starts_with('-') || slug.ends_with('-') || slug.contains("--");
    if slug.is_empty() || slug.len() > 64 || !valid_chars || has_bad_hyphen {
        bail!("Skill name must be 1-64 lowercase letters, digits, and single hyphens")
    }
    Ok(())
}

fn frontmatter(markdown: &str) -> Result<&str> {
    let content = markdown
        .strip_prefix("---")
        .context("SKILL.md must start with YAML frontmatter")?;
    let end = content
        .find("\n---")
        .context("SKILL.md YAML frontmatter is not closed")?;
    Ok(&content[..end])
}

fn validate_description(description: &str) -> Result<()> {
    let length = description.trim().chars().count();
    if length == 0 || length > 1024 {
        bail!("Skill description must contain 1-1024 characters")
    }
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|text| (!text.trim().is_empty()).then(|| text.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{parse_manifest, validate_slug};

    #[test]
    fn parses_a_valid_skill_frontmatter() {
        let manifest = parse_manifest(
            "---\nname: code-review\ndescription: Review a change safely\n---\n# Instructions",
        )
        .unwrap();
        assert_eq!(manifest.name, "code-review");
        assert_eq!(manifest.description, "Review a change safely");
    }

    #[test]
    fn rejects_invalid_slug_and_missing_description() {
        assert!(validate_slug("../escape").is_err());
        assert!(parse_manifest("---\nname: valid-skill\ndescription: \n---\n").is_err());
    }
}
