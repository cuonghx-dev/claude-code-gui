use std::path::PathBuf;

use app_core::types::{FileNode, GitStatus, Project, ProjectInfo, Settings};
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

#[tauri::command]
pub async fn projects_create(
    state: State<'_, AppState>,
    path: String,
) -> Result<Project, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::create(&claude_dir, &path)
}

#[tauri::command]
pub async fn projects_rename(
    state: State<'_, AppState>,
    name: String,
    new_name: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::rename(&claude_dir, &name, &new_name)
}

#[tauri::command]
pub async fn projects_delete(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::delete(&claude_dir, &name)
}

#[tauri::command]
pub async fn projects_git_status(
    state: State<'_, AppState>,
    name: String,
) -> Result<GitStatus, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let project = app_core::projects::get(&claude_dir, &name)?;
    app_core::git::status(&PathBuf::from(&project.working_dir))
}

#[tauri::command]
pub async fn projects_settings_get(
    state: State<'_, AppState>,
    name: String,
) -> Result<Settings, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let project = app_core::projects::get(&claude_dir, &name)?;
    app_core::settings::project_get(&PathBuf::from(&project.working_dir))
}

#[tauri::command]
pub async fn projects_settings_put(
    state: State<'_, AppState>,
    name: String,
    settings: Settings,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let project = app_core::projects::get(&claude_dir, &name)?;
    app_core::settings::project_put(&PathBuf::from(&project.working_dir), &settings)
}

#[tauri::command]
pub async fn projects_claude_md_get(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::claude_md_get(&claude_dir, &name)
}

#[tauri::command]
pub async fn projects_claude_md_put(
    state: State<'_, AppState>,
    name: String,
    content: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::projects::claude_md_put(&claude_dir, &name, &content)
}
