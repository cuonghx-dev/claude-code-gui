# claude-code-gui

Visual manager for Claude Code agents, commands, skills, plans, plugins, MCP servers, and sessions.

Built on **Tauri 2** with a **Vue 3 + Vite** frontend and a pure-Rust backend. Reads and writes the existing `~/.claude/` directory layout — no migration required. Conversational use happens through an embedded terminal running the `claude` CLI.

See [`docs/SPEC.md`](docs/SPEC.md) for the full specification and [`docs/decisions/`](docs/decisions/) for ADRs covering locked architectural choices.

## Quick start

Prerequisites:
- Rust 1.82+ (pinned via `rust-toolchain.toml`)
- [Bun](https://bun.sh) for the frontend
- [Tauri CLI](https://tauri.app/start/prerequisites/) (`cargo install tauri-cli --version "^2"`)
- Platform build deps per https://tauri.app/start/prerequisites/

```bash
# Install frontend deps
bun install --cwd frontend

# Dev (auto-runs `bun --cwd frontend dev`)
cd src-tauri && cargo tauri dev

# Release build
cargo tauri build
```

## Repository layout

```
claude-code-gui/
├── docs/
│   ├── SPEC.md                # full specification
│   └── decisions/             # ADRs (D1–D16)
├── frontend/                  # Vue 3 + Vite SPA
├── src-tauri/                 # Tauri shell + Rust workspace
│   ├── crates/
│   │   ├── core/              # FS domain logic, no Tauri deps
│   │   ├── pty/               # portable-pty wrapper, session manager
│   │   ├── watcher/           # notify + ignore-aware filewatcher
│   │   ├── claude_cli/        # claude -p subprocess wrapper
│   │   └── app/               # Tauri binary, command bindings, AppState
│   ├── tauri.conf.json
│   └── capabilities/
└── .beads/                    # cross-session task tracker
```

## Issue tracking

Multi-session work is tracked in [beads](https://github.com/steveyegge/beads). Run `bd ready` to see unblocked tasks; `bd show <id>` for details.

## License

MIT — see [LICENSE](LICENSE).
