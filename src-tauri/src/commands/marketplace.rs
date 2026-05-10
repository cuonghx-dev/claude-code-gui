use std::sync::Arc;

use app_core::types::{AvailablePlugin, MarketplaceSource, MarketplaceSourceInput};
use app_core::{AppError, RequestId};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

#[tauri::command]
pub async fn marketplace_available(
    state: State<'_, AppState>,
) -> Result<Vec<AvailablePlugin>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::available(&claude_dir)
}

#[tauri::command]
pub async fn marketplace_sources_list(
    state: State<'_, AppState>,
) -> Result<Vec<MarketplaceSource>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::sources_list(&claude_dir)
}

#[tauri::command]
pub async fn marketplace_sources_add(
    state: State<'_, AppState>,
    input: MarketplaceSourceInput,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::sources_add(&claude_dir, input)
}

#[tauri::command]
pub async fn marketplace_sources_remove(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::sources_remove(&claude_dir, &name)
}

#[tauri::command]
pub async fn marketplace_sources_update(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::sources_update(&claude_dir, &name).await
}

#[tauri::command]
pub async fn marketplace_install(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<RequestId, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let request_id = RequestId::new();
    let event = format!("marketplace:install:{request_id}");

    // Resolve install URL: look up `name` in `source` from the cached
    // sources list. Fail loudly if the source hasn't been refreshed yet.
    let sources = app_core::marketplace::sources_list(&claude_dir)?;
    let install_url = sources
        .iter()
        .find(|s| s.name == source)
        .and_then(|s| s.plugins.iter().find(|p| p.id == name))
        .map(|p| p.install_url.clone())
        .ok_or_else(|| {
            AppError::not_found(format!(
                "plugin '{name}' not in source '{source}' (run sources_update first)"
            ))
        })?;

    let app_for_emit = app.clone();
    let event_for_cb = event.clone();
    let progress: app_core::marketplace::Progress = Arc::new(move |step, pct| {
        let _ = app_for_emit.emit(
            &event_for_cb,
            serde_json::json!({
                "kind": "progress",
                "step": step,
                "percent": pct,
            }),
        );
    });

    // Run on the blocking pool — git2 clone is sync.
    let claude_dir_for_task = claude_dir.clone();
    let id = name.clone();
    let progress_for_task = Arc::clone(&progress);
    tokio::task::spawn_blocking(move || {
        match app_core::marketplace::install(
            &claude_dir_for_task,
            &id,
            &install_url,
            progress_for_task,
        ) {
            Ok(()) => {
                let _ = app.emit(&event, serde_json::json!({ "kind": "done" }));
            }
            Err(e) => {
                let _ = app.emit(
                    &event,
                    serde_json::json!({ "kind": "error", "error": e.to_string() }),
                );
            }
        }
    });

    Ok(request_id)
}

#[tauri::command]
pub async fn marketplace_uninstall(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::marketplace::uninstall(&claude_dir, &id)
}
