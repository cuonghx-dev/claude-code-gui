//! Import a skill directory straight from GitHub.
//!
//! The vendored libgit2 is built without HTTPS, so instead of cloning this
//! walks the REST contents API (`/repos/{owner}/{repo}/contents/{path}`) and
//! downloads each file. Only public repos work without a token; `GH_TOKEN` /
//! `GITHUB_TOKEN` is sent when set, which also lifts the 60 req/h limit.
//!
//! The download lands in a hidden sibling of `skills/` and is renamed into
//! place only once complete, so a failed import never leaves half a skill.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::io;
use crate::types::Skill;
use crate::{AppError, ErrorCode};

const API_BASE: &str = "https://api.github.com";
const IMPORTS_FILE: &str = ".imports.json";
/// A skill is a handful of markdown files and scripts; anything past these
/// limits is almost certainly the wrong URL (a whole repo).
const MAX_FILES: usize = 200;
const MAX_TOTAL_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
struct GithubRef {
    owner: String,
    repo: String,
    /// Branch, tag or sha; `None` means the repo's default branch.
    git_ref: Option<String>,
    /// Directory inside the repo, `""` for the root.
    path: String,
}

/// Accepts `github.com/o/r`, `…/tree/<ref>/<dir>`, and `…/blob/<ref>/<dir>/SKILL.md`.
/// Refs containing `/` are not supported (the URL cannot disambiguate them).
fn parse_url(url: &str) -> Result<GithubRef, AppError> {
    let bad = || AppError::invalid(format!("not a GitHub skill URL: '{url}'"));
    let rest = url
        .trim()
        .trim_end_matches('/')
        .strip_prefix("https://github.com/")
        .or_else(|| url.trim().strip_prefix("github.com/"))
        .ok_or_else(bad)?;
    let rest = rest.split(['?', '#']).next().unwrap_or_default();
    let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
    let (owner, repo) = match parts.as_slice() {
        [o, r, ..] => (o.to_string(), r.trim_end_matches(".git").to_string()),
        _ => return Err(bad()),
    };
    let (git_ref, mut path_parts) = match parts.get(2) {
        None => (None, vec![]),
        Some(&"tree") | Some(&"blob") => {
            let r = parts.get(3).ok_or_else(bad)?;
            (Some(r.to_string()), parts[4..].to_vec())
        }
        Some(_) => return Err(bad()),
    };
    if path_parts.last() == Some(&"SKILL.md") {
        path_parts.pop();
    }
    Ok(GithubRef {
        owner,
        repo,
        git_ref,
        path: path_parts.join("/"),
    })
}

#[derive(Deserialize)]
struct ContentEntry {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    path: String,
    #[serde(default)]
    size: u64,
    download_url: Option<String>,
}

struct Fetcher {
    client: reqwest::Client,
    api_base: String,
    token: Option<String>,
}

impl Fetcher {
    fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(url)
            .header("User-Agent", "claude-code-gui")
            .header("Accept", "application/vnd.github+json");
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        req
    }

    async fn list_dir(&self, gh: &GithubRef, path: &str) -> Result<Vec<ContentEntry>, AppError> {
        let mut url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.api_base, gh.owner, gh.repo, path
        );
        if let Some(r) = &gh.git_ref {
            url.push_str("?ref=");
            url.push_str(r);
        }
        let resp = self.get(&url).send().await.map_err(net_err)?;
        match resp.status().as_u16() {
            200 => {}
            404 => {
                return Err(AppError::not_found(format!(
                    "'{}/{}' has no directory '{path}' (or the repo is private)",
                    gh.owner, gh.repo
                )))
            }
            403 | 429 => {
                return Err(AppError::new(
                    ErrorCode::Internal,
                    "GitHub rate limit reached; set GH_TOKEN and try again",
                ))
            }
            s => return Err(net_err(format!("GitHub answered HTTP {s} for {url}"))),
        }
        let body: serde_json::Value = resp.json().await.map_err(net_err)?;
        if !body.is_array() {
            return Err(AppError::invalid(format!(
                "'{path}' is a file; link the skill's directory"
            )));
        }
        serde_json::from_value(body).map_err(|e| net_err(format!("unexpected listing: {e}")))
    }

    /// Every file under `root`, as `(path relative to root, download url)`.
    async fn walk(&self, gh: &GithubRef) -> Result<Vec<(PathBuf, String)>, AppError> {
        let mut files = Vec::new();
        let mut total: u64 = 0;
        let mut pending = vec![gh.path.clone()];
        while let Some(dir) = pending.pop() {
            for e in self.list_dir(gh, &dir).await? {
                match e.kind.as_str() {
                    "dir" => pending.push(e.path),
                    "file" => {
                        let rel = relative(&gh.path, &e.path, &e.name)?;
                        let url = e
                            .download_url
                            .ok_or_else(|| net_err(format!("no download url for {}", e.path)))?;
                        total += e.size;
                        files.push((rel, url));
                    }
                    // symlinks and submodules could point anywhere; skip them.
                    other => tracing::info!(path = %e.path, kind = other, "skipping non-file entry"),
                }
                if files.len() > MAX_FILES || total > MAX_TOTAL_BYTES {
                    return Err(AppError::invalid(format!(
                        "that directory is too big for a skill (limit {MAX_FILES} files / {} MiB)",
                        MAX_TOTAL_BYTES / 1024 / 1024
                    )));
                }
            }
        }
        Ok(files)
    }

    async fn download(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let resp = self.get(url).send().await.map_err(net_err)?;
        if !resp.status().is_success() {
            return Err(net_err(format!("HTTP {} downloading {url}", resp.status())));
        }
        Ok(resp.bytes().await.map_err(net_err)?.to_vec())
    }
}

