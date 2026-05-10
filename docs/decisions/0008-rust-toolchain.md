# ADR 0008 — Rust toolchain: pinned 1.88

**Status**: Accepted
**Date**: 2026-05-10

## Context

Tauri 2's MSRV moves with releases. Floating to `stable` exposes us to silent breakage when a clippy lint changes.

## Decision

Pin via `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.88"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

CI matrix uses the same toolchain file via `dtolnay/rust-toolchain@stable` (auto-detects).

Bumps are deliberate PRs, never automatic.

## Consequences

- `cargo` and CI agree on toolchain.
- Bumping rust requires updating one file and verifying clippy passes.
