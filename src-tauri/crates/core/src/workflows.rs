//! Read+write logic for `~/.claude/workflows/`. Plain JavaScript scripts whose
//! first statement is an `export const meta = { name, description, phases }`
//! object literal (see the dynamic-workflows docs). The meta block is parsed
//! for list display only — the file on disk stays the source of truth.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::io;
use crate::types::{Workflow, WorkflowInput};
use crate::AppError;

const WORKFLOWS_SUBDIR: &str = "workflows";

pub fn list(claude_dir: &Path) -> Result<Vec<Workflow>, AppError> {
    let root = claude_dir.join(WORKFLOWS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("js") {
            continue;
        }
        match read_one(&path) {
            Ok(w) => out.push(w),
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "skipping unreadable workflow")
            }
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

pub fn get(claude_dir: &Path, slug: &str) -> Result<Workflow, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|w| w.slug == slug)
        .ok_or_else(|| AppError::not_found(format!("workflow '{slug}' not found")))
}

fn read_one(path: &Path) -> Result<Workflow, AppError> {
    let body = std::fs::read_to_string(path)?;
    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let meta = parse_meta(&body);
    let meta_dir = std::fs::metadata(path)?;
    let modified_at = meta_dir
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    Ok(Workflow {
        name: meta.name.unwrap_or_else(|| slug.clone()),
        description: meta.description.unwrap_or_default(),
        phases: meta.phases,
        slug,
        filename,
        body,
        file_path: path.to_string_lossy().into_owned(),
        size_bytes: meta_dir.len(),
        modified_at,
    })
}

pub fn create(claude_dir: &Path, input: WorkflowInput) -> Result<Workflow, AppError> {
    io::validate_slug(&input.slug)?;
    let path = file_path(claude_dir, &input.slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "workflow '{}' already exists",
            input.slug
        )));
    }
    io::atomic_write(&path, input.body.as_bytes())?;
    get(claude_dir, &input.slug)
}

pub fn update(claude_dir: &Path, slug: &str, input: WorkflowInput) -> Result<Workflow, AppError> {
    io::validate_slug(&input.slug)?;
    let existing = get(claude_dir, slug)?;
    let new_path = file_path(claude_dir, &input.slug);
    let old_path = PathBuf::from(&existing.file_path);
    if old_path != new_path && new_path.exists() {
        return Err(AppError::invalid(format!(
            "target slug '{}' already exists",
            input.slug
        )));
    }
    io::atomic_write(&new_path, input.body.as_bytes())?;
    if old_path != new_path {
        let _ = io::remove_file(&old_path);
    }
    get(claude_dir, &input.slug)
}

pub fn delete(claude_dir: &Path, slug: &str) -> Result<(), AppError> {
    let existing = get(claude_dir, slug)?;
    io::remove_file(Path::new(&existing.file_path))
}

fn file_path(claude_dir: &Path, slug: &str) -> PathBuf {
    claude_dir.join(WORKFLOWS_SUBDIR).join(format!("{slug}.js"))
}

#[derive(Default)]
struct Meta {
    name: Option<String>,
    description: Option<String>,
    phases: Vec<String>,
}

/// Best-effort read of the `export const meta = { … }` literal. Anything the
/// script computes at runtime is not resolvable here (and Claude Code itself
/// drops such a workflow from autocomplete), so unparseable fields stay `None`.
fn parse_meta(src: &str) -> Meta {
    let Some(block) = meta_block(src) else {
        return Meta::default();
    };
    Meta {
        name: string_field(&block, "name"),
        description: string_field(&block, "description"),
        phases: phase_titles(&block),
    }
}

/// The `{ … }` following `export const meta =`, brace-matched and with string
/// and comment contents skipped so a `}` inside them doesn't end the block.
fn meta_block(src: &str) -> Option<String> {
    let re = regex::Regex::new(r"export\s+const\s+meta\s*=\s*\{").ok()?;
    let m = re.find(src)?;
    let bytes = src.as_bytes();
    let open = m.end() - 1; // index of the `{`
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(src[open..=i].to_string());
                }
            }
            q @ (b'\'' | b'"' | b'`') => {
                i = skip_string(bytes, i, q);
            }
            b'/' if i + 1 < bytes.len() => match bytes[i + 1] {
                b'/' => {
                    i = bytes[i..]
                        .iter()
                        .position(|&c| c == b'\n')
                        .map(|p| i + p)
                        .unwrap_or(bytes.len());
                }
                b'*' => {
                    i = src[i + 2..]
                        .find("*/")
                        .map(|p| i + 2 + p + 1)
                        .unwrap_or(bytes.len());
                }
                _ => {}
            },
            _ => {}
        }
        i += 1;
    }
    None
}

