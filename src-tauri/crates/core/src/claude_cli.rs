use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AppError, ErrorCode};

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCliInfo {
    pub path: String,
    pub version: String,
}

/// Best-effort probe for the `claude` binary on PATH. Returns `None` (not an
/// error) if the binary is missing — the UI surfaces a setup banner.
pub fn probe() -> Option<ClaudeCliInfo> {
    let path = which()?;
    let version = version_of(&path)?;
    Some(ClaudeCliInfo {
        path: path.to_string_lossy().into_owned(),
        version,
    })
}

/// Resolve the claude binary path or fail loudly. Used by `compose_command`
/// — at PTY-spawn time, no claude binary means the user can't have a session.
pub fn path() -> Result<PathBuf, AppError> {
    if let Ok(env) = std::env::var("CLAUDE_CLI_PATH") {
        let p = PathBuf::from(env);
        if p.is_file() {
            return Ok(p);
        }
    }
    which().ok_or_else(|| {
        AppError::new(ErrorCode::NotFound, "`claude` binary not found on PATH")
    })
}

fn which() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "claude.exe" } else { "claude" };
    std::env::var_os("PATH")?
        .to_string_lossy()
        .split(if cfg!(windows) { ';' } else { ':' })
        .map(|d| std::path::Path::new(d).join(exe))
        .find(|p| p.is_file())
}

fn version_of(path: &std::path::Path) -> Option<String> {
    let out = Command::new(path).arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
