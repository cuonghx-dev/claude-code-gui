# ADR 0011 — Code-signing prerequisites: start procurement Phase 0

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §8 requires Apple Developer ID + notarization, Windows Authenticode, optional Linux GPG, and an updater Ed25519 keypair embedded at build time. Lead times for procurement run in human-time, not CI-time.

## Decision

Acquire **during Phase 0** (not Phase 6):

| Item | Lead time |
|---|---|
| Apple Developer Program enrollment | 1–3 days individual / 2–6 weeks org (D-U-N-S) |
| Authenticode EV cert + hardware token order | 5–15 days shipping |
| Updater Ed25519 keypair (`tauri signer generate`) | Instant — but **immutable** (rotating breaks all installed clients); generate once, back up safely |

Defer to Phase 6:

- Apple Developer ID Application certificate (instant via portal)
- Apple `notarytool` API key (instant via App Store Connect)
- Actual signing wiring in `release.yml`
- Linux GPG signing (optional, post-1.0)

## Critical risk

Updater keypair has **no rotation story**. Once a binary is shipped with a pubkey, that keypair lives forever or all installed clients break. Triple-verify Phase 0 keypair backup before any release build.

## Consequences

- `tauri.conf.json` ships with the updater pubkey from Phase 0 onward.
- Private key lives in CI secrets `TAURI_PRIVATE_KEY` + `TAURI_KEY_PASSWORD`.
- Any release built without the keypair is wasted effort — don't ship until Phase 0 procurement closes.
