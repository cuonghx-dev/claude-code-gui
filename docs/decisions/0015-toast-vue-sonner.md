# ADR 0015 — Toast layer: vue-sonner

**Status**: Accepted
**Date**: 2026-05-10

## Context

Phase 3 surfaces async results (mutation success/failure, `fs:flood` banners, marketplace install progress) via toasts. Options: `radix-vue Toast` (matches our base UI), `vue-sonner` (port of Sonner; simpler API).

## Decision

`vue-sonner`. Cleaner imperative API (`toast.success(...)`), built-in queueing, smaller bundle, no extra primitive integration.

Wrapper at `frontend/src/lib/toasts.ts` exposes a typed surface used everywhere — components don't import `vue-sonner` directly. Lets us swap the underlying toast lib later without touching call sites.

## Consequences

- Two component libs in play (radix-vue + vue-sonner). Acceptable given they don't overlap functionally.
