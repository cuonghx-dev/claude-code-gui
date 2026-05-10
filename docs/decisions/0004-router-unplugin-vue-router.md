# ADR 0004 — Router: `unplugin-vue-router` (file-based)

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §5 lays out routes assuming file-based routing (`pages/agents/[slug].vue`, deeply nested under `sessions/project/[projectName]/session/[sessionId].vue`). Hand-rolled `vue-router` config would require keeping a config table in sync with the filesystem.

## Decision

Use [`unplugin-vue-router`](https://github.com/posva/unplugin-vue-router) for file-based routing with typed routes.

Pros:
- Routes match SPEC §5 paths verbatim with zero config table
- Renaming a file renames the route — refactor-safe
- Typed `useRoute()` / `definePage` via `vue-router/auto`

Cons:
- Build-time codegen step (handled transparently by Vite plugin)

## Consequences

- All pages live under `frontend/src/pages/`; the file tree is the route tree.
- TS imports use `vue-router/auto` instead of `vue-router` in pages that need typed route params.
