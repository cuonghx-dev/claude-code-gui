use app_core::types::{Workflow, WorkflowInput};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn workflows_list(state: State<'_, AppState>) -> Result<Vec<Workflow>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::workflows::list(&claude_dir)
}

#[tauri::command]
pub async fn workflows_get(state: State<'_, AppState>, slug: String) -> Result<Workflow, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::workflows::get(&claude_dir, &slug)
}

#[tauri::command]
pub async fn workflows_create(
    state: State<'_, AppState>,
    input: WorkflowInput,
) -> Result<Workflow, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::workflows::create(&claude_dir, input)
}

#[tauri::command]
pub async fn workflows_update(
    state: State<'_, AppState>,
    slug: String,
    input: WorkflowInput,
) -> Result<Workflow, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::workflows::update(&claude_dir, &slug, input)
}

#[tauri::command]
pub async fn workflows_delete(state: State<'_, AppState>, slug: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::workflows::delete(&claude_dir, &slug)
}
