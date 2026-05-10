use app_core::relationships::RelationshipsGraph;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn relationships_graph(
    state: State<'_, AppState>,
) -> Result<RelationshipsGraph, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::relationships::build(&claude_dir)
}
