use std::path::PathBuf;

use app_core::types::DirEntry;
use app_core::AppError;

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
