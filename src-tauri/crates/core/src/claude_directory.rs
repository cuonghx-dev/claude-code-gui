//! Explorer for the `.claude` directories — the documented layout
//! (<https://code.claude.com/docs/en/claude-directory>) checked against what is
//! actually on disk.
//!
//! The catalog below mirrors the docs page: every documented entry is returned
//! whether or not it exists, so the UI can show both "what you have" and "what
//! you could add". The catalog is then unioned with a real `read_dir`, because
//! roughly twenty directories that Claude Code writes — `file-history`,
//! `jobs`, `teams`, `cli-history`, `plans`, … — appear in no docs page and
//! were previously invisible here.
//!
//! Children are fetched on demand rather than eagerly, so the tree can go
//! deeper than two levels without every open paying for the whole subtree.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::types::{ClaudeDirEntry, ClaudeDirKind, ClaudeDirScope, ClaudeDirTree};
use crate::{AppError, ErrorCode};

/// Cap on children listed per documented directory — keeps the IPC payload
/// bounded for big trees like `~/.claude/projects/`.
const MAX_CHILDREN: usize = 200;

const DOCS_BASE: &str = "https://code.claude.com/docs/en";

struct CatalogEntry {
    id: &'static str,
    /// Path relative to the tree root.
    rel: &'static str,
    kind: ClaudeDirKind,
    one_liner: &'static str,
    badge: Option<&'static str>,
    docs_path: Option<&'static str>,
}

const fn file(
    id: &'static str,
    rel: &'static str,
    one_liner: &'static str,
    badge: &'static str,
    docs_path: Option<&'static str>,
) -> CatalogEntry {
    CatalogEntry {
        id,
        rel,
        kind: ClaudeDirKind::File,
        one_liner,
        badge: Some(badge),
        docs_path,
    }
}

const fn dir(
    id: &'static str,
    rel: &'static str,
    one_liner: &'static str,
    docs_path: Option<&'static str>,
) -> CatalogEntry {
    CatalogEntry {
        id,
        rel,
        kind: ClaudeDirKind::Dir,
        one_liner,
        badge: None,
        docs_path,
    }
}

/// Rooted at `$HOME`, so the sibling `~/.claude.json` fits in the same tree.
const GLOBAL_CATALOG: &[CatalogEntry] = &[
    file(
        "claude-json",
        ".claude.json",
        "App state and UI preferences",
        "local",
        None,
    ),
    file(
        "global-claude-md",
        ".claude/CLAUDE.md",
        "Personal preferences across every project",
        "local",
        Some("memory"),
    ),
    file(
        "global-settings",
        ".claude/settings.json",
        "Default settings for all projects",
        "local",
        Some("settings"),
    ),
    file(
        "keybindings",
        ".claude/keybindings.json",
        "Custom keyboard shortcuts",
        "local",
        Some("keybindings"),
    ),
    dir(
        "global-rules",
        ".claude/rules",
        "User-level rules that apply to every project",
        Some("rules"),
    ),
    dir(
        "global-skills",
        ".claude/skills",
        "Personal skills available in every project",
        Some("skills"),
    ),
    dir(
        "global-commands",
        ".claude/commands",
        "Personal single-file commands available in every project",
        Some("slash-commands"),
    ),
    dir(
        "global-agents",
        ".claude/agents",
        "Personal subagents available in every project",
        Some("sub-agents"),
    ),
    dir(
        "global-workflows",
        ".claude/workflows",
        "Personal dynamic workflows available in every project",
        None,
    ),
    dir(
        "global-output-styles",
        ".claude/output-styles",
        "Custom system-prompt sections that adjust how Claude works",
        Some("output-styles"),
    ),
    dir(
        "global-plugins",
        ".claude/plugins",
        "Installed plugins and their marketplace sources",
        Some("plugins"),
    ),
    dir("themes", ".claude/themes", "Custom color themes", None),
    dir(
        "global-agent-memory",
        ".claude/agent-memory",
        "Subagent persistent memory, separate from your main session auto memory",
        Some("memory"),
    ),
    dir(
        "global-projects",
        ".claude/projects",
        "Per-project session transcripts and auto memory",
        Some("memory"),
    ),
];

