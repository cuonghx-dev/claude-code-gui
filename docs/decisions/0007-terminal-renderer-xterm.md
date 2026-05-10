# ADR 0007 — Terminal renderer: xterm.js v5

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §6 mandates `xterm.js`. Alternatives like `@battlefieldduck/xterm-svelte`, `react-xtermjs`, or hand-rolled DOM are not viable given the Vue + Tauri target.

## Decision

Pin:

- `@xterm/xterm@5`
- `@xterm/addon-fit@0.x` (matches v5)
- `@xterm/addon-web-links@0.x`

Resize is debounced at 100 ms via `useDebounceFn` from `@vueuse/core` (per SPEC §6 revision).

Disposal: `term.dispose()` must run in `onBeforeUnmount` to avoid Vue HMR–leaked terminals during dev.

## Consequences

- Locked to xterm.js v5 lifecycle. v6 (when released) requires a coordinated upgrade.
