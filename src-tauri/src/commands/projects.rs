use std::path::PathBuf;

use app_core::types::{FileNode, Project, ProjectInfo};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn projects_list(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::list(&claude_dir)
}

#[tauri::command]
pub async fn projects_get(state: State<'_, AppState>, name: String) -> Result<Project, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::get(&claude_dir, &name)
}

#[tauri::command]
pub async fn projects_resolve(path: String) -> Result<ProjectInfo, AppError> {
    Ok(app_core::projects::resolve(&PathBuf::from(path)))
}

#[tauri::command]
pub async fn projects_files(
    state: State<'_, AppState>,
    name: String,
    sub_path: Option<String>,
) -> Result<Vec<FileNode>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::files(&claude_dir, &name, sub_path.as_deref())
}
