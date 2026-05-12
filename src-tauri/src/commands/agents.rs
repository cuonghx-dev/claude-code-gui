use std::collections::HashMap;
use std::sync::Arc;

use app_core::types::{Agent, AgentImport, AgentInput, ImproveRequest};
use app_core::{AppError, RequestId};
use tauri::{AppHandle, Emitter, State};

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
pub async fn agents_update_raw(
    state: State<'_, AppState>,
    slug: String,
    content: String,
) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::update_raw(&claude_dir, &slug, &content)
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

/// Async one-shot. Returns the request id immediately; deltas arrive on
/// `claude:improve:{request_id}`.
#[tauri::command]
pub async fn agents_improve_instructions(
    app: AppHandle,
    input: ImproveRequest,
) -> Result<RequestId, AppError> {
    let id = RequestId::new();
    let app_for_emit = app.clone();
    let emit: claude_cli::Emit = Arc::new(move |name, payload| {
        if let Err(e) = app_for_emit.emit(name, payload) {
            tracing::warn!(error = %e, event = %name, "improve emit failed");
        }
    });
    claude_cli::improve_instructions(emit, id, input)?;
    Ok(id)
}
