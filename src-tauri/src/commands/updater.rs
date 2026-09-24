//! Updater commands. The JS `check()` cannot change endpoints, so the
//! channel ("stable" / "beta") is applied here by substituting `{{channel}}`
//! in the configured endpoints before the plugin sees them.

use app_core::{AppError, ErrorCode};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};
use ts_rs::TS;

use crate::state::AppState;

/// Update found by `updater_check`, kept until `updater_install` consumes it.
#[derive(Default)]
pub struct PendingUpdate(pub tokio::sync::Mutex<Option<Update>>);

#[derive(Serialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
    /// RFC 3339.
    pub date: Option<String>,
}

fn updater_err(e: impl std::fmt::Display) -> AppError {
    AppError::new(ErrorCode::Internal, format!("updater: {e}"))
}

/// Endpoint templates from `plugins.updater.endpoints`, with the channel
/// filled in. Tauri substitutes `{{target}}` etc. itself afterwards.
fn channel_endpoints(raw: &[String], channel: &str) -> Result<Vec<tauri::Url>, AppError> {
    raw.iter()
        .map(|e| {
            let url = e.replace("{{channel}}", channel);
            tauri::Url::parse(&url).map_err(|err| updater_err(format!("bad endpoint '{url}': {err}")))
        })
        .collect()
}

fn channel_of(config_channel: Option<&str>) -> &'static str {
    match config_channel {
        Some("beta") => "beta",
        _ => "stable",
    }
}

#[tauri::command]
pub async fn updater_check(
    app: AppHandle,
    state: State<'_, AppState>,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateInfo>, AppError> {
    let channel = channel_of(state.config.read().await.updater_channel.as_deref());
    let raw: Vec<String> = app
        .config()
        .plugins
        .0
        .get("updater")
        .and_then(|u| u.get("endpoints"))
        .and_then(|e| serde_json::from_value(e.clone()).ok())
        .unwrap_or_default();
    let endpoints = channel_endpoints(&raw, channel)?;

    let update = app
        .updater_builder()
        .endpoints(endpoints)
        .map_err(updater_err)?
        .build()
        .map_err(updater_err)?
        .check()
        .await
        .map_err(updater_err)?;

    let info = update.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
        notes: u.body.clone(),
        date: u
            .date
            .and_then(|d| chrono::DateTime::from_timestamp(d.unix_timestamp(), 0))
            .map(|d| d.to_rfc3339()),
    });
    *pending.0.lock().await = update;
    Ok(info)
}

/// Download, verify and install the update found by the last check. The
/// caller relaunches the app afterwards.
#[tauri::command]
pub async fn updater_install(app: AppHandle) -> Result<(), AppError> {
    let pending = app.state::<PendingUpdate>();
    let update = pending
        .0
        .lock()
        .await
        .take()
        .ok_or_else(|| AppError::invalid("no update pending; check for updates first"))?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(updater_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_channel_and_keeps_tauri_placeholders() {
        let raw = vec!["https://u.example.com/{{channel}}/{{target}}/{{current_version}}".to_string()];
        let urls = channel_endpoints(&raw, channel_of(Some("beta"))).unwrap();
        let s = urls[0].as_str();
        assert!(s.starts_with("https://u.example.com/beta/"), "{s}");
        assert!(s.contains("target"), "tauri placeholders survive: {s}");
        assert_eq!(channel_of(None), "stable");
        assert_eq!(channel_of(Some("nightly")), "stable");
    }
}
