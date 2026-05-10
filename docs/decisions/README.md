# Architecture Decision Records

ADRs documenting locked design choices. Numbered chronologically; status starts as **Accepted** when committed and may change to **Superseded** if replaced.

| # | Title | Status |
|---|-------|--------|
| [0001](0001-ui-radix-vue.md) | UI library: radix-vue + Tailwind | Accepted |
| [0002](0002-server-state-tanstack-query.md) | Server state: TanStack Query (Vue Query) | Accepted |
| [0003](0003-pinia-ui-state.md) | Pinia for UI state only | Accepted |
| [0004](0004-router-unplugin-vue-router.md) | Router: unplugin-vue-router (file-based) | Accepted |
| [0005](0005-package-manager-bun.md) | Package manager: bun | Accepted |
| [0006](0006-ts-rs-export.md) | ts-rs export: committed types + drift gate | Accepted |
| [0007](0007-terminal-renderer-xterm.md) | Terminal renderer: xterm.js v5 | Accepted |
| [0008](0008-rust-toolchain.md) | Rust toolchain: pinned 1.82 | Accepted |
| [0009](0009-logging-tracing.md) | Logging: tracing + tracing-appender | Accepted |
| [0010](0010-tests.md) | Test framework | Accepted |
| [0011](0011-code-signing-procurement.md) | Code-signing prerequisites | Accepted |
| [0012](0012-hooks-metrics-deferred.md) | Hooks-file metrics path: deferred post-1.0 | Accepted |
| [0013](0013-markdown-editor-codemirror.md) | Markdown editor: CodeMirror 6 | Accepted |
| [0014](0014-form-validation.md) | Form validation: vee-validate + zod | Accepted |
| [0015](0015-toast-vue-sonner.md) | Toast layer: vue-sonner | Accepted |
| [0016](0016-license-mit.md) | License: MIT | Accepted |

## Adding an ADR

1. Pick the next number.
2. Create `NNNN-kebab-case-title.md`.
3. Use sections: **Status**, **Date**, **Context**, **Decision**, **Consequences** (and **Rejected alternatives** if relevant).
4. Add a row to the table above.
5. Mention the ADR number in the PR description.
