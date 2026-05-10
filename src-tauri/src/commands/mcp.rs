use std::path::PathBuf;

use app_core::types::{McpImportPayload, McpScope, McpServer, McpServerInput};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn mcp_list(
    state: State<'_, AppState>,
    scope: McpScope,
    working_dir: Option<String>,
) -> Result<Vec<McpServer>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::mcp::list(&claude_dir, scope, wd.as_deref())
}

#[tauri::command]
pub async fn mcp_get(
    state: State<'_, AppState>,
    name: String,
    scope: McpScope,
    working_dir: Option<String>,
) -> Result<McpServer, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::mcp::get(&claude_dir, &name, scope, wd.as_deref())
}

#[tauri::command]
pub async fn mcp_create(
    state: State<'_, AppState>,
    input: McpServerInput,
    scope: McpScope,
    working_dir: Option<String>,
) -> Result<McpServer, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::mcp::create(&claude_dir, input, scope, wd.as_deref())
}

#[tauri::command]
pub async fn mcp_delete(
    state: State<'_, AppState>,
    name: String,
    scope: McpScope,
    working_dir: Option<String>,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let wd = working_dir.map(PathBuf::from);
    app_core::mcp::delete(&claude_dir, &name, scope, wd.as_deref())
}

#[tauri::command]
pub async fn mcp_import(
    state: State<'_, AppState>,
    payload: McpImportPayload,
) -> Result<Vec<McpServer>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::mcp::import(&claude_dir, payload)
}
