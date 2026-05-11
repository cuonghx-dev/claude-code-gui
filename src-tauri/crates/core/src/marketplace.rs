//! Marketplace registry — Claude Code's canonical store, plus install/uninstall.
//!
//! On disk:
//! - `~/.claude/plugins/known_marketplaces.json` — map of `<name>` →
//!   `{ source, installLocation, lastUpdated }`. This file is owned by
//!   Claude Code itself; we read it directly so the GUI mirrors the CLI's
//!   view of configured marketplaces.
//! - `<installLocation>/.claude-plugin/marketplace.json` — the cached plugin
//!   catalog for a single source.
//!
//! Source types:
//! - `github`: `repo` field is `owner/name`; we surface `https://github.com/<repo>`
//!   as the URL.
//! - `http`: `url` field is an HTTP(S) endpoint returning a JSON manifest.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};

use crate::io;
use crate::types::{AvailablePlugin, MarketplaceSource, MarketplaceSourceInput};
use crate::{AppError, ErrorCode};

const KNOWN_FILE: &str = "plugins/known_marketplaces.json";
const PLUGINS_SUBDIR: &str = "plugins";

/// Progress callback supplied by the IPC layer. Steps are advisory text
/// like `"cloning"`, `"installing"`, `"done"`.
pub type Progress = Arc<dyn Fn(&str, Option<u32>) + Send + Sync + 'static>;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct KnownEntry {
    source: KnownSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    install_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_updated: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auto_update: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
struct KnownSource {
    source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

fn read_known(claude_dir: &Path) -> Result<BTreeMap<String, KnownEntry>, AppError> {
    let path = claude_dir.join(KNOWN_FILE);
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

fn write_known(claude_dir: &Path, map: &BTreeMap<String, KnownEntry>) -> Result<(), AppError> {
    let path = claude_dir.join(KNOWN_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(map)?;
    io::atomic_write(&path, serialized.as_bytes())
}

pub fn sources_list(claude_dir: &Path) -> Result<Vec<MarketplaceSource>, AppError> {
    let map = read_known(claude_dir)?;
    let mut sources: Vec<MarketplaceSource> = map
        .into_iter()
        .map(|(name, e)| build_source(name, e))
        .collect();
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sources)
}

fn build_source(name: String, e: KnownEntry) -> MarketplaceSource {
    let (source_type, url) = match e.source.source.as_str() {
        "github" => (
            "github".to_string(),
            e.source
                .repo
                .as_ref()
                .map(|r| format!("https://github.com/{r}"))
                .unwrap_or_default(),
        ),
        "http" => ("http".to_string(), e.source.url.unwrap_or_default()),
        other => (other.to_string(), e.source.url.unwrap_or_default()),
    };
    let plugins = e
        .install_location
        .as_deref()
        .and_then(|p| read_marketplace_cache(Path::new(p), &name).ok())
        .unwrap_or_default();
    MarketplaceSource {
        name,
        source_type,
        url,
        plugins,
        last_updated: e.last_updated,
    }
}

fn read_marketplace_cache(install: &Path, source_name: &str) -> Result<Vec<AvailablePlugin>, AppError> {
    let path = install.join(".claude-plugin").join("marketplace.json");
    if !path.is_file() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&path)?;
    parse_claude_marketplace(&raw, source_name)
}

#[derive(serde::Deserialize)]
struct ClaudeMarketplace {
    #[serde(default)]
    plugins: Vec<ClaudePlugin>,
}

#[derive(serde::Deserialize)]
struct ClaudePlugin {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

fn parse_claude_marketplace(raw: &str, source_name: &str) -> Result<Vec<AvailablePlugin>, AppError> {
    let m: ClaudeMarketplace = serde_json::from_str(raw)?;
    Ok(m.plugins
        .into_iter()
        .map(|p| AvailablePlugin {
            id: format!("{}@{}", p.name, source_name),
            name: p.name,
            description: p.description,
            version: p.version,
            source: source_name.to_string(),
            install_url: p.source.unwrap_or_default(),
        })
        .collect())
}

pub fn sources_add(claude_dir: &Path, input: MarketplaceSourceInput) -> Result<(), AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::invalid("source name cannot be empty"));
    }
    let mut map = read_known(claude_dir)?;
    if map.contains_key(name) {
        return Err(AppError::invalid(format!(
            "source '{name}' already exists"
        )));
    }
    let source = match input.source_type.as_str() {
        "github" => KnownSource {
            source: "github".into(),
            repo: parse_github_repo(&input.url),
            url: None,
        },
        "http" => KnownSource {
            source: "http".into(),
            repo: None,
            url: Some(input.url.clone()),
        },
        other => {
            return Err(AppError::invalid(format!(
                "sourceType must be 'github' or 'http', got '{other}'"
            )));
        }
    };
    map.insert(
        name.to_string(),
        KnownEntry {
            source,
            install_location: None,
            last_updated: None,
            auto_update: None,
        },
    );
    write_known(claude_dir, &map)
}

