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

pub fn compose(claude_dir: &Path, opts: &TerminalOpts) -> Result<CommandBuilder, AppError> {
    let claude_path = app_core::claude_cli::path()?;
    let mut cmd = CommandBuilder::new(&claude_path);

    let mut launched = false;

    if let Some(slug) = &opts.agent_slug {
        let agent = app_core::agents::get(claude_dir, slug)?;
        cmd.arg("--append-system-prompt");
        cmd.arg(agent.body);
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
            cmd.arg("--model");
            cmd.arg(m);
        }
        launched = true;
    }

    if let Some(template) = &opts.command_template {
        // The CLI consumes `--prompt` as the initial input. Phase 4 keeps
        // the command body opaque from the watcher's perspective.
        cmd.arg("--prompt");
        cmd.arg(template);
        launched = true;
    }

    if let Some(mode) = &opts.permission_mode {
        cmd.arg("--permission-mode");
        cmd.arg(mode.as_cli_flag());
    }

    if let Some(style) = &opts.output_style_id {
        cmd.arg("--output-style");
        cmd.arg(style);
    }

    if let Some(resume) = &opts.resume_session_id {
        cmd.arg("--resume");
        cmd.arg(resume);
        launched = true;
    }

    if let Some(wd) = &opts.working_dir {
        cmd.cwd(wd);
    }

    if !launched {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "terminal requires an agent slug, resume session id, or command template",
        ));
    }

    // Inherit a minimal env. portable-pty inherits the parent env by
    // default, which we want — the user's $PATH must be available so
    // claude can find git, etc.
    cmd.env("CLAUDE_CODE_GUI", "1");
    Ok(cmd)
}
