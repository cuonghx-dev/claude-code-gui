use app_core::types::{ChordValidation, Keybinding, KeybindingsDoc};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn keybindings_get(state: State<'_, AppState>) -> Result<KeybindingsDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::keybindings::get(&claude_dir)
}

#[tauri::command]
pub async fn keybindings_put(
    state: State<'_, AppState>,
    bindings: Vec<Keybinding>,
    expected_mtime_ms: Option<i64>,
) -> Result<KeybindingsDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::keybindings::put(&claude_dir, &bindings, expected_mtime_ms)
}

#[tauri::command]
pub async fn keybindings_create(state: State<'_, AppState>) -> Result<KeybindingsDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::keybindings::create_default(&claude_dir)
}

#[tauri::command]
pub async fn keybindings_raw_get(state: State<'_, AppState>) -> Result<String, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::keybindings::raw_get(&claude_dir)
}

#[tauri::command]
pub async fn keybindings_raw_put(
    state: State<'_, AppState>,
    content: String,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::keybindings::raw_put(&claude_dir, &content, expected_mtime_ms)
}

#[tauri::command]
pub async fn keybindings_validate(chord: String) -> Result<ChordValidation, AppError> {
    Ok(app_core::keybindings::validate_chord(&chord))
}