/// Extract `owner/name` from a github URL. Accepts `git@github.com:o/n`,
/// `https://github.com/o/n(.git)`, or a bare `o/n`.
fn parse_github_repo(url: &str) -> Option<String> {
    let stripped = url
        .trim()
        .trim_end_matches('/')
        .strip_suffix(".git")
        .unwrap_or_else(|| url.trim().trim_end_matches('/'));
    if let Some(rest) = stripped.strip_prefix("git@github.com:") {
        return Some(rest.to_string());
    }
    if let Some(rest) = stripped.strip_prefix("https://github.com/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = stripped.strip_prefix("http://github.com/") {
        return Some(rest.to_string());
    }
    if stripped.matches('/').count() == 1 && !stripped.contains(' ') {
        return Some(stripped.to_string());
    }
    None
}

pub fn sources_remove(claude_dir: &Path, name: &str) -> Result<(), AppError> {
    let mut map = read_known(claude_dir)?;
    let entry = map
        .remove(name)
        .ok_or_else(|| AppError::not_found(format!("source '{name}' not found")))?;
    if let Some(loc) = entry.install_location.as_deref() {
        let p = PathBuf::from(loc);
        if p.is_dir() {
            let _ = io::remove_dir_all(&p);
        }
    }
    write_known(claude_dir, &map)
}

/// Refresh `lastUpdated` on a source by re-reading the cached
/// `marketplace.json` from its install location. We don't pull the upstream
/// repo here — Claude Code itself owns the clone and refresh cadence.
pub async fn sources_update(claude_dir: &Path, name: &str) -> Result<(), AppError> {
    let mut map = read_known(claude_dir)?;
    let entry = map
        .get_mut(name)
        .ok_or_else(|| AppError::not_found(format!("source '{name}' not found")))?;
    entry.last_updated = Some(chrono::Utc::now().to_rfc3339());
    write_known(claude_dir, &map)
}

