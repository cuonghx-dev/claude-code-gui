# ADR 0012 — Hooks-file metrics path: deferred post-1.0

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §7 commits to regex-based parsing of PTY output for token usage and tool call metrics, with `strip-ansi-escapes::strip` to handle ANSI control sequences. An alternative path: inject a temporary `settings.json` via `claude --settings <path>` containing a `PostToolUse` / `Stop` hook that appends structured JSON to a temp JSONL file, which the watcher then tails. This bypasses regex entirely.

## Decision

**Defer the hooks-file path post-1.0.** Phase 4 ships the regex monitor as specified.

Reasoning:
- Regex path with ANSI stripping is correct for current Claude CLI output (already covered by §7 revision).
- Hooks-file path requires composing user's existing hooks with our injected hooks (naïve `--settings` replaces, doesn't merge), introduces temp-file lifecycle management, and may not capture every signal (mid-stream token summaries print to stdout independently of tool events).
- Resumed sessions started outside the GUI won't have our injected hooks — regex stays useful for those flows regardless.

## Feature flag

Add `AppConfig.experimental_hooks_metrics: bool = false` in Phase 0. When/if we ship the hooks-file path, it gates behind this flag for opt-in dogfooding before becoming the default.

## Consequences

- Phase 4 scope stays tight (~2 weeks).
- Regex path becomes the long-term primary; hooks-file becomes an optional upgrade if regex breaks against a future CLI release.
