# ADR 0009 — Logging: tracing + tracing-appender + tracing-subscriber

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §8 already commits to `tracing`. Format and rotation policy need locking.

## Decision

- `tracing` for instrumentation
- `tracing-appender` for daily-rotated file logs (14-day retention)
- `tracing-subscriber` with two configurations:
  - **dev**: `tracing_subscriber::fmt().pretty()` — colorized, human-readable
  - **release**: `tracing_subscriber::fmt().json()` — JSON Lines, machine-parseable

Filter via `RUST_LOG` env (default `info`).

Log file paths (per SPEC §8):
- macOS: `~/Library/Logs/com.anthropic.claude-code-gui/app.log`
- Windows: `%LOCALAPPDATA%\com.anthropic.claude-code-gui\logs\`
- Linux: `~/.local/share/com.anthropic.claude-code-gui/logs/`

A panic hook is installed before `tauri::Builder::default()` so panics inside `setup()` are logged before the process exits.

## Consequences

- Production logs are JSON for log aggregator compatibility.
- Dev logs are pretty for direct readability.
- Single env var (`RUST_LOG`) controls verbosity.
