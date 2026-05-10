//! Thin git2 wrapper for the project page's git status panel. Read-only:
//! we surface what HEAD/branch/upstream/changed-files look like, never
//! mutate the working tree from here.

use std::path::Path;

use git2::{Repository, StatusOptions};

use crate::types::{GitFileStatus, GitStatus};
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