/// Rooted at the project working directory.
const PROJECT_CATALOG: &[CatalogEntry] = &[
    file(
        "claude-md",
        "CLAUDE.md",
        "Project instructions Claude reads every session",
        "committed",
        Some("memory"),
    ),
    file(
        "mcp-json",
        ".mcp.json",
        "Project-scoped MCP servers, shared with your team",
        "committed",
        Some("mcp"),
    ),
    file(
        "worktreeinclude",
        ".worktreeinclude",
        "Gitignored files to copy into new worktrees",
        "committed",
        None,
    ),
    file(
        "settings-json",
        ".claude/settings.json",
        "Permissions, hooks, and configuration",
        "committed",
        Some("settings"),
    ),
    file(
        "settings-local-json",
        ".claude/settings.local.json",
        "Your personal settings overrides for this project",
        "gitignored",
        Some("settings"),
    ),
    dir(
        "rules",
        ".claude/rules",
        "Topic-scoped instructions, optionally gated by file paths",
        Some("rules"),
    ),
    dir(
        "skills",
        ".claude/skills",
        "Reusable prompts you or Claude invoke by name",
        Some("skills"),
    ),
    dir(
        "commands",
        ".claude/commands",
        "Project single-file commands, invoked as /name",
        Some("slash-commands"),
    ),
    dir(
        "agents",
        ".claude/agents",
        "Specialized subagents with their own context window",
        Some("sub-agents"),
    ),
    dir(
        "workflows",
        ".claude/workflows",
        "Dynamic workflow scripts that orchestrate many subagents",
        None,
    ),
    dir(
        "output-styles",
        ".claude/output-styles",
        "Project-scoped output styles, if your team shares any",
        Some("output-styles"),
    ),
    dir(
        "agent-memory",
        ".claude/agent-memory",
        "Subagent persistent memory for this project",
        Some("memory"),
    ),
];

/// Scan the global tree. `claude_dir` is the resolved `~/.claude` — its parent
/// is used as the root so `.claude.json` is reachable.
pub fn global_tree(claude_dir: &Path) -> ClaudeDirTree {
    let root = claude_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| claude_dir.to_path_buf());
    ClaudeDirTree {
        scope: ClaudeDirScope::Global,
        root: root.to_string_lossy().into_owned(),
        exists: claude_dir.is_dir(),
        entries: union_scan(&root, claude_dir, GLOBAL_CATALOG),
    }
}

/// Scan a project working directory.
pub fn project_tree(project_dir: &Path) -> ClaudeDirTree {
    ClaudeDirTree {
        scope: ClaudeDirScope::Project,
        root: project_dir.to_string_lossy().into_owned(),
        exists: project_dir.join(".claude").is_dir(),
        entries: union_scan(project_dir, &project_dir.join(".claude"), PROJECT_CATALOG),
    }
}

/// Both trees at once. `project_dir` is optional — the picker starts empty.
pub fn trees(
    claude_dir: &Path,
    project_dir: Option<&Path>,
) -> Result<Vec<ClaudeDirTree>, AppError> {
    let mut out = vec![global_tree(claude_dir)];
    if let Some(p) = project_dir {
        if !p.is_dir() {
            return Err(AppError::not_found(format!(
                "not a directory: {}",
                p.display()
            )));
        }
        out.push(project_tree(p));
    }
    Ok(out)
}

fn scan(root: &Path, catalog: &[CatalogEntry]) -> Vec<ClaudeDirEntry> {
    catalog.iter().map(|c| entry_for(root, c)).collect()
}

