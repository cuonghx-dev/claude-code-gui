# ADR 0003 — Pinia for UI state only

**Status**: Accepted
**Date**: 2026-05-10

## Context

ADR 0002 commits TanStack Query for server state. Pinia still has a role for synchronous, transient UI state.

## Decision

Pinia stores hold UI-only state. Allowed:

- ✅ active modal id, sidebar collapsed bool, theme, command-palette open state
- ✅ slug of currently-edited entity (for unsaved-changes detection)
- ✅ draft text from `useDraftRecovery`

Forbidden:

- ❌ any data fetched from Rust, ever — belongs in TanStack Query

Stores: `useUiStore` (global chrome), `useTerminalUiStore` (active panel state — non-PTY data), `useDraftStore` (drafts keyed by `${domain}:${slug}`).

## Consequences

- Code review enforces the ban: server data in Pinia → request changes.
- Two state systems but with non-overlapping responsibilities, so the cognitive cost is low.
