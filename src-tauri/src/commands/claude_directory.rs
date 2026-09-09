use app_core::types::{ClaudeDirEntry, ClaudeDirTree};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

/// Explorer tree for `~/.claude` plus, when a project path is supplied, that
/// project's `.claude`.
#[tauri::command]
pub async fn claude_directory_tree(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<Vec<ClaudeDirTree>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let project = project_path
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .map(app_core::claude_directory::resolve_project_dir);
    app_core::claude_directory::trees(&claude_dir, project.as_deref())
}

/// Immediate children of one directory in the tree, fetched when a node is
/// expanded. The core layer refuses any path outside the `.claude` trees.
#[tauri::command]
pub async fn claude_directory_children(
    state: State<'_, AppState>,
    path: String,
    project_path: Option<String>,
) -> Result<Vec<ClaudeDirEntry>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let project = project_path
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .map(app_core::claude_directory::resolve_project_dir);
    app_core::claude_directory::children(
        &claude_dir,
        project.as_deref(),
        std::path::Path::new(&path),
    )
}
