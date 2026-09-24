use std::path::PathBuf;

use app_core::types::DirEntry;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn directories_list(parent: String) -> Result<Vec<DirEntry>, AppError> {
    let p = app_core::files::expand_tilde(&parent);
    app_core::files::directories_list(&p)
}

#[tauri::command]
pub async fn files_read(path: String) -> Result<String, AppError> {
    let p = app_core::files::expand_tilde(&path);
    app_core::files::files_read(&p)
}

/// Resolve `$HOME` for the frontend's directory picker default.
#[tauri::command]
pub async fn fs_home_dir() -> Result<String, AppError> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| AppError::internal("could not resolve HOME"))?;
    Ok(home.to_string_lossy().into_owned())
}

/// Subscribe the global watcher to a project working directory. Idempotent —
/// re-watching a path returns the existing subscription id.
#[tauri::command]
pub async fn watch_project_dir(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    let id = state.watcher.watch_project(&PathBuf::from(path))?;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn unwatch_path(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|e| AppError::invalid(format!("invalid subscription id: {e}")))?;
    state.watcher.unwatch(uuid)
}

/// Show `path` selected in Finder / Explorer / the Linux file manager.
#[tauri::command]
pub async fn reveal_in_finder(path: String) -> Result<(), AppError> {
    let p = PathBuf::from(app_core::files::expand_tilde(&path));
    if !p.exists() {
        return Err(AppError::not_found(format!("'{}' does not exist", p.display())));
    }
    tauri_plugin_opener::reveal_item_in_dir(&p)
        .map_err(|e| AppError::internal(format!("reveal failed: {e}")))
}
