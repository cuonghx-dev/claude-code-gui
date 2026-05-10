use app_core::types::{Plan, PlanInput};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn plans_list(state: State<'_, AppState>) -> Result<Vec<Plan>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::list(&claude_dir)
}

#[tauri::command]
pub async fn plans_get(state: State<'_, AppState>, slug: String) -> Result<Plan, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::get(&claude_dir, &slug)
}

#[tauri::command]
pub async fn plans_create(
    state: State<'_, AppState>,
    input: PlanInput,
) -> Result<Plan, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::create(&claude_dir, input)
}

#[tauri::command]
pub async fn plans_update(
    state: State<'_, AppState>,
    slug: String,
    input: PlanInput,
) -> Result<Plan, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::update(&claude_dir, &slug, input)
}

#[tauri::command]
pub async fn plans_delete(state: State<'_, AppState>, slug: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::delete(&claude_dir, &slug)
}
