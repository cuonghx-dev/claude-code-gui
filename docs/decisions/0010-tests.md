# ADR 0010 — Test framework

**Status**: Accepted
**Date**: 2026-05-10

## Decision

**Rust**:
- Built-in `#[test]`
- `insta` for snapshot tests (frontmatter round-tripping, JSONL session schemas)
- `tempfile` for `CLAUDE_DIR` fixtures
- `fixtures/.claude/` directory for golden tests of session JSONL parsing

**Frontend**:
- No Vitest unit tests — Vue components are mostly view layer
- Playwright for visual regression on a handful of golden screenshots, added in Phase 6

## Rejected

- **Vitest for Vue components**: per SPEC §9, deliberately skipped — most components are presentational. The cost of unit-test scaffolding outweighs the value.
- **Storybook**: Phase 6+ if we need component sandbox; not before.

## Consequences

- Rust gets full test coverage; frontend visual regression catches the last mile.