/// Catalog rows first, in catalog order, then whatever else is on disk.
///
/// `disk_root` is the directory actually enumerated: the global tree's entry
/// paths are rooted at $HOME so `~/.claude.json` is reachable, but only
/// `~/.claude` is listed — never the whole home directory.
fn union_scan(
    root: &Path,
    disk_root: &Path,
    catalog: &[CatalogEntry],
) -> Vec<ClaudeDirEntry> {
    let mut out = scan(root, catalog);
    let known: Vec<String> = catalog
        .iter()
        .map(|c| c.rel.rsplit('/').next().unwrap_or(c.rel).to_string())
        .collect();
    for entry in unknown_entries(disk_root, catalog) {
        if !known.contains(&entry.label) {
            out.push(entry);
        }
    }
    out
}

fn entry_for(root: &Path, c: &CatalogEntry) -> ClaudeDirEntry {
    let path = root.join(c.rel);
    let meta = std::fs::metadata(&path).ok();
    let exists = meta.is_some();
    let is_dir = matches!(c.kind, ClaudeDirKind::Dir);
    ClaudeDirEntry {
        id: c.id.to_string(),
        label: c.rel.rsplit('/').next().unwrap_or(c.rel).to_string(),
        path: path.to_string_lossy().into_owned(),
        kind: c.kind,
        exists,
        known: true,
        size_bytes: meta.as_ref().filter(|m| m.is_file()).map(|m| m.len()),
        child_count: if is_dir && exists {
            Some(count_children(&path))
        } else {
            None
        },
        modified_at: meta.as_ref().and_then(modified_rfc3339),
        one_liner: c.one_liner.to_string(),
        badge: c.badge.map(str::to_string),
        docs_url: c.docs_path.map(|p| format!("{DOCS_BASE}/{p}")),
        has_children: is_dir && exists && count_children(&path) > 0,
        truncated: false,
        is_symlink: meta.as_ref().is_some_and(|_| path.is_symlink()),
        children: Vec::new(),
    }
}

/// Immediate children of a path inside one of the trees.
///
/// The containment guard is the important part: `path` arrives from the
/// frontend, so without it this is an arbitrary-filesystem-read IPC. Symlinks
/// are never followed — `~/.claude/plugins` holds symlinked marketplaces, and
/// descending them would walk outside the tree.
pub fn children(
    claude_dir: &Path,
    project_dir: Option<&Path>,
    path: &Path,
) -> Result<Vec<ClaudeDirEntry>, AppError> {
    let mut roots: Vec<PathBuf> = vec![claude_dir.to_path_buf()];
    // The global tree is rooted at $HOME so it can show `~/.claude.json`, but
    // only that one sibling is reachable, never all of $HOME.
    if let Some(home) = claude_dir.parent() {
        roots.push(home.join(".claude.json"));
    }
    if let Some(pd) = project_dir {
        roots.push(pd.to_path_buf());
    }

    let canonical = path
        .canonicalize()
        .map_err(|_| AppError::not_found(format!("{} not found", path.display())))?;
    let contained = roots.iter().any(|r| {
        let root = r.canonicalize().unwrap_or_else(|_| r.clone());
        canonical == root || canonical.starts_with(&root)
    });
    if !contained {
        return Err(AppError::new(
            ErrorCode::PermissionDenied,
            format!("{} is outside the .claude directories", path.display()),
        ));
    }
    if !canonical.is_dir() {
        return Ok(vec![]);
    }
    Ok(list_children(&canonical))
}

/// On-disk entries that no catalog row covers.
fn unknown_entries(root: &Path, catalog: &[CatalogEntry]) -> Vec<ClaudeDirEntry> {
    let Ok(read) = std::fs::read_dir(root) else {
        return vec![];
    };
    let mut out: Vec<ClaudeDirEntry> = read
        .flatten()
        .filter_map(|e| {
            let child = e.path();
            let name = child.file_name()?.to_str()?.to_string();
            if catalog.iter().any(|c| c.rel == name) {
                return None;
            }
            Some(describe_disk_entry(&child, name))
        })
        .collect();
    out.sort_by(|a, b| {
        let a_dir = matches!(a.kind, ClaudeDirKind::Dir);
        let b_dir = matches!(b.kind, ClaudeDirKind::Dir);
        b_dir.cmp(&a_dir).then_with(|| a.label.cmp(&b.label))
    });
    out
}

