# ADR 0002 — Server state: TanStack Query (Vue Query)

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §2 originally hedged ("State via Pinia or composables alone — choose one"). For an IPC-driven app where every fetch is a `invoke()`, hand-rolled refetch maps and loading-state composables become significant boilerplate. TanStack Query offers caching, dedupe, automatic refetch, and a `queryClient` we can call `invalidateQueries` against from a single global `fs:change` listener.

## Decision

`@tanstack/vue-query` for **all** server state (any IPC `invoke` that fetches or mutates). Provider registered via `VueQueryPlugin` in `frontend/src/main.ts`. Singleton `QueryClient` exported from `frontend/src/lib/queryClient.ts`.

Defaults:
- `staleTime: 30_000`
- `gcTime: 5 * 60_000`
- `retry: 1` (local IPC; retries only help against true transient errors)
- `refetchOnWindowFocus: false` (Tauri windows don't blur like browsers; rely on `fs:change` instead)

Composables (`useAgents`, `useCommands`, …) are thin `useQuery` / `useMutation` wrappers keyed via `qk.*` from `frontend/src/lib/queryKeys.ts`.

## Consequences

- Single `fs:change` listener calls `queryClient.invalidateQueries` — no per-component refetch wiring.
- Sidebar counts and list pages share the same query cache; one fetch, two consumers.
- Adds ~14 KB gzipped to the bundle.
- Mandates the Pinia separation in ADR 0003 (Pinia for UI state only).
