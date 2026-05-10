use app_core::types::{Skill, SkillImportSource, SkillInput};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn skills_list(state: State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::list(&claude_dir)
}

#[tauri::command]
pub async fn skills_get(state: State<'_, AppState>, slug: String) -> Result<Skill, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::get(&claude_dir, &slug)
}

#[tauri::command]
pub async fn skills_create(
    state: State<'_, AppState>,
    input: SkillInput,
) -> Result<Skill, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::create(&claude_dir, input)
}

#[tauri::command]
pub async fn skills_update(
    state: State<'_, AppState>,
    slug: String,
    input: SkillInput,
) -> Result<Skill, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::update(&claude_dir, &slug, input)
}

#[tauri::command]
pub async fn skills_delete(state: State<'_, AppState>, slug: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::delete(&claude_dir, &slug)
}

#[tauri::command]
pub async fn skills_export(state: State<'_, AppState>, slug: String) -> Result<Vec<u8>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::export(&claude_dir, &slug)
}

#[tauri::command]
pub async fn skills_import(
    state: State<'_, AppState>,
    source: SkillImportSource,
) -> Result<Vec<Skill>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::import(&claude_dir, source)
}