fn describe_disk_entry(child: &Path, name: String) -> ClaudeDirEntry {
    // symlink_metadata so a symlinked directory is reported as what it is
    // rather than as its target.
    let meta = std::fs::symlink_metadata(child).ok();
    let is_symlink = meta.as_ref().is_some_and(|m| m.file_type().is_symlink());
    let is_dir = child.is_dir() && !is_symlink;
    ClaudeDirEntry {
        id: child.to_string_lossy().into_owned(),
        label: name,
        path: child.to_string_lossy().into_owned(),
        kind: if is_dir { ClaudeDirKind::Dir } else { ClaudeDirKind::File },
        exists: true,
        known: false,
        size_bytes: (!is_dir).then(|| meta.as_ref().map(|m| m.len()).unwrap_or(0)),
        child_count: is_dir.then(|| count_children(child)),
        modified_at: meta.as_ref().and_then(modified_rfc3339),
        one_liner: String::new(),
        badge: None,
        docs_url: None,
        has_children: is_dir && count_children(child) > 0,
        truncated: false,
        is_symlink,
        children: Vec::new(),
    }
}

/// Immediate children of a documented directory, dirs first then files, both
/// alphabetical. Dotfiles are kept — `.gitkeep` and friends matter here.
fn list_children(path: &Path) -> Vec<ClaudeDirEntry> {
    let read = match std::fs::read_dir(path) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "cannot list directory");
            return Vec::new();
        }
    };
    let mut out: Vec<ClaudeDirEntry> = read
        .flatten()
        .filter_map(|e| {
            let child = e.path();
            let name = child.file_name()?.to_str()?.to_string();
            Some(describe_disk_entry(&child, name))
        })
        .collect();
    out.sort_by(|a, b| {
        let a_dir = matches!(a.kind, ClaudeDirKind::Dir);
        let b_dir = matches!(b.kind, ClaudeDirKind::Dir);
        b_dir.cmp(&a_dir).then_with(|| a.label.cmp(&b.label))
    });
    let over_cap = out.len() > MAX_CHILDREN;
    out.truncate(MAX_CHILDREN);
    if over_cap {
        if let Some(last) = out.last_mut() {
            last.truncated = true;
        }
    }
    out
}

fn count_children(path: &Path) -> u32 {
    if path.is_symlink() {
        return 0;
    }
    std::fs::read_dir(path)
        .map(|r| r.flatten().count() as u32)
        .unwrap_or(0)
}

fn modified_rfc3339(meta: &std::fs::Metadata) -> Option<String> {
    let d = meta
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?;
    chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0).map(|dt| dt.to_rfc3339())
}

