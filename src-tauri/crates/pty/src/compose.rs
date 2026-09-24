//! Resolve a `TerminalOpts` into a `portable_pty::CommandBuilder`.
//!
//! Invariant (locked from SPEC §6 revisions and ADR 0011): the terminal
//! subsystem is a strict Claude wrapper. `compose` MUST return
//! `InvalidInput` if none of `agent_slug` / `resume_session_id` /
//! `command_template` is set. The `$SHELL` fallback was removed
//! deliberately — the OS-level entitlements granted to this app
//! (fs scope on `~/.claude/**`, deep-link handler) are sized for
//! Claude only; opening a generic interactive shell would break the
//! threat model in SPEC §8.

use std::path::Path;

use portable_pty::CommandBuilder;

use app_core::types::TerminalOpts;
use app_core::{AppError, ErrorCode};

/// `session_id` is the PTY id. A fresh launch passes it to claude as
/// `--session-id` so the JSONL transcript under `~/.claude/projects/` and
/// the `cli-history/` snapshot share one id; a resume keeps the CLI's id.
pub fn compose(
    claude_dir: &Path,
    opts: &TerminalOpts,
    session_id: &str,
) -> Result<CommandBuilder, AppError> {
    let argv = args(claude_dir, opts, session_id)?;
    let claude_path = app_core::claude_cli::path()?;
    let mut cmd = CommandBuilder::new(&claude_path);
    cmd.args(&argv);

    if let Some(wd) = &opts.working_dir {
        cmd.cwd(wd);
    }

    // Inherit a minimal env. portable-pty inherits the parent env by
    // default, which we want — the user's $PATH must be available so
    // claude can find git, etc.
    cmd.env("CLAUDE_CODE_GUI", "1");
    Ok(cmd)
}

/// The `claude` argv (without the binary) for `opts`.
fn args(claude_dir: &Path, opts: &TerminalOpts, session_id: &str) -> Result<Vec<String>, AppError> {
    let mut argv: Vec<String> = Vec::new();
    let mut launched = false;

    if let Some(slug) = &opts.agent_slug {
        let agent = app_core::agents::get(claude_dir, slug)?;
        argv.push("--append-system-prompt".into());
        argv.push(agent.body);
        let model = opts.model.clone().or_else(|| {
            agent
                .frontmatter
                .model
                .map(|m| match m {
                    app_core::types::AgentModel::Opus => "opus".to_string(),
                    app_core::types::AgentModel::Sonnet => "sonnet".to_string(),
                    app_core::types::AgentModel::Haiku => "haiku".to_string(),
                })
        });
        if let Some(m) = model {
            argv.push("--model".into());
            argv.push(m);
        }
        launched = true;
    }

    if let Some(mode) = &opts.permission_mode {
        argv.push("--permission-mode".into());
        argv.push(mode.as_cli_flag().into());
    }

    // The CLI has no `--output-style` flag; the style is a setting, so pass
    // it as an inline settings layer for this session only.
    if let Some(style) = &opts.output_style_id {
        argv.push("--settings".into());
        argv.push(serde_json::json!({ "outputStyle": style }).to_string());
    }

    match &opts.resume_session_id {
        Some(resume) => {
            argv.push("--resume".into());
            argv.push(resume.clone());
            launched = true;
        }
        None => {
            argv.push("--session-id".into());
            argv.push(session_id.to_string());
        }
    }

    // The initial prompt is the CLI's positional argument. It goes last,
    // after `--`, so a template starting with `-` is never read as a flag.
    if let Some(template) = &opts.command_template {
        argv.push("--".into());
        argv.push(template.clone());
        launched = true;
    }

    if !launched {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "terminal requires an agent slug, resume session id, or command template",
        ));
    }
    Ok(argv)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> TerminalOpts {
        serde_json::from_value(serde_json::json!({ "cols": 80, "rows": 24 })).unwrap()
    }

    #[test]
    fn template_is_trailing_positional() {
        let mut o = opts();
        o.command_template = Some("/review the diff".into());
        o.output_style_id = Some("Explanatory".into());
        let argv = args(Path::new("/nonexistent"), &o, "pty-1").unwrap();
        assert!(!argv.iter().any(|a| a == "--prompt" || a == "--output-style"));
        assert_eq!(argv[argv.len() - 2..], ["--", "/review the diff"]);
        let i = argv.iter().position(|a| a == "--settings").unwrap();
        assert_eq!(argv[i + 1], r#"{"outputStyle":"Explanatory"}"#);
    }

    #[test]
    fn requires_a_launch_target() {
        let err = args(Path::new("/nonexistent"), &opts(), "pty-1").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidInput);
    }

    #[test]
    fn fresh_launch_pins_session_id_to_pty_id() {
        let mut o = opts();
        o.command_template = Some("hi".into());
        let argv = args(Path::new("/nonexistent"), &o, "pty-1").unwrap();
        let i = argv.iter().position(|a| a == "--session-id").unwrap();
        assert_eq!(argv[i + 1], "pty-1");
        assert!(!argv.iter().any(|a| a == "--resume"));
    }

    #[test]
    fn resume_keeps_cli_session_id() {
        let mut o = opts();
        o.resume_session_id = Some("old".into());
        let argv = args(Path::new("/nonexistent"), &o, "pty-1").unwrap();
        assert!(!argv.iter().any(|a| a == "--session-id"));
        let i = argv.iter().position(|a| a == "--resume").unwrap();
        assert_eq!(argv[i + 1], "old");
    }
}
