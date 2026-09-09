//! Thin git2 wrapper for the project page's git status panel. Read-only:
//! we surface what HEAD/branch/upstream/changed-files look like, never
//! mutate the working tree from here.

use std::path::Path;

use git2::{Repository, StatusOptions};

use crate::types::{GitFileStatus, GitStatus, WorktreeInclude, WorktreeInfo};
use crate::{AppError, ErrorCode};

pub fn status(working_dir: &Path) -> Result<GitStatus, AppError> {
    let repo = match Repository::discover(working_dir) {
        Ok(r) => r,
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            return Ok(GitStatus {
                branch: None,
                upstream: None,
                ahead: 0,
                behind: 0,
                files: vec![],
                clean: true,
            });
        }
        Err(e) => return Err(AppError::new(ErrorCode::Git, e.message().to_string())),
    };

    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    let (upstream, ahead, behind) = if let Some(h) = &head {
        if let Ok(local) = h.target().ok_or_else(|| ()).map_err(|_| ()) {
            let local_oid = local;
            let upstream_name = h
                .name()
                .and_then(|n| repo.branch_upstream_name(n).ok())
                .and_then(|s| s.as_str().map(|s| s.to_string()));
            let mut ahead = 0u32;
            let mut behind = 0u32;
            let mut up_short = None;
            if let Some(name) = upstream_name {
                if let Ok(reference) = repo.find_reference(&name) {
                    if let Some(target) = reference.target() {
                        if let Ok((a, b)) = repo.graph_ahead_behind(local_oid, target) {
                            ahead = a as u32;
                            behind = b as u32;
                        }
                    }
                    up_short = reference.shorthand().map(|s| s.to_string());
                }
            }
            (up_short, ahead, behind)
        } else {
            (None, 0, 0)
        }
    } else {
        (None, 0, 0)
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).renames_head_to_index(true);
    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| AppError::new(ErrorCode::Git, e.message().to_string()))?;
    let mut files = Vec::with_capacity(statuses.len());
    for s in statuses.iter() {
        let path = s.path().unwrap_or_default().to_string();
        let st = s.status();
        let (label, staged) = label_for(st);
        files.push(GitFileStatus {
            path,
            status: label,
            staged,
        });
    }

    let clean = files.is_empty();
    Ok(GitStatus {
        branch,
        upstream,
        ahead,
        behind,
        files,
        clean,
    })
}

fn label_for(st: git2::Status) -> (String, bool) {
    let staged = st.is_index_new()
        || st.is_index_modified()
        || st.is_index_deleted()
        || st.is_index_renamed()
        || st.is_index_typechange();
    let label = if st.is_wt_new() || st.is_index_new() {
        "??"
    } else if st.is_wt_modified() || st.is_index_modified() {
        "M"
    } else if st.is_wt_deleted() || st.is_index_deleted() {
        "D"
    } else if st.is_wt_renamed() || st.is_index_renamed() {
        "R"
    } else if st.is_conflicted() {
        "U"
    } else {
        "?"
    };
    (label.to_string(), staged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_repo_returns_clean() {
        let td = tempfile::tempdir().unwrap();
        let st = status(td.path()).unwrap();
        assert!(st.clean);
        assert!(st.branch.is_none());
    }
}

/// Every worktree of the repository `working_dir` belongs to.
///
/// libgit2 lists only linked worktrees, so the main checkout is added
/// explicitly — omitting the directory you are standing in would make the list
/// misleading. `status` is deliberately not computed here: running it across N
/// worktrees is expensive, and the caller can ask per worktree.
pub fn worktrees(working_dir: &Path) -> Result<Vec<WorktreeInfo>, AppError> {
    let repo = match Repository::discover(working_dir) {
        Ok(r) => r,
        Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(vec![]),
        Err(e) => return Err(AppError::new(ErrorCode::Git, e.message().to_string())),
    };

    let current = working_dir.canonicalize().unwrap_or_else(|_| working_dir.to_path_buf());
    let main_path = repo
        .workdir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo.path().to_path_buf());

    let mut out = vec![WorktreeInfo {
        name: main_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("main")
            .to_string(),
        branch: head_branch(&repo),
        head: head_short_id(&repo),
        is_main: true,
        is_current: canonical(&main_path) == current,
        is_locked: false,
        lock_reason: None,
        prunable: false,
        path: main_path.to_string_lossy().into_owned(),
    }];

    let names = match repo.worktrees() {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(error = %e, "cannot list worktrees");
            return Ok(out);
        }
    };
    for name in names.iter().flatten() {
        let Ok(wt) = repo.find_worktree(name) else {
            continue;
        };
        let path = wt.path().to_path_buf();
        // A worktree whose directory is gone is still listed: that is exactly
        // the state `git worktree prune` exists for, and it should be visible.
        let lock = wt.is_locked().ok();
        let (is_locked, lock_reason) = match lock {
            Some(git2::WorktreeLockStatus::Locked(reason)) => (true, reason),
            _ => (false, None),
        };
        let linked = Repository::open_from_worktree(&wt).ok();

        out.push(WorktreeInfo {
            name: name.to_string(),
            branch: linked.as_ref().and_then(head_branch),
            head: linked.as_ref().and_then(head_short_id),
            is_main: false,
            is_current: canonical(&path) == current,
            is_locked,
            lock_reason,
            prunable: !path.is_dir(),
            path: path.to_string_lossy().into_owned(),
        });
    }

    Ok(out)
}