/// Index of the closing quote of the string starting at `start`, or the last
/// index when unterminated.
fn skip_string(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            c if c == quote => return i,
            _ => {}
        }
        i += 1;
    }
    bytes.len() - 1
}

/// `key: '…'` / `key: "…"` / `key: `…`` at any depth inside the block.
fn string_field(block: &str, key: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(
        r#"(?s)\b{key}\s*:\s*(?:'([^'\\]*(?:\\.[^'\\]*)*)'|"([^"\\]*(?:\\.[^"\\]*)*)"|`([^`\\]*(?:\\.[^`\\]*)*)`)"#
    ))
    .ok()?;
    let caps = re.captures(block)?;
    let raw = (1..=3).find_map(|i| caps.get(i)).map(|m| m.as_str())?;
    Some(unescape(raw))
}

/// Every `title` in the `phases: [ … ]` array, in declaration order.
fn phase_titles(block: &str) -> Vec<String> {
    let Ok(open) = regex::Regex::new(r"\bphases\s*:\s*\[") else {
        return vec![];
    };
    let Some(m) = open.find(block) else {
        return vec![];
    };
    let rest = &block[m.end()..];
    let end = rest.find(']').unwrap_or(rest.len());
    let array = &rest[..end];
    let Ok(title) = regex::Regex::new(
        r#"(?s)\btitle\s*:\s*(?:'([^'\\]*(?:\\.[^'\\]*)*)'|"([^"\\]*(?:\\.[^"\\]*)*)"|`([^`\\]*(?:\\.[^`\\]*)*)`)"#,
    ) else {
        return vec![];
    };
    title
        .captures_iter(array)
        .filter_map(|c| (1..=3).find_map(|i| c.get(i)).map(|m| unescape(m.as_str())))
        .collect()
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = r#"export const meta = {
  name: 'audit-routes',
  description: 'Audit every route handler for missing auth checks',
  phases: [{ title: 'Review' }, { title: 'Verify', detail: 'cross-check' }],
}

const found = await agent('List every .ts file under src/routes/. } not a brace')
return found
"#;

    #[test]
    fn parses_meta() {
        let meta = parse_meta(SCRIPT);
        assert_eq!(meta.name.as_deref(), Some("audit-routes"));
        assert_eq!(
            meta.description.as_deref(),
            Some("Audit every route handler for missing auth checks")
        );
        assert_eq!(meta.phases, vec!["Review", "Verify"]);
    }

    #[test]
    fn meta_absent_is_tolerated() {
        let meta = parse_meta("const x = 1\n");
        assert!(meta.name.is_none());
        assert!(meta.description.is_none());
        assert!(meta.phases.is_empty());
    }

    #[test]
    fn list_and_get() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("workflows");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("audit-routes.js"), SCRIPT).unwrap();
        std::fs::write(dir.join("bare.js"), "return 1\n").unwrap();
        std::fs::write(dir.join("notes.md"), "ignored").unwrap();

        let all = list(td.path()).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(get(td.path(), "audit-routes").unwrap().name, "audit-routes");
        let bare = get(td.path(), "bare").unwrap();
        assert_eq!(bare.name, "bare");
        assert_eq!(bare.description, "");
    }

    #[test]
    fn write_round_trip() {
        let td = tempfile::tempdir().unwrap();
        create(
            td.path(),
            WorkflowInput {
                slug: "draft".into(),
                body: SCRIPT.into(),
            },
        )
        .unwrap();
        assert_eq!(get(td.path(), "draft").unwrap().filename, "draft.js");

        update(
            td.path(),
            "draft",
            WorkflowInput {
                slug: "final".into(),
                body: SCRIPT.into(),
            },
        )
        .unwrap();
        assert!(get(td.path(), "draft").is_err());
        assert_eq!(get(td.path(), "final").unwrap().slug, "final");

        delete(td.path(), "final").unwrap();
        assert!(get(td.path(), "final").is_err());
    }
}
