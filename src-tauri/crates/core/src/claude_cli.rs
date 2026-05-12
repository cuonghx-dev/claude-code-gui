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

/// Augment the process `PATH` with the user's login-shell `PATH`. macOS GUI
/// apps inherit only `/usr/bin:/bin:/usr/sbin:/sbin` from launchd, which
/// misses Homebrew, npm-global, nvm, pnpm, etc. Run this once at startup
/// before any binary lookups happen.
///
/// Best-effort: on Windows or if `$SHELL` isn't usable, this is a no-op.
pub fn inherit_login_path() {
    if cfg!(windows) {
        return;
    }
    let Ok(shell) = std::env::var("SHELL") else {
        return;
    };
    // `-l` triggers login profile, `-i` triggers interactive rc, `-c` runs
    // the command. zsh and bash both honor this combination. We use the
    // printf "%s" form (not echo) to avoid shells that strip trailing
    // newlines or print "PATH" on completion banners.
    let Ok(out) = Command::new(&shell)
        .arg("-lic")
        .arg("printf '%s' \"$PATH\"")
        .output()
    else {
        return;
    };
    if !out.status.success() {
        return;
    }
    let new_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if new_path.is_empty() {
        return;
    }
    let current = std::env::var("PATH").unwrap_or_default();
    let existing: std::collections::HashSet<&str> = current.split(':').filter(|s| !s.is_empty()).collect();
    let extra: Vec<&str> = new_path
        .split(':')
        .filter(|s| !s.is_empty() && !existing.contains(s))
        .collect();
    if extra.is_empty() {
        return;
    }
    let merged = if current.is_empty() {
        extra.join(":")
    } else {
        format!("{}:{current}", extra.join(":"))
    };
    std::env::set_var("PATH", merged);
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
