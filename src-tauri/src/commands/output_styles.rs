use std::path::PathBuf;

use app_core::types::{OutputStyle, OutputStyleInput, OutputStyleScope};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn output_styles_list(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<OutputStyle>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::output_styles::list(&claude_dir, wd.as_deref())
}

#[tauri::command]
pub async fn output_styles_get(
    state: State<'_, AppState>,
    id: String,
    scope: OutputStyleScope,
    working_dir: Option<String>,
) -> Result<OutputStyle, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::output_styles::get(&claude_dir, &id, scope, wd.as_deref())
}

#[tauri::command]
pub async fn output_styles_create(
    state: State<'_, AppState>,
    input: OutputStyleInput,
) -> Result<OutputStyle, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::output_styles::create(&claude_dir, input)
}

#[tauri::command]
pub async fn output_styles_delete(
    state: State<'_, AppState>,
    id: String,
    scope: OutputStyleScope,
    working_dir: Option<String>,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::output_styles::delete(&claude_dir, &id, scope, wd.as_deref())
}