fn net_err(e: impl std::fmt::Display) -> AppError {
    AppError::new(ErrorCode::Internal, format!("github import: {e}"))
}

/// `entry_path` relative to `root`, refusing anything that is not a plain
/// descendant — the listing comes from the network.
fn relative(root: &str, entry_path: &str, name: &str) -> Result<PathBuf, AppError> {
    let rel = if root.is_empty() {
        entry_path
    } else {
        entry_path
            .strip_prefix(root)
            .and_then(|r| r.strip_prefix('/'))
            .unwrap_or(name)
    };
    let p = PathBuf::from(rel);
    if p.as_os_str().is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(net_err(format!("refusing path '{entry_path}'")));
    }
    Ok(p)
}

pub async fn import(claude_dir: &Path, url: &str) -> Result<Vec<Skill>, AppError> {
    let token = std::env::var("GH_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .ok()
        .filter(|t| !t.is_empty());
    import_from(claude_dir, url, API_BASE, token).await
}

async fn import_from(
    claude_dir: &Path,
    url: &str,
    api_base: &str,
    token: Option<String>,
) -> Result<Vec<Skill>, AppError> {
    let gh = parse_url(url)?;
    let slug = gh
        .path
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(&gh.repo)
        .to_string();
    io::validate_slug(&slug)?;

    let skills_dir = claude_dir.join("skills");
    let dest = skills_dir.join(&slug);
    if dest.exists() {
        return Err(AppError::invalid(format!("skill '{slug}' already exists")));
    }

    let fetcher = Fetcher {
        client: reqwest::Client::new(),
        api_base: api_base.trim_end_matches('/').to_string(),
        token,
    };
    let files = fetcher.walk(&gh).await?;
    if !files.iter().any(|(p, _)| p == Path::new("SKILL.md")) {
        return Err(AppError::invalid(format!(
            "no SKILL.md at the top of '{}' — link the skill's own directory",
            if gh.path.is_empty() { &gh.repo } else { &gh.path }
        )));
    }

    std::fs::create_dir_all(&skills_dir)?;
    let staging = tempfile::Builder::new()
        .prefix(".ccg-import-")
        .tempdir_in(&skills_dir)?;
    for (rel, file_url) in &files {
        let bytes = fetcher.download(file_url).await?;
        let target = staging.path().join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, bytes)?;
    }
    if dest.exists() {
        return Err(AppError::invalid(format!("skill '{slug}' already exists")));
    }
    std::fs::rename(staging.keep(), &dest)?;

    record_import(claude_dir, &slug, url, gh.git_ref.as_deref());
    Ok(vec![crate::skills::get(claude_dir, &slug)?])
}