/// Resolve a project path typed or picked in the UI. Kept here so the command
/// layer stays a thin wrapper.
pub fn resolve_project_dir(input: &str) -> PathBuf {
    crate::files::expand_tilde(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_entries_returned_even_when_missing() {
        let td = tempfile::tempdir().unwrap();
        let tree = project_tree(td.path());
        assert!(!tree.exists);
        assert_eq!(tree.entries.len(), PROJECT_CATALOG.len());
        assert!(tree.entries.iter().all(|e| !e.exists));
        assert!(tree.entries.iter().all(|e| e.known));
    }

    #[test]
    fn existing_files_and_dirs_are_stated() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("CLAUDE.md"), "hello").unwrap();
        std::fs::create_dir_all(td.path().join(".claude/skills/security-review")).unwrap();
        std::fs::write(
            td.path().join(".claude/skills/security-review/SKILL.md"),
            "x",
        )
        .unwrap();
        std::fs::write(td.path().join(".claude/skills/note.md"), "x").unwrap();

        let tree = project_tree(td.path());
        assert!(tree.exists);

        let md = tree.entries.iter().find(|e| e.id == "claude-md").unwrap();
        assert!(md.exists);
        assert_eq!(md.size_bytes, Some(5));

        let skills = tree.entries.iter().find(|e| e.id == "skills").unwrap();
        assert!(skills.exists);
        assert_eq!(skills.child_count, Some(2));
        assert!(skills.has_children);
        // Children arrive through `children()`, not with the tree.
        assert!(skills.children.is_empty());

        let kids = children(&td.path().join(".claude"), Some(td.path()), Path::new(&skills.path))
            .unwrap();
        // Dirs sort before files.
        assert_eq!(kids[0].label, "security-review");
        assert!(!kids[0].known);
        assert_eq!(kids[0].child_count, Some(1));
        assert_eq!(kids[1].label, "note.md");
    }

    #[test]
    fn undocumented_directories_are_listed_as_unknown() {
        let td = tempfile::tempdir().unwrap();
        let claude_dir = td.path().join(".claude");
        std::fs::create_dir_all(claude_dir.join("file-history/session-a")).unwrap();
        std::fs::create_dir_all(claude_dir.join("jobs")).unwrap();
        std::fs::write(claude_dir.join("history.jsonl"), "{}").unwrap();

        let tree = global_tree(&claude_dir);
        let fh = tree.entries.iter().find(|e| e.label == "file-history").unwrap();
        assert!(!fh.known, "no docs page covers it, but it is real and must be visible");
        assert!(fh.exists && fh.has_children);
        assert!(fh.one_liner.is_empty());
        assert!(tree.entries.iter().any(|e| e.label == "jobs"));
        assert!(tree.entries.iter().any(|e| e.label == "history.jsonl"));

        // Catalog rows still come first, and a documented-but-absent entry is
        // still offered.
        assert!(tree.entries[0].known);
        assert!(tree.entries.iter().any(|e| e.known && !e.exists));
    }

    #[test]
    fn children_refuses_paths_outside_the_trees() {
        let td = tempfile::tempdir().unwrap();
        let claude_dir = td.path().join(".claude");
        std::fs::create_dir_all(claude_dir.join("jobs")).unwrap();
        let outside = td.path().join("elsewhere");
        std::fs::create_dir_all(&outside).unwrap();

        assert!(children(&claude_dir, None, &claude_dir.join("jobs")).is_ok());

        // Without this guard the command is an arbitrary filesystem read.
        let err = children(&claude_dir, None, &outside).unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert!(children(&claude_dir, None, Path::new("/etc")).is_err());
        assert!(children(&claude_dir, None, &claude_dir.join("../..")).is_err());
    }

    #[test]
    fn symlinks_are_shown_but_never_descended() {
        let td = tempfile::tempdir().unwrap();
        let claude_dir = td.path().join(".claude");
        std::fs::create_dir_all(claude_dir.join("plugins")).unwrap();
        let target = td.path().join("marketplace");
        std::fs::create_dir_all(target.join("inner")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, claude_dir.join("plugins/linked")).unwrap();

        let kids = children(&claude_dir, None, &claude_dir.join("plugins")).unwrap();
        let link = kids.iter().find(|k| k.label == "linked").unwrap();
        assert!(link.is_symlink);
        assert_eq!(link.child_count, None, "a symlink is not walked");
    }

    #[test]
    fn global_tree_is_rooted_at_home() {
        let td = tempfile::tempdir().unwrap();
        let claude_dir = td.path().join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        std::fs::write(td.path().join(".claude.json"), "{}").unwrap();

        let tree = global_tree(&claude_dir);
        assert!(tree.exists);
        let json = tree.entries.iter().find(|e| e.id == "claude-json").unwrap();
        assert!(json.exists);
        assert_eq!(json.path, td.path().join(".claude.json").to_string_lossy());
    }

    #[test]
    fn project_tree_rejects_missing_dir() {
        let td = tempfile::tempdir().unwrap();
        let missing = td.path().join("nope");
        let err = trees(td.path(), Some(&missing)).unwrap_err();
        assert!(err.to_string().contains("not a directory"));
    }
}
