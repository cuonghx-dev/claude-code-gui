# ADR 0005 — Package manager: bun

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §8 already invokes `bun --cwd ../frontend dev` from `tauri.conf.json`. We need one package manager — adding pnpm/npm/yarn fallback paths leads to lockfile churn.

## Decision

`bun` is the only supported frontend package manager. CI uses `oven-sh/setup-bun@v2`. Lockfile `bun.lockb` is committed.

## Consequences

- `bun install --frozen-lockfile` in CI.
- Contributors install bun (https://bun.sh).
- No `package-lock.json`, no `pnpm-lock.yaml`.