/// Aggregate of `AvailablePlugin`s across every source, deduped by id.
pub fn available(claude_dir: &Path) -> Result<Vec<AvailablePlugin>, AppError> {
    let sources = sources_list(claude_dir)?;
    let mut out = Vec::new();
    for s in sources {
        out.extend(s.plugins);
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Install a plugin from `install_url` (a git URL) into
/// `<claude_dir>/plugins/<id>/`. Kept for the legacy custom-marketplace
/// flow; Claude Code's own marketplaces install via the CLI, not via this
/// path.
pub fn install(
    claude_dir: &Path,
    id: &str,
    install_url: &str,
    progress: Progress,
) -> Result<(), AppError> {
    io::validate_slug(id)?;
    let dest = claude_dir.join(PLUGINS_SUBDIR).join(id);
    if dest.exists() {
        return Err(AppError::invalid(format!(
            "plugin '{id}' already installed"
        )));
    }
    progress("cloning", Some(0));

    let mut callbacks = RemoteCallbacks::new();
    let progress_for_cb = Arc::clone(&progress);
    callbacks.transfer_progress(move |stats| {
        let total = stats.total_objects().max(1) as u64;
        let received = stats.received_objects() as u64;
        let pct = ((received * 95) / total).min(95) as u32;
        progress_for_cb("cloning", Some(pct));
        true
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    std::fs::create_dir_all(dest.parent().unwrap_or(Path::new("/")))?;
    RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(install_url, &dest)
        .map_err(|e| AppError::new(ErrorCode::Git, e.message().to_string()))?;

    progress("installing", Some(95));
    let manifest_path = dest.join("plugin.json");
    if !manifest_path.is_file() {
        let manifest = serde_json::json!({
            "name": id,
            "version": null,
            "description": null,
            "enabled": true,
        });
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    }
    progress("done", Some(100));
    Ok(())
}

pub fn uninstall(claude_dir: &Path, id: &str) -> Result<(), AppError> {
    io::validate_slug(id)?;
    let dest = claude_dir.join(PLUGINS_SUBDIR).join(id);
    if !dest.is_dir() {
        return Err(AppError::not_found(format!("plugin '{id}' not installed")));
    }
    io::remove_dir_all(&dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_known(claude_dir: &Path, entries: &[(&str, &str, &str)]) {
        let mut map = serde_json::Map::new();
        for (name, kind, repo_or_url) in entries {
            let source = if *kind == "github" {
                serde_json::json!({ "source": "github", "repo": repo_or_url })
            } else {
                serde_json::json!({ "source": "http", "url": repo_or_url })
            };
            map.insert(
                (*name).to_string(),
                serde_json::json!({
                    "source": source,
                    "lastUpdated": "2026-01-01T00:00:00Z",
                }),
            );
        }
        let path = claude_dir.join(KNOWN_FILE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&map).unwrap()).unwrap();
    }

    #[test]
    fn list_reads_known_marketplaces() {
        let td = tempfile::tempdir().unwrap();
        seed_known(td.path(), &[("foo", "github", "owner/foo")]);
        let sources = sources_list(td.path()).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "foo");
        assert_eq!(sources[0].source_type, "github");
        assert_eq!(sources[0].url, "https://github.com/owner/foo");
        assert_eq!(
            sources[0].last_updated.as_deref(),
            Some("2026-01-01T00:00:00Z"),
        );
    }

    #[test]
    fn add_and_remove_github() {
        let td = tempfile::tempdir().unwrap();
        sources_add(
            td.path(),
            MarketplaceSourceInput {
                name: "official".into(),
                source_type: "github".into(),
                url: "https://github.com/anthropics/plugins.git".into(),
            },
        )
        .unwrap();
        let sources = sources_list(td.path()).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].url, "https://github.com/anthropics/plugins");

        sources_remove(td.path(), "official").unwrap();
        assert!(sources_list(td.path()).unwrap().is_empty());
    }

    #[test]
    fn parse_github_repo_forms() {
        assert_eq!(
            parse_github_repo("https://github.com/o/n").as_deref(),
            Some("o/n"),
        );
        assert_eq!(
            parse_github_repo("https://github.com/o/n.git").as_deref(),
            Some("o/n"),
        );
        assert_eq!(
            parse_github_repo("git@github.com:o/n.git").as_deref(),
            Some("o/n"),
        );
        assert_eq!(parse_github_repo("o/n").as_deref(), Some("o/n"));
        assert!(parse_github_repo("not a repo").is_none());
    }

    #[test]
    fn plugins_from_marketplace_cache() {
        let td = tempfile::tempdir().unwrap();
        let install = td.path().join("plugins/marketplaces/sample");
        std::fs::create_dir_all(install.join(".claude-plugin")).unwrap();
        std::fs::write(
            install.join(".claude-plugin/marketplace.json"),
            r#"{"name":"sample","plugins":[{"name":"alpha","description":"A","source":"./"}]}"#,
        )
        .unwrap();
        let mut map = serde_json::Map::new();
        map.insert(
            "sample".into(),
            serde_json::json!({
                "source": { "source": "github", "repo": "o/sample" },
                "installLocation": install.to_str().unwrap(),
            }),
        );
        std::fs::create_dir_all(td.path().join("plugins")).unwrap();
        std::fs::write(
            td.path().join(KNOWN_FILE),
            serde_json::to_string_pretty(&map).unwrap(),
        )
        .unwrap();
        let sources = sources_list(td.path()).unwrap();
        assert_eq!(sources[0].plugins.len(), 1);
        assert_eq!(sources[0].plugins[0].id, "alpha@sample");
    }
}
