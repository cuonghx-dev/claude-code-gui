# Contributing

## Workflow

1. `bd ready` — find an unblocked task.
2. `bd show <id>` — get full context.
3. `bd update <id> --claim` — claim it.
4. Implement.
5. `bd close <id> --reason "..."` — close on completion.
6. `bd dolt push` — sync to remote.

See [`docs/SPEC.md`](docs/SPEC.md) for architecture and [`docs/decisions/`](docs/decisions/) for locked design choices.

## Build

```bash
bun install --cwd frontend
cd src-tauri && cargo tauri dev
```

## ts-rs binding flow

Rust IPC types are exported to TypeScript via [`ts-rs`](https://crates.io/crates/ts-rs). Generated `.ts` files in `frontend/src/types/ipc/` are **committed** so frontend builds don't depend on running `cargo`.

When you add or change a Rust IPC type:

1. Derive `ts_rs::TS` plus `#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]` on the type.
2. Run `cargo test --workspace` — the auto-generated `export_bindings_*` tests materialize updated `.ts` files.
3. Run `bun --cwd frontend run build:ipc-barrel` to refresh the index.
4. Stage the generated diff: `git add frontend/src/types/ipc/`.
5. Commit alongside the Rust change.

CI runs:

```bash
cargo test --workspace
bun --cwd frontend run build:ipc-barrel
git diff --exit-code -- frontend/src/types/ipc/
```

If the diff is non-empty, CI fails. Re-run the two commands and commit.

### Conventions

- **camelCase TypeScript shape**: every IPC struct uses `#[serde(rename_all = "camelCase")]`.
- **Discriminant enums** use `#[serde(rename_all = "kebab-case")]`.
- **`serde_json::Value` fields** annotate with `#[ts(type = "unknown")]`.
- **`chrono::DateTime` and `uuid::Uuid`** map cleanly via the `chrono-impl` and `uuid-impl` ts-rs features (already enabled in workspace deps).

### Optional pre-commit hook

Save the following as `.git/hooks/pre-commit` and `chmod +x`:

```bash
#!/usr/bin/env bash
set -e
cargo test --workspace
bun --cwd frontend run build:ipc-barrel
git diff --exit-code -- frontend/src/types/ipc/
```

Don't enforce — CI is the gate. The hook just shortens the feedback loop.

## Style

- Rust: `cargo fmt` + `cargo clippy --workspace -- -D warnings`.
- Vue/TS: `bun --cwd frontend run lint`.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/). Do **not** include `Co-Authored-By:` lines (per project convention).
