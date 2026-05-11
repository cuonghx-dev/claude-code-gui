//! Read+write logic for Claude Code's installed plugins. Plugins live under
//! `~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/` with a manifest
//! at `.claude-plugin/plugin.json`. The canonical install registry is
//! `~/.claude/plugins/installed_plugins.json`; enabled state is tracked in
//! `~/.claude/settings.json` under `enabledPlugins`. Plugin id format is
//! `<plugin-name>@<marketplace-name>` (matching the registry keys).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::io;
use crate::types::{Plugin, PluginDetail};
use crate::AppError;

const PLUGINS_SUBDIR: &str = "plugins";
const SKILLS_SUBDIR: &str = "skills";
const REGISTRY_FILE: &str = "installed_plugins.json";
const SETTINGS_FILE: &str = "settings.json";

pub fn list(claude_dir: &Path) -> Result<Vec<Plugin>, AppError> {
    let registry = read_registry(claude_dir);
    let enabled_map = read_enabled_map(claude_dir);

    let mut out = Vec::new();
    for (id, entries) in &registry {
        let Some(install_dir) = pick_install_dir(entries) else {
            tracing::warn!(plugin = %id, "no usable install path");
            continue;
        };
        let last = entries.last();
        let registry_version = last.and_then(|e| e.version.clone());
        let installed_at = last
            .and_then(|e| e.last_updated.clone().or_else(|| e.installed_at.clone()));
        match read_one(
            id,
            &install_dir,
            &enabled_map,
            registry_version.as_deref(),
            installed_at,
        ) {
            Ok(p) => out.push(p),
            Err(e) => tracing::warn!(error = %e, plugin = %id, "skipping unreadable plugin"),
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
    let readme = read_first_readme(Path::new(&plugin.dir));
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
}

#[derive(serde::Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct InstalledEntry {
    #[serde(default)]
    install_path: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    installed_at: Option<String>,
    #[serde(default)]
    last_updated: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct InstalledFile {
    #[serde(default)]
    plugins: BTreeMap<String, Vec<InstalledEntry>>,
}

fn read_registry(claude_dir: &Path) -> BTreeMap<String, Vec<InstalledEntry>> {
    let path = claude_dir.join(PLUGINS_SUBDIR).join(REGISTRY_FILE);
    if !path.is_file() {
        return BTreeMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<InstalledFile>(&raw)
            .map(|f| f.plugins)
            .unwrap_or_default(),
        Err(_) => BTreeMap::new(),
    }
}

fn read_enabled_map(claude_dir: &Path) -> BTreeMap<String, bool> {
    let path = claude_dir.join(SETTINGS_FILE);
    if !path.is_file() {
        return BTreeMap::new();
    }
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return BTreeMap::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return BTreeMap::new(),
    };
    let Some(map) = v.get("enabledPlugins").and_then(|m| m.as_object()) else {
        return BTreeMap::new();
    };
    map.iter()
        .filter_map(|(k, val)| val.as_bool().map(|b| (k.clone(), b)))
        .collect()
}

fn pick_install_dir(entries: &[InstalledEntry]) -> Option<PathBuf> {
    for entry in entries.iter().rev() {
        if let Some(p) = entry.install_path.as_deref() {
            let dir = PathBuf::from(p);
            if dir.is_dir() {
                return Some(dir);
            }
        }
    }
    None
}

fn read_one(
    id: &str,
    dir: &Path,
    enabled: &BTreeMap<String, bool>,
    registry_version: Option<&str>,
    installed_at: Option<String>,
) -> Result<Plugin, AppError> {
    let manifest_path = dir.join(".claude-plugin").join("plugin.json");
    let manifest: PluginManifest = if manifest_path.is_file() {
        serde_json::from_str(&std::fs::read_to_string(&manifest_path)?).unwrap_or_default()
    } else {
        PluginManifest::default()
    };

    let mut skills = Vec::new();
    let skills_root = dir.join(SKILLS_SUBDIR);
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

    let mut parts = id.splitn(2, '@');
    let display_name = parts.next().unwrap_or(id);
    let marketplace = parts.next().map(String::from);

    Ok(Plugin {
        id: id.to_string(),
        name: manifest.name.unwrap_or_else(|| display_name.to_string()),
        version: manifest.version.or_else(|| registry_version.map(String::from)),
        description: manifest.description,
        enabled: enabled.get(id).copied().unwrap_or(true),
        dir: dir.to_string_lossy().into_owned(),
        skills,
        marketplace,
        installed_at,
    })
}

fn read_first_readme(dir: &Path) -> Option<String> {
    for name in ["README.md", "Readme.md", "readme.md"] {
        if let Ok(s) = std::fs::read_to_string(dir.join(name)) {
            return Some(s);
        }
    }
    None
}

pub fn delete(claude_dir: &Path, id: &str) -> Result<(), AppError> {
    let registry_path = claude_dir.join(PLUGINS_SUBDIR).join(REGISTRY_FILE);
    let raw = std::fs::read_to_string(&registry_path)
        .map_err(|_| AppError::not_found(format!("plugin '{id}' not found")))?;
    let mut root: serde_json::Value = serde_json::from_str(&raw)?;

    let install_paths: Vec<String> = root
        .get("plugins")
        .and_then(|v| v.get(id))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("installPath").and_then(|s| s.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let removed = root
        .get_mut("plugins")
        .and_then(|v| v.as_object_mut())
        .and_then(|obj| obj.remove(id))
        .is_some();
    if !removed {
        return Err(AppError::not_found(format!("plugin '{id}' not found")));
    }

    for p in install_paths {
        let dir = PathBuf::from(&p);
        if dir.is_dir() {
            let _ = io::remove_dir_all(&dir);
        }
    }

    let serialized = serde_json::to_string_pretty(&root)?;
    io::atomic_write(&registry_path, serialized.as_bytes())?;

    let _ = set_enabled_flag(claude_dir, id, None);
    Ok(())
}

pub fn set_enabled(claude_dir: &Path, id: &str, enabled: bool) -> Result<(), AppError> {
    let registry = read_registry(claude_dir);
    if !registry.contains_key(id) {
        return Err(AppError::not_found(format!("plugin '{id}' not found")));
    }
    set_enabled_flag(claude_dir, id, Some(enabled))
}

fn set_enabled_flag(claude_dir: &Path, id: &str, value: Option<bool>) -> Result<(), AppError> {
    let path = claude_dir.join(SETTINGS_FILE);
    let mut root: serde_json::Value = if path.is_file() {
        serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_else(|_| {
            serde_json::Value::Object(serde_json::Map::new())
        })
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };
    if !root.is_object() {
        root = serde_json::Value::Object(serde_json::Map::new());
    }
    let obj = root.as_object_mut().expect("ensured object above");
    let entry = obj
        .entry("enabledPlugins".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    if !entry.is_object() {
        *entry = serde_json::Value::Object(serde_json::Map::new());
    }
    let map = entry.as_object_mut().expect("ensured object above");
    match value {
        Some(b) => {
            map.insert(id.to_string(), serde_json::Value::Bool(b));
        }
        None => {
            map.remove(id);
        }
    }
    let serialized = serde_json::to_string_pretty(&root)?;
    io::atomic_write(&path, serialized.as_bytes())
}

/// Persist an advisory list of "active" skill slugs for a plugin. Stored in a
/// sidecar `~/.claude/plugins/<id>.activeSkills.json` since the upstream
/// `installed_plugins.json` schema has no such field. The on-disk skill
/// directories under the plugin's install path are not touched.
pub fn update_skills(claude_dir: &Path, id: &str, slugs: Vec<String>) -> Result<(), AppError> {
    let registry = read_registry(claude_dir);
    if !registry.contains_key(id) {
        return Err(AppError::not_found(format!("plugin '{id}' not found")));
    }
    let safe = id.replace('/', "_").replace('\\', "_");
    let sidecar = claude_dir
        .join(PLUGINS_SUBDIR)
        .join(format!("{safe}.activeSkills.json"));
    let serialized = serde_json::to_string_pretty(&slugs)?;
    io::atomic_write(&sidecar, serialized.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_registry(claude_dir: &Path, plugins: &[(&str, &str, &str)]) {
        let mut map = serde_json::Map::new();
        for (id, install_path, version) in plugins {
            let entry = serde_json::json!([{
                "scope": "user",
                "installPath": install_path,
                "version": version,
            }]);
            map.insert((*id).to_string(), entry);
        }
        let body = serde_json::json!({ "version": 2, "plugins": map });
        let path = claude_dir.join(PLUGINS_SUBDIR).join(REGISTRY_FILE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&body).unwrap()).unwrap();
    }

    #[test]
    fn list_from_registry() {
        let td = tempfile::tempdir().unwrap();
        let install = td.path().join("cache/mkt/plug/1.0.0");
        std::fs::create_dir_all(install.join(".claude-plugin")).unwrap();
        std::fs::write(
            install.join(".claude-plugin/plugin.json"),
            r#"{"name":"Plug","description":"hi","version":"1.0.0"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(install.join("skills/foo")).unwrap();
        std::fs::write(install.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n").unwrap();
        std::fs::write(install.join("README.md"), "# Plug\n").unwrap();

        write_registry(
            td.path(),
            &[("plug@mkt", install.to_str().unwrap(), "1.0.0")],
        );

        let plugins = list(td.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "plug@mkt");
        assert_eq!(plugins[0].name, "Plug");
        assert_eq!(plugins[0].description.as_deref(), Some("hi"));
        assert_eq!(plugins[0].skills, vec!["foo"]);
        assert!(plugins[0].enabled);

        let detail = get(td.path(), "plug@mkt").unwrap();
        assert!(detail.readme.is_some());
    }

    #[test]
    fn list_skips_missing_install_path() {
        let td = tempfile::tempdir().unwrap();
        write_registry(td.path(), &[("ghost@x", "/does/not/exist", "1")]);
        let plugins = list(td.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn enabled_reflects_settings() {
        let td = tempfile::tempdir().unwrap();
        let install = td.path().join("cache/m/p/1");
        std::fs::create_dir_all(install.join(".claude-plugin")).unwrap();
        std::fs::write(install.join(".claude-plugin/plugin.json"), r#"{"name":"P"}"#).unwrap();
        write_registry(td.path(), &[("p@m", install.to_str().unwrap(), "1")]);
        std::fs::write(
            td.path().join(SETTINGS_FILE),
            r#"{"enabledPlugins":{"p@m":false}}"#,
        )
        .unwrap();

        let plugins = list(td.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert!(!plugins[0].enabled);
    }

    #[test]
    fn set_enabled_round_trip() {
        let td = tempfile::tempdir().unwrap();
        let install = td.path().join("cache/m/p/1");
        std::fs::create_dir_all(&install).unwrap();
        write_registry(td.path(), &[("p@m", install.to_str().unwrap(), "1")]);
        set_enabled(td.path(), "p@m", false).unwrap();
        let raw = std::fs::read_to_string(td.path().join(SETTINGS_FILE)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["enabledPlugins"]["p@m"], false);
    }

    #[test]
    fn delete_removes_install_and_registry_entry() {
        let td = tempfile::tempdir().unwrap();
        let install = td.path().join("cache/m/p/1");
        std::fs::create_dir_all(&install).unwrap();
        write_registry(td.path(), &[("p@m", install.to_str().unwrap(), "1")]);
        delete(td.path(), "p@m").unwrap();
        assert!(!install.exists());
        let raw =
            std::fs::read_to_string(td.path().join(PLUGINS_SUBDIR).join(REGISTRY_FILE)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert!(v["plugins"].get("p@m").is_none());
    }
}
