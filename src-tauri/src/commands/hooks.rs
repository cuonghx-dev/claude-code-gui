use std::path::PathBuf;

use app_core::types::HookGroup;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn hooks_list(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<HookGroup>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::hooks::list(&claude_dir, wd.as_deref())
}
