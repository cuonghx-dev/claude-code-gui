use std::collections::HashMap;

use app_core::types::{Agent, AgentImport, AgentInput};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn agents_list(state: State<'_, AppState>) -> Result<Vec<Agent>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::list(&claude_dir)
}

#[tauri::command]
pub async fn agents_get(state: State<'_, AppState>, slug: String) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::get(&claude_dir, &slug)
}

#[tauri::command]
pub async fn agents_skill_counts(state: State<'_, AppState>) -> Result<HashMap<String, usize>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::skill_counts(&claude_dir)
}

#[tauri::command]
pub async fn agents_create(
    state: State<'_, AppState>,
    input: AgentInput,
) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::create(&claude_dir, input)
}

#[tauri::command]
pub async fn agents_update(
    state: State<'_, AppState>,
    slug: String,
    input: AgentInput,
) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::update(&claude_dir, &slug, input)
}

#[tauri::command]
pub async fn agents_delete(state: State<'_, AppState>, slug: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::delete(&claude_dir, &slug)
}

#[tauri::command]
pub async fn agents_export(state: State<'_, AppState>, slug: String) -> Result<String, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::export(&claude_dir, &slug)
}

#[tauri::command]
pub async fn agents_import(
    state: State<'_, AppState>,
    payload: AgentImport,
) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::import(&claude_dir, payload)
}
