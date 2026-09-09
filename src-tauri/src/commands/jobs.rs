use app_core::types::{Job, JobDetail};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn jobs_list(state: State<'_, AppState>) -> Result<Vec<Job>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::jobs::list(&claude_dir)
}

#[tauri::command]
pub async fn jobs_get(state: State<'_, AppState>, job_id: String) -> Result<JobDetail, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::jobs::get(&claude_dir, &job_id)
}
