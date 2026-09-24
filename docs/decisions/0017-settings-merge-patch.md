# ADR 0017 — Settings writes are per-scope merge patches

**Status**: Accepted
**Date**: 2026-09-09 (recorded 2026-09-24)

## Context

SPEC §6 listed `settings_put(settings: Settings)` and
`projects_settings_put(...)`, which write a typed `Settings` struct back to
disk. A real `~/.claude/settings.json` holds far more keys than the struct
models; anything the struct does not know about lived in a
`#[serde(flatten)] extra` map that ts-rs skips, so every settings page had
to cast around it and one missed key would be silently dropped. The CLI
also writes these files while the GUI is open, with no lock protocol.

## Decision

Drop `settings_put` / `projects_settings_put`. Writes go through
`settings_patch(scope, patch, expected_mtime_ms)` in
`crates/core/src/settings_scope.rs`:

- **RFC 7386 merge patch** over the file's parsed JSON. A form sends only
  the keys it owns (`{"permissions":{"allow":[…]}}`, `{"statusLine":null}`
  to delete). The write path never deserializes into a typed struct, so
  unknown keys survive by construction. Arrays replace wholesale, so list
  editors (hooks, permissions) patch the whole array.
- **Scopes**: managed, user, project, local. `effective()` returns a flat
  list of `{key, value, source, overridden}` and unions list keys such as
  `permissions.allow` across scopes, as the CLI does.
- **Optimistic concurrency**: reads return `mtimeMs`; a write with a stale
  `expected_mtime_ms` is rejected and the UI offers a reload. No advisory
  lock — the CLI would not honor it.
- **Backup** to `<name>.ccg.bak` before each write (the CLI owns `.bak`).

`Settings` stays as a read-only projection for the few typed fields the UI
displays.

## Consequences

- Editing one key changes only that key; key order is preserved
  (`serde_json` `preserve_order`).
- Settings files are strict JSON, so structured writes cannot lose comments.
- Every settings form must express its change as a patch, never as a whole
  document.
