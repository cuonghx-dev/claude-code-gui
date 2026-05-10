# ADR 0016 — License: MIT

**Status**: Accepted
**Date**: 2026-05-10

## Decision

MIT License. Permissive, broadly compatible, the de facto default for tooling in this space.

Apache-2.0 was considered for its explicit patent grant. Rejected because (a) the codebase is unlikely to attract patent-relevant claims, and (b) MIT is shorter and more recognizable in a developer-tools context.

## Consequences

- `LICENSE` file at repo root contains the MIT text.
- README and `tauri.conf.json` declare MIT.
- Contributors implicitly agree to MIT licensing of their patches per GitHub's inbound = outbound model. No CLA.