/// Parse `<project>/.worktreeinclude` and show which files it actually covers.
///
/// The file lists gitignored paths that should still be copied into a new
/// worktree, in gitignore syntax — so `ignore`'s GitignoreBuilder parses it
/// rather than a plain glob matcher, which would mishandle `!` negation and
/// directory semantics.
pub fn worktree_include(working_dir: &Path) -> Result<WorktreeInclude, AppError> {
    const MAX_MATCHES: usize = 500;

    let path = working_dir.join(".worktreeinclude");
    if !path.is_file() {
        return Ok(WorktreeInclude {
            path: path.to_string_lossy().into_owned(),
            exists: false,
            patterns: vec![],
            matched_files: vec![],
            truncated: false,
        });
    }

    let raw = std::fs::read_to_string(&path)?;
    let patterns: Vec<String> = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();

    let mut builder = ignore::gitignore::GitignoreBuilder::new(working_dir);
    for p in &patterns {
        let _ = builder.add_line(None, p);
    }
    let matcher = builder
        .build()
        .map_err(|e| AppError::new(ErrorCode::Git, format!("invalid .worktreeinclude: {e}")))?;

    let mut matched_files = Vec::new();
    let mut truncated = false;
    for entry in walkdir::WalkDir::new(working_dir)
        .max_depth(6)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git" && e.file_name() != "node_modules")
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if matcher.matched(entry.path(), false).is_ignore() {
            if matched_files.len() >= MAX_MATCHES {
                truncated = true;
                break;
            }
            matched_files.push(
                entry
                    .path()
                    .strip_prefix(working_dir)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    matched_files.sort();

    Ok(WorktreeInclude {
        path: path.to_string_lossy().into_owned(),
        exists: true,
        patterns,
        matched_files,
        truncated,
    })
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn head_branch(repo: &Repository) -> Option<String> {
    repo.head().ok()?.shorthand().map(str::to_string)
}

fn head_short_id(repo: &Repository) -> Option<String> {
    let oid = repo.head().ok()?.target()?;
    Some(oid.to_string()[..7.min(oid.to_string().len())].to_string())
}

#[cfg(test)]
mod worktree_tests {
    use super::*;

    fn init_repo(dir: &Path) -> Repository {
        let repo = Repository::init(dir).unwrap();
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("a.txt")).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let sig = git2::Signature::now("t", "t@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
        drop(tree);
        repo
    }

    #[test]
    fn a_non_repo_has_no_worktrees() {
        let td = tempfile::tempdir().unwrap();
        assert!(worktrees(td.path()).unwrap().is_empty());
    }

    #[test]
    fn the_main_checkout_is_listed_and_flagged() {
        let td = tempfile::tempdir().unwrap();
        init_repo(td.path());

        let wts = worktrees(td.path()).unwrap();
        assert_eq!(wts.len(), 1, "libgit2 omits the main checkout; we add it back");
        assert!(wts[0].is_main);
        assert!(wts[0].is_current);
        assert!(!wts[0].prunable);
        assert!(wts[0].head.is_some());
        // The default branch name depends on the machine's git config.
        assert!(matches!(wts[0].branch.as_deref(), Some("main") | Some("master")));
    }

    #[test]
    fn linked_worktrees_are_listed_with_their_branch() {
        let td = tempfile::tempdir().unwrap();
        let repo = init_repo(td.path());
        let wt_path = td.path().join("linked");
        repo.worktree("feature", &wt_path, None).unwrap();

        let wts = worktrees(td.path()).unwrap();
        assert_eq!(wts.len(), 2);
        let linked = wts.iter().find(|w| !w.is_main).unwrap();
        assert_eq!(linked.name, "feature");
        assert!(!linked.is_current, "we asked about the main checkout");
        assert!(linked.path.ends_with("linked"));
        assert!(linked.head.is_some());

        // Asking from inside the linked worktree flips is_current.
        let from_linked = worktrees(&wt_path).unwrap();
        assert!(from_linked.iter().find(|w| !w.is_main).unwrap().is_current);
    }

    #[test]
    fn a_missing_worktree_directory_is_reported_as_prunable() {
        let td = tempfile::tempdir().unwrap();
        let repo = init_repo(td.path());
        let wt_path = td.path().join("gone");
        repo.worktree("stale", &wt_path, None).unwrap();
        std::fs::remove_dir_all(&wt_path).unwrap();

        let wts = worktrees(td.path()).unwrap();
        let stale = wts.iter().find(|w| w.name == "stale").unwrap();
        assert!(stale.prunable, "the admin files remain; the checkout is gone");
    }

    #[test]
    fn worktree_include_is_absent_without_error() {
        let td = tempfile::tempdir().unwrap();
        let inc = worktree_include(td.path()).unwrap();
        assert!(!inc.exists);
        assert!(inc.patterns.is_empty() && inc.matched_files.is_empty());
    }

    #[test]
    fn worktree_include_matches_gitignore_syntax() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join(".worktreeinclude"),
            "# secrets a new worktree still needs\n.env\n*.local.json\n!ignored.local.json\n",
        )
        .unwrap();
        std::fs::write(td.path().join(".env"), "X=1").unwrap();
        std::fs::write(td.path().join("settings.local.json"), "{}").unwrap();
        std::fs::write(td.path().join("ignored.local.json"), "{}").unwrap();
        std::fs::write(td.path().join("README.md"), "hi").unwrap();

        let inc = worktree_include(td.path()).unwrap();
        assert!(inc.exists);
        // Comments and blank lines are dropped; the negation is kept.
        assert_eq!(inc.patterns.len(), 3);
        assert!(inc.matched_files.contains(&".env".to_string()));
        assert!(inc.matched_files.contains(&"settings.local.json".to_string()));
        // gitignore semantics: the negation wins, which a plain glob matcher
        // would have gotten wrong.
        assert!(!inc.matched_files.contains(&"ignored.local.json".to_string()));
        assert!(!inc.matched_files.contains(&"README.md".to_string()));
        assert!(!inc.truncated);
    }
}
