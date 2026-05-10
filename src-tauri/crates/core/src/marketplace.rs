//! Marketplace source registry + install/uninstall.
//!
//! On disk:
//! - `~/.claude/.marketplaces.json` — array of `MarketplaceSource`.
//! - `~/.claude/plugins/<id>/plugin.json` — installed plugin manifest.
//!
//! Source types:
//! - `github`: a git URL. `update` clones to a temp dir and reads
//!   `marketplace.json`; `install(name, install_url)` clones the plugin
//!   repo into `~/.claude/plugins/<id>/`.
//! - `http`: an HTTP(S) URL returning a JSON manifest. `update` GETs the
//!   manifest. `install` downloads each entry's tarball — Phase 3 only
//!   accepts http manifests that point at git repos and reuses the git
//!   clone path; tarball install lands in Phase 6.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};

use crate::io;
use crate::types::{AvailablePlugin, MarketplaceSource, MarketplaceSourceInput};
use crate::{AppError, ErrorCode};

const FILE: &str = ".marketplaces.json";
const PLUGINS_SUBDIR: &str = "plugins";

/// Progress callback supplied by the IPC layer. Steps are advisory text
/// like `"cloning"`, `"installing"`, `"done"`.
pub type Progress = Arc<dyn Fn(&str, Option<u32>) + Send + Sync + 'static>;

pub fn sources_list(claude_dir: &Path) -> Result<Vec<MarketplaceSource>, AppError> {
    let path = claude_dir.join(FILE);
    if !path.is_file() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn sources_add(claude_dir: &Path, input: MarketplaceSourceInput) -> Result<(), AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::invalid("source name cannot be empty"));
    }
    if !matches!(input.source_type.as_str(), "github" | "http") {
        return Err(AppError::invalid("sourceType must be 'github' or 'http'"));
    }
    let mut sources = sources_list(claude_dir)?;
    if sources.iter().any(|s| s.name == input.name) {
        return Err(AppError::invalid(format!(
            "source '{}' already exists",
            input.name
        )));
    }
    sources.push(MarketplaceSource {
        name: input.name,
        source_type: input.source_type,
        url: input.url,
        plugins: vec![],
        last_updated: None,
    });
    write_sources(claude_dir, &sources)
}

pub fn sources_remove(claude_dir: &Path, name: &str) -> Result<(), AppError> {
    let mut sources = sources_list(claude_dir)?;
    let before = sources.len();
    sources.retain(|s| s.name != name);
    if sources.len() == before {
        return Err(AppError::not_found(format!("source '{name}' not found")));
    }
    write_sources(claude_dir, &sources)
}

/// Refresh a single source's plugin manifest. For `github` sources this
/// clones to a tempdir and reads `marketplace.json`; for `http` it GETs
/// the URL and parses as JSON. Both cache the result back into
/// `.marketplaces.json` so subsequent `marketplace_available` calls are
/// offline-friendly.
pub async fn sources_update(claude_dir: &Path, name: &str) -> Result<(), AppError> {
    let mut sources = sources_list(claude_dir)?;
    let pos = sources
        .iter()
        .position(|s| s.name == name)
        .ok_or_else(|| AppError::not_found(format!("source '{name}' not found")))?;

    let plugins = match sources[pos].source_type.as_str() {
        "github" => fetch_github_manifest(&sources[pos].url, &sources[pos].name)?,
        "http" => fetch_http_manifest(&sources[pos].url, &sources[pos].name).await?,
        other => return Err(AppError::invalid(format!("unknown sourceType '{other}'"))),
    };

    sources[pos].plugins = plugins;
    sources[pos].last_updated = Some(chrono::Utc::now().to_rfc3339());
    write_sources(claude_dir, &sources)
}

/// Aggregate of cached `AvailablePlugin`s across every source.
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
/// `<claude_dir>/plugins/<id>/`. The progress callback receives advisory
/// step labels and an optional 0-100 percentage.
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
        let pct = ((received * 95) / total).min(95) as u32; // reserve last 5% for post-clone
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
    // If the cloned repo doesn't have a plugin.json, scaffold a minimal one.
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

fn write_sources(claude_dir: &Path, sources: &[MarketplaceSource]) -> Result<(), AppError> {
    let path = claude_dir.join(FILE);
    let serialized = serde_json::to_string_pretty(sources)?;
    io::atomic_write(&path, serialized.as_bytes())
}

/// Clone the source repo to a tempdir, read `marketplace.json` from its
/// root, and return the parsed plugin list. Used by `sources_update`.
fn fetch_github_manifest(url: &str, source_name: &str) -> Result<Vec<AvailablePlugin>, AppError> {
    let td = tempfile::tempdir().map_err(|e| AppError::new(ErrorCode::IoError, e.to_string()))?;
    git2::Repository::clone(url, td.path())
        .map_err(|e| AppError::new(ErrorCode::Git, e.message().to_string()))?;
    let manifest_path = td.path().join("marketplace.json");
    if !manifest_path.is_file() {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "github source has no marketplace.json at repo root",
        ));
    }
    let raw = std::fs::read_to_string(&manifest_path)?;
    parse_manifest(&raw, source_name)
}

async fn fetch_http_manifest(
    url: &str,
    source_name: &str,
) -> Result<Vec<AvailablePlugin>, AppError> {
    let raw = reqwest::get(url)
        .await
        .map_err(|e| AppError::new(ErrorCode::Network, e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::new(ErrorCode::Network, e.to_string()))?;
    parse_manifest(&raw, source_name)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    install_url: String,
}

fn parse_manifest(raw: &str, source_name: &str) -> Result<Vec<AvailablePlugin>, AppError> {
    let entries: Vec<ManifestEntry> = serde_json::from_str(raw)?;
    Ok(entries
        .into_iter()
        .map(|e| AvailablePlugin {
            id: e.id.clone(),
            name: e.name.unwrap_or_else(|| e.id.clone()),
            description: e.description,
            version: e.version,
            source: source_name.to_string(),
            install_url: e.install_url,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_crud_round_trip() {
        let td = tempfile::tempdir().unwrap();
        sources_add(
            td.path(),
            MarketplaceSourceInput {
                name: "official".into(),
                source_type: "github".into(),
                url: "https://example.com/plugins.git".into(),
            },
        )
        .unwrap();
        let list = sources_list(td.path()).unwrap();
        assert_eq!(list.len(), 1);
        sources_remove(td.path(), "official").unwrap();
        assert!(sources_list(td.path()).unwrap().is_empty());
    }

    #[test]
    fn parse_manifest_basic() {
        let raw = r#"[{"id":"hello","installUrl":"https://x"}]"#;
        let plugins = parse_manifest(raw, "src").unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "hello");
        assert_eq!(plugins[0].source, "src");
    }
}
