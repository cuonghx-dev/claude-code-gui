use app_core::types::{TerminalOpts, TerminalSession};
use app_core::{AppError, ErrorCode, SessionId};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

#[tauri::command]
pub async fn terminal_session_create(
    state: State<'_, AppState>,
    opts: TerminalOpts,
) -> Result<SessionId, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let id = state.pty.create(&claude_dir, opts).await?;
    Ok(SessionId(id))
}

#[tauri::command]
pub async fn terminal_session_input(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    let id = parse_id(&session_id)?;
    state.pty.input(id, data.as_bytes()).await
}

#[tauri::command]
pub async fn terminal_session_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    let id = parse_id(&session_id)?;
    state.pty.resize(id, cols, rows).await
}

#[tauri::command]
pub async fn terminal_session_kill(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    let id = parse_id(&session_id)?;
    state.pty.kill(id).await
}

#[tauri::command]
pub async fn terminal_sessions_list(
    state: State<'_, AppState>,
) -> Result<Vec<TerminalSession>, AppError> {
    Ok(state.pty.list().await)
}

#[tauri::command]
pub async fn terminal_session_get(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<TerminalSession, AppError> {
    let id = parse_id(&session_id)?;
    state.pty.get(id).await
}

/// `commands_execute(slug, args, working_dir)` — looks up the slash
/// command body, builds a `TerminalOpts` with `command_template` and
/// returns the new session id. The frontend attaches an existing
/// `ChatTerminal` to it.
#[tauri::command]
pub async fn commands_execute(
    state: State<'_, AppState>,
    slug: String,
    args: Option<String>,
    working_dir: Option<String>,
) -> Result<SessionId, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let cmd = app_core::commands::get(&claude_dir, &slug)?;
    let template = match args {
        Some(a) => cmd.body.replace("{{args}}", &a),
        None => cmd.body.clone(),
    };
    let opts = TerminalOpts {
        agent_slug: cmd.frontmatter.agent.clone(),
        working_dir,
        cols: 100,
        rows: 32,
        model: None,
        permission_mode: None,
        output_style_id: None,
        resume_session_id: None,
        command_template: Some(template),
    };
    let id = state.pty.create(&claude_dir, opts).await?;
    Ok(SessionId(id))
}

fn parse_id(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s).map_err(|e| {
        AppError::new(ErrorCode::InvalidInput, format!("invalid session id: {e}"))
    })
}