/// Remember where an imported skill came from (`~/.claude/.imports.json`,
/// SPEC §3). Best-effort: the skill is already installed.
fn record_import(claude_dir: &Path, slug: &str, url: &str, git_ref: Option<&str>) {
    let path = claude_dir.join(IMPORTS_FILE);
    let mut doc: BTreeMap<String, serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    doc.insert(
        slug.to_string(),
        serde_json::json!({
            "source": "github",
            "url": url,
            "ref": git_ref,
            "importedAt": chrono::Utc::now().to_rfc3339(),
        }),
    );
    let written = serde_json::to_string_pretty(&doc)
        .map_err(AppError::from)
        .and_then(|s| io::atomic_write(&path, s.as_bytes()));
    if let Err(e) = written {
        tracing::warn!(error = %e, "could not record skill import");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repo_tree_and_blob_urls() {
        let r = parse_url("https://github.com/acme/skills").unwrap();
        assert_eq!((r.owner.as_str(), r.repo.as_str(), r.git_ref, r.path.as_str()), ("acme", "skills", None, ""));

        let r = parse_url("https://github.com/acme/skills/tree/main/pdf/").unwrap();
        assert_eq!((r.git_ref.as_deref(), r.path.as_str()), (Some("main"), "pdf"));

        let r = parse_url("github.com/acme/skills.git/blob/v1/a/b/SKILL.md?plain=1").unwrap();
        assert_eq!((r.repo.as_str(), r.git_ref.as_deref(), r.path.as_str()), ("skills", Some("v1"), "a/b"));

        for bad in ["https://gitlab.com/a/b", "https://github.com/a", "https://github.com/a/b/issues/1"] {
            assert!(parse_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn relative_paths_cannot_escape() {
        assert_eq!(relative("pdf", "pdf/scripts/x.py", "x.py").unwrap(), PathBuf::from("scripts/x.py"));
        assert_eq!(relative("", "SKILL.md", "SKILL.md").unwrap(), PathBuf::from("SKILL.md"));
        assert!(relative("", "../evil", "evil").is_err());
        assert!(relative("", "/etc/passwd", "passwd").is_err());
    }

    mod server {
        use super::super::*;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        /// Serves a fake contents API for `acme/skills` with a `pdf/` skill.
        async fn fake_github() -> String {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let base = format!("http://{}", listener.local_addr().unwrap());
            let b = base.clone();
            tokio::spawn(async move {
                while let Ok((mut sock, _)) = listener.accept().await {
                    let b = b.clone();
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 8192];
                        let n = sock.read(&mut buf).await.unwrap_or(0);
                        let req = String::from_utf8_lossy(&buf[..n]).to_string();
                        let path = req.split(' ').nth(1).unwrap_or("").to_string();
                        let entry = |kind: &str, p: &str| {
                            let name = p.rsplit('/').next().unwrap();
                            serde_json::json!({ "type": kind, "name": name, "path": p, "size": 10,
                                "download_url": if kind == "file" { serde_json::json!(format!("{b}/raw/{p}")) } else { serde_json::Value::Null } })
                        };
                        let (status, body) = match path.as_str() {
                            "/repos/acme/skills/contents/pdf?ref=main" => ("200 OK", serde_json::json!([
                                entry("file", "pdf/SKILL.md"), entry("dir", "pdf/scripts"), entry("symlink", "pdf/link")
                            ]).to_string()),
                            "/repos/acme/skills/contents/pdf/scripts?ref=main" => ("200 OK", serde_json::json!([
                                entry("file", "pdf/scripts/run.py")
                            ]).to_string()),
                            "/repos/acme/skills/contents/empty?ref=main" => ("200 OK", serde_json::json!([
                                entry("file", "empty/README.md")
                            ]).to_string()),
                            "/raw/pdf/SKILL.md" => ("200 OK", "---\nname: pdf\ndescription: PDF tools\n---\nUse it.\n".to_string()),
                            p if p.starts_with("/raw/") => ("200 OK", "print('hi')\n".to_string()),
                            _ => ("404 Not Found", "{}".to_string()),
                        };
                        let resp = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                        let _ = sock.write_all(resp.as_bytes()).await;
                    });
                }
            });
            base
        }

        #[tokio::test]
        async fn imports_a_skill_directory_atomically() {
            let base = fake_github().await;
            let td = tempfile::tempdir().unwrap();
            let url = "https://github.com/acme/skills/tree/main/pdf";

            let skills = import_from(td.path(), url, &base, None).await.unwrap();
            assert_eq!(skills[0].slug, "pdf");
            let dir = td.path().join("skills/pdf");
            assert!(dir.join("scripts/run.py").is_file());
            assert!(!dir.join("link").exists(), "symlinks are skipped");
            let imports = std::fs::read_to_string(td.path().join(".imports.json")).unwrap();
            assert!(imports.contains(url));

            // Importing again is refused; no staging dirs are left behind.
            assert!(import_from(td.path(), url, &base, None).await.is_err());
            let leftovers: Vec<_> = std::fs::read_dir(td.path().join("skills"))
                .unwrap()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with('.'))
                .collect();
            assert!(leftovers.is_empty());
        }

        #[tokio::test]
        async fn refuses_directories_without_skill_md_and_missing_paths() {
            let base = fake_github().await;
            let td = tempfile::tempdir().unwrap();
            let err = import_from(td.path(), "https://github.com/acme/skills/tree/main/empty", &base, None)
                .await
                .unwrap_err();
            assert!(err.message.contains("SKILL.md"), "{}", err.message);
            assert!(!td.path().join("skills/empty").exists());

            let err = import_from(td.path(), "https://github.com/acme/skills/tree/main/nope", &base, None)
                .await
                .unwrap_err();
            assert_eq!(err.code, ErrorCode::NotFound);
        }
    }
}
