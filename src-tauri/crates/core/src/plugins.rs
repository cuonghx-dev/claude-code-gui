//! Read-side logic for `~/.claude/plugins/`. Phase 1: list + get + readme.
//!
//! Plugin layout: `<plugin-id>/{plugin.json, skills/<slug>/SKILL.md, README.md}`.
//! The `plugin.json` is tolerantly parsed; missing fields default to None.

use std::path::Path;

use crate::types::{Plugin, PluginDetail};
use crate::AppError;

const PLUGINS_SUBDIR: &str = "plugins";
const SKILLS_SUBDIR: &str = "skills";

pub fn list(claude_dir: &Path) -> Result<Vec<Plugin>, AppError> {
    let root = claude_dir.join(PLUGINS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        match read_one(&path) {
            Ok(p) => out.push(p),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unreadable plugin"),
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

pub fn get(claude_dir: &Path, id: &str) -> Result<PluginDetail, AppError> {
    let plugin = list(claude_dir)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::not_found(format!("plugin '{id}' not found")))?;
    let readme_path = std::path::Path::new(&plugin.dir).join("README.md");
    let readme = std::fs::read_to_string(&readme_path).ok();
    Ok(PluginDetail { plugin, readme })
}

#[derive(serde::Deserialize, Default)]
struct PluginManifest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

fn default_enabled() -> bool {
    true
}

fn read_one(dir: &Path) -> Result<Plugin, AppError> {
    let id = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let manifest_path = dir.join("plugin.json");
    let manifest: PluginManifest = if manifest_path.is_file() {
        serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?
    } else {
        PluginManifest::default()
    };

    let skills_root = dir.join(SKILLS_SUBDIR);
    let mut skills = Vec::new();
    if skills_root.is_dir() {
        for entry in std::fs::read_dir(&skills_root)?.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("SKILL.md").is_file() {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    skills.push(name.to_string());
                }
            }
        }
        skills.sort();
    }

    Ok(Plugin {
        id: id.clone(),
        name: manifest.name.unwrap_or(id),
        version: manifest.version,
        description: manifest.description,
        enabled: manifest.enabled,
        dir: dir.to_string_lossy().into_owned(),
        skills,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_with_manifest_and_skills() {
        let td = tempfile::tempdir().unwrap();
        let plugin = td.path().join("plugins/test-plugin");
        std::fs::create_dir_all(plugin.join("skills/foo")).unwrap();
        std::fs::write(plugin.join("plugin.json"), r#"{"name":"Test","version":"1.0.0"}"#).unwrap();
        std::fs::write(plugin.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n\n").unwrap();
        std::fs::write(plugin.join("README.md"), "# Test\n\nReadme").unwrap();

        let plugins = list(td.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Test");
        assert_eq!(plugins[0].skills, vec!["foo"]);

        let detail = get(td.path(), "test-plugin").unwrap();
        assert!(detail.readme.is_some());
    }
}
