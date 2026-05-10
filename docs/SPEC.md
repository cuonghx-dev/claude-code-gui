# claude-code-gui — Specification

Greenfield Tauri 2 desktop application: visual manager for Claude Code agents, commands, skills, plans, plugins, MCP servers, and sessions. Reads/writes `~/.claude/`. Conversational use happens through an embedded terminal running the `claude` CLI.

**Stack**: Tauri 2 shell · Vite + Vue 3 + vue-router + Tailwind frontend · Rust workspace backend (`core` / `pty` / `watcher` / `claude_cli` / `app`) · `ts-rs` type sharing · `invoke()` + `listen()` IPC (no HTTP/WS).

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture](#2-architecture)
3. [Data Model](#3-data-model)
4. [IPC Contract](#4-ipc-contract)
5. [Pages & Navigation](#5-pages--navigation)
6. [Terminal Subsystem](#6-terminal-subsystem)
7. [File Watcher & Context Monitor](#7-file-watcher--context-monitor)
8. [Distribution & Runtime](#8-distribution--runtime)
9. [Extensibility](#9-extensibility)
10. [Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Overview

### What it is

`claude-code-gui` is a single-binary desktop app that wraps the `~/.claude/` directory in a visual editor and runtime. Built on Tauri 2 with a Vue 3 SPA frontend and a pure Rust backend.

It exposes:

- **Authoring UI** for agents, slash commands, skills, plans, output styles, MCP servers
- **Marketplace** for discovering and installing plugins from configured sources
- **Embedded terminal** that runs the `claude` CLI with optional agent system prompt preload
- **Session browser** for past Claude Code projects and conversations under `~/.claude/projects/`
- **Live monitoring** of token usage, cost, file changes, and tool calls during a CLI session

### Who it's for

- Developers using Claude Code who want a visual editor for a growing agent/skill collection
- Power users running multiple Claude Code sessions wanting a single pane to inspect them
- Anyone who'd rather click than hand-edit YAML frontmatter

### Goals

1. **Single binary distribution.** One signed installer per platform (`.dmg` / `.msi` / `.AppImage`), < 20 MB compressed.
2. **Native performance.** OS WebView (WKWebView / WebView2 / WebKitGTK). No Node runtime. No Chromium.
3. **Zero data migration.** Reads and writes the existing `~/.claude/` layout maintained by the Claude CLI.
4. **Terminal-only chat.** No SDK integration for conversational use. The CLI is the canonical entry point.
5. **Pure Rust backend.** Every server-side responsibility is a Tauri command, a Tokio task, or a spawned subprocess (`claude`, `git`).
6. **Type-safe IPC.** Rust-defined request/response shapes are exported as TypeScript via `ts-rs`. No drift.

### Non-goals

- Web deployment. Desktop only.
- Multi-user or server mode.
- Mobile.
- Hosting Claude or any LLM. Requires `claude` CLI on the host.
- Authentication. Single local user, single `~/.claude/` directory.

### Glossary

| Term | Meaning |
|------|---------|
| **Agent** | Markdown file under `~/.claude/agents/` with YAML frontmatter (name, model, color, etc.) and an instructions body. Acts as a system prompt template. |
| **Command** | Markdown file under `~/.claude/commands/` invoked as `/name` in chat. Frontmatter declares args and allowed tools. |
| **Skill** | Self-contained directory `~/.claude/skills/<name>/SKILL.md`. Loaded on-demand by the CLI when triggers match. |
| **Plugin** | Marketplace bundle installed under `~/.claude/plugins/`. |
| **MCP server** | External tool server speaking the Model Context Protocol. Registered in `.mcp.json`. |
| **Plan** | Plain markdown design doc under `~/.claude/plans/`. |
| **Output style** | Style sheet for assistant output — global or project-scoped markdown. |
| **Session** | A single Claude conversation. JSONL under `~/.claude/projects/<encoded>/<sessionId>.jsonl`. |
| **Project** | A working directory whose path Claude CLI has been opened in. Surfaces under `~/.claude/projects/` with `/` encoded as `-`. |

---

## 2. Architecture

### High-level diagram

```
┌──────────────────────────────────────────────────────────────┐
│                     Tauri 2 Application                      │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ WebView  (WKWebView / WebView2 / WebKitGTK)          │    │
│  │                                                      │    │
│  │  Vue 3 SPA  (Vite)                                   │    │
│  │  ├─ Routes:  /agents /commands /skills /plans …      │    │
│  │  ├─ Composables:  useAgents, useTerminal, …          │    │
│  │  ├─ Components:   PageHeader, ChatTerminal, …        │    │
│  │  └─ Tauri JS API:  invoke(), event::listen()         │    │
│  └────────────────────────┬─────────────────────────────┘    │
│                           │ IPC                              │
│  ┌────────────────────────▼─────────────────────────────┐    │
│  │ Rust core  (Tauri main process)                      │    │
│  │                                                      │    │
│  │  Commands ────────────────────────────────────┐      │    │
│  │  │ agents::* commands::* skills::* plans::*    │      │    │
│  │  │ output_styles::* mcp::* plugins::*          │      │    │
│  │  │ marketplace::* projects::* sessions::*      │      │    │
│  │  │ settings::* config::* terminal::*           │      │    │
│  │  │ claude_cli::improve_instructions            │      │    │
│  │  └─────────────────────────────────────────────┘      │    │
│  │                                                      │    │
│  │  Long-running Tokio tasks                            │    │
│  │  ├─ PTY readers (per session) → events               │    │
│  │  ├─ File watcher (notify) → events                   │    │
│  │  └─ claude -p subprocess (one-shot rewrites)         │    │
│  │                                                      │    │
│  │  Stateless utilities                                 │    │
│  │  ├─ frontmatter parser (serde_yaml)                  │    │
│  │  ├─ relationship extractor                           │    │
│  │  ├─ git ops (git2)                                   │    │
│  │  └─ marketplace fetcher (reqwest + git2)             │    │
│  └──────────────────────────────────────────────────────┘    │
│                           │                                  │
└───────────────────────────┼──────────────────────────────────┘
                            │
                            ▼
                ~/.claude/      (filesystem)
                ~/.claude/projects/
                claude, git    (PATH)
```

### Repository layout

```
claude-code-gui/
├── frontend/                       # Vue 3 + Vite SPA
│   ├── src/
│   │   ├── pages/                  # File-based routes (vue-router)
│   │   ├── components/
│   │   ├── composables/
│   │   ├── utils/
│   │   │   └── ipc.ts              # invoke()/listen() wrapper
│   │   ├── types/
│   │   │   ├── ipc/                # Generated by ts-rs (committed)
│   │   │   └── ui.ts               # UI-only types
│   │   ├── App.vue
│   │   └── main.ts
│   ├── index.html
│   ├── vite.config.ts
│   └── package.json
├── src-tauri/                      # Tauri shell + Rust backend
│   ├── Cargo.toml                  # workspace
│   ├── tauri.conf.json
│   ├── icons/
│   ├── build.rs
│   └── crates/
│       ├── core/                   # FS domain logic, no Tauri deps
│       │   ├── src/
│       │   │   ├── agents.rs
│       │   │   ├── commands.rs
│       │   │   ├── skills.rs
│       │   │   ├── plans.rs
│       │   │   ├── output_styles.rs
│       │   │   ├── plugins.rs
│       │   │   ├── marketplace.rs
│       │   │   ├── mcp.rs
│       │   │   ├── projects.rs
│       │   │   ├── sessions.rs
│       │   │   ├── settings.rs
│       │   │   ├── frontmatter.rs
│       │   │   ├── relationships.rs
│       │   │   ├── claude_dir.rs
│       │   │   ├── git.rs
│       │   │   ├── models.rs       # pricing / context window registry
│       │   │   ├── types/          # ts-rs derived shared types
│       │   │   └── lib.rs
│       │   └── tests/              # golden + snapshot tests
│       ├── pty/                    # portable-pty wrapper, session manager
│       ├── watcher/                # notify + debouncer
│       ├── claude_cli/             # claude -p subprocess wrapper
│       └── app/                    # Tauri binary: command bindings, AppState
│           ├── src/
│           │   ├── main.rs
│           │   ├── commands/       # one file per domain
│           │   ├── events.rs       # event name constants
│           │   ├── error.rs        # AppError + From impls
│           │   └── state.rs        # AppState
│           └── Cargo.toml
└── docs/
```

### Layer responsibilities

#### Frontend

- **Vue 3 + Vite + vue-router**. SPA, no SSR. State via Pinia (or composables alone — choose one and stick with it).
- **TypeScript everywhere.** Generated `types/ipc/` is the source of truth for backend payloads.
- **UI library**: pick one — `@nuxt/ui` is a Nuxt-only option; Tauri build defaults to `radix-vue` + Tailwind. (Decision deferred to first PR.)
- **Routing**: file-based router via `unplugin-vue-router` or hand-rolled `vue-router` config. File-based keeps page mapping obvious.
- **No HTTP, no WebSocket.** Only `invoke()` and `listen()`.

#### Rust backend

Cargo workspace with five crates:

| Crate | Depends on | Responsibility |
|-------|-----------|----------------|
| `core` | nothing Tauri-specific | All FS reads/writes, parsing, relationships, marketplace fetch, git ops. Fully testable with `cargo test`. |
| `pty` | `core`, `portable-pty` | PTY session lifecycle. Owns the `PtyManager`. |
| `watcher` | `core`, `notify` | Global file watcher + per-path subscriptions. |
| `claude_cli` | `core` | Spawns `claude -p --output-format stream-json` for non-conversational SDK use cases. |
| `app` | all of the above + `tauri` | Registers commands, owns `AppState`, plugin setup, deep links. |

`core` has zero Tauri dependencies. This is enforced by Cargo features and reviewed in PR.

### Frontend ↔ Backend protocol

Two channels:

1. **Commands.** Request/response. Frontend calls `invoke('agents_list')` and awaits a typed payload. Errors throw a typed `AppError`.
2. **Events.** Server → client push. Used for terminal output, file changes, long-running operation progress.

### State management

`AppState` (held by Tauri's `State<AppState>`):

```rust
pub struct AppState {
    pty: Arc<PtyManager>,                          // tokio::Mutex<HashMap<Uuid, PtyHandle>>
    watcher: Arc<WatcherHandle>,
    claude_dir: Arc<RwLock<PathBuf>>,
    claude_cli: Arc<RwLock<Option<ClaudeCliInfo>>>,
    config: Arc<RwLock<AppConfig>>,                // tauri-plugin-store backed
}
```

- Per-session PTY tasks run on the Tokio runtime. Output is pushed via `app.emit("pty:output:{id}", payload)`.
- The file watcher runs as a single global task. Subscribers are identified by `WatchSubscription` IDs; each component listens on `fs:change` and filters by path prefix.
- Long-running async commands return a `request_id` immediately and emit progress events keyed by that id.

### Dependencies

Top-level versions (pin major+minor; let minor revisions float):

```toml
# crates/core
serde         = { version = "1",   features = ["derive"] }
serde_json    = "1"
serde_yaml    = "0.9"
ts-rs         = { version = "9",   features = ["serde-compat", "uuid-impl", "chrono-impl"] }
thiserror     = "1"
anyhow        = "1"
tracing       = "0.1"
walkdir       = "2"
glob          = "0.3"
git2          = { version = "0.19", features = ["vendored-libgit2"] }
reqwest       = { version = "0.12", features = ["json", "rustls-tls", "stream"] }
sha2          = "0.10"
uuid          = { version = "1",   features = ["v4", "serde"] }
chrono        = { version = "0.4", features = ["serde"] }
tokio         = { version = "1",   features = ["fs", "process", "sync", "macros", "rt-multi-thread", "io-util"] }
rmcp          = "0.1"

# crates/pty
portable-pty  = "0.8"

# crates/watcher
notify                  = "6"
notify-debouncer-mini   = "0.4"

# crates/app
tauri                   = { version = "2", features = ["macos-private-api"] }
tauri-plugin-dialog     = "2"
tauri-plugin-shell      = "2"
tauri-plugin-fs         = "2"
tauri-plugin-os         = "2"
tauri-plugin-process    = "2"
tauri-plugin-store      = "2"
tauri-plugin-opener     = "2"
tauri-plugin-updater    = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-deep-link  = "2"
```

---

## 3. Data Model

The on-disk layout is owned by the Claude CLI. `claude-code-gui` reads and writes it without migration.

### On-disk layout

```
~/.claude/
├── agents/                       # markdown + YAML frontmatter (recursive)
├── commands/                     # markdown + YAML frontmatter (recursive)
├── skills/<slug>/SKILL.md        # one dir per skill
├── plans/                        # plain markdown
├── output-styles/                # markdown + frontmatter (global)
├── projects/                     # CLI session JSONL, one dir per encoded project path
│   └── -Users-foo-app/
│       ├── 01h2k3...jsonl
│       └── ...
├── plugins/                      # marketplace-installed bundles
├── cli-history/                  # PTY snapshot per session (claude-code-gui-owned)
├── settings.json                 # global settings
├── .mcp.json                     # global MCP server registry
├── .imports.json                 # GitHub skill import metadata
└── .marketplaces.json            # configured marketplace sources
```

Per-project `<projectPath>/.claude/`:
- `output-styles/` — project-scoped styles
- `.mcp.json` — project-scoped MCP servers
- `settings.json` — project-scoped settings
- `CLAUDE.md` — project conventions

### Type sharing

`ts-rs` exports Rust types to TypeScript:

```rust
#[derive(Serialize, Deserialize, ts_rs::TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub slug: String,
    pub filename: String,
    pub directory: String,
    pub frontmatter: AgentFrontmatter,
    pub body: String,
    pub has_memory: bool,
    pub file_path: String,
}
```

`cargo test --workspace` runs the export step. Generated `.ts` files are committed to `frontend/src/types/ipc/`. Frontend imports them verbatim.

### Type catalog

Defined once in Rust (`crates/core/src/types/`), exported to TS via `ts-rs`:

| Domain | Types |
|--------|-------|
| Agents | `Agent`, `AgentFrontmatter`, `AgentInput`, `AgentImport`, `AgentModel`, `AgentMemory`, `AgentTool` |
| Commands | `Command`, `CommandFrontmatter`, `CommandInput` |
| Skills | `Skill`, `SkillFrontmatter`, `SkillInput`, `SkillSource` |
| Plans | `Plan`, `PlanInput` |
| Output Styles | `OutputStyle`, `OutputStyleScope`, `OutputStyleInput` |
| Plugins | `Plugin`, `PluginDetail`, `AvailablePlugin`, `MarketplaceSource` |
| MCP | `McpServer`, `McpServerInput`, `McpCapabilities`, `McpTool`, `McpResource`, `McpPrompt` |
| Projects | `Project`, `ProjectInfo`, `FileNode`, `GitStatus`, `GitFileStatus` |
| Sessions | `SessionSummary`, `Message`, `MessageKind`, `Page<T>`, `TokenUsage` |
| Terminal | `TerminalSession`, `TerminalOpts`, `PermissionMode` |
| Models | `ModelMeta`, `ModelPricing` |
| Errors | `AppError`, `ErrorCode` |
| Misc | `Settings`, `AppConfig`, `ClaudeCliInfo`, `DirEntry`, `RequestId`, `SessionId` |

### Frontmatter examples

#### Agent
```yaml
---
name: Code Reviewer
description: Reviews PRs for security and clarity
model: sonnet            # 'opus' | 'sonnet' | 'haiku'
color: "#7c3aed"
memory: user             # 'user' | 'project' | 'local' | 'none'
skills: [refactor-helper]
tools:   [Read, Grep, Bash]
---

You are a senior reviewer. Focus on...
```

#### Command
```yaml
---
name: review-pr
description: Review the current PR
argument-hint: "[pr-number]"
allowed-tools: [Read, Bash]
agent: code-reviewer
---

Run a review against {{args}}...
```

#### Skill
```yaml
---
name: refactor-helper
description: Suggests safe refactors
context: when            # 'when' | 'always'
agent: code-reviewer
---

When the user asks to refactor...
```

#### Output Style
```yaml
---
name: Concise
description: Terse, no fluff
keepCodingInstructions: true
---

<style instructions body>
```

### Session storage

Sessions live in `~/.claude/projects/<encoded>/<sessionId>.jsonl`. The encoded path replaces `/` with `-` (`/Users/foo/app` → `-Users-foo-app`).

The Rust side parses each JSONL line with a tolerant deserializer (CLI line schemas drift across versions) and exposes a paginated `Page<Message>` for the session viewer:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_after: Option<usize>,
    pub total: Option<usize>,
}

pub struct Message {
    pub id: String,
    pub kind: MessageKind,
    pub role: Option<Role>,
    pub timestamp: String,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_result: Option<serde_json::Value>,
    pub thinking: Option<String>,
    pub is_error: bool,
}

pub enum MessageKind {
    Text,
    Thinking,
    ToolUse,
    ToolResult,
    Image,
    Status,
    Error,
}
```

There is **no** in-app concept of a chat session distinct from a CLI session. No JSONL written by `claude-code-gui` aside from `cli-history/` PTY snapshots.

### Token & pricing registry

`crates/core/src/models.rs`:

```rust
pub struct ServerModelMeta {
    pub api_id: &'static str,
    pub input_price_per_mtok: f64,
    pub output_price_per_mtok: f64,
    pub cache_read_price_per_mtok: f64,
    pub cache_write_price_per_mtok: f64,
    pub context_window: u32,
}

pub const MODEL_ALIAS_KEY_OPUS:   &str = "opus";
pub const MODEL_ALIAS_KEY_SONNET: &str = "sonnet";
pub const MODEL_ALIAS_KEY_HAIKU:  &str = "haiku";

pub fn pricing(alias: &str) -> Option<&'static ServerModelMeta>;
pub fn context_window(alias: &str) -> Option<u32>;
pub fn resolve(alias_or_id: &str) -> Option<&'static ServerModelMeta>;
```

Frontend has its own UI registry in `frontend/src/utils/models.ts` (label, color, tagline). Two registries on purpose: pricing changes don't trigger frontend rebuilds; UI palette changes don't touch the backend.

---

## 4. IPC Contract

### Conventions

- Command names: `snake_case`, prefixed by domain. `agents_list`, `terminal_session_create`.
- All commands return `Result<T, AppError>`.
- Payload types defined in Rust, exported via `ts-rs`.
- Events use a hierarchical name with `:` separators: `pty:output:{session_id}`.
- Frontend never constructs raw paths to `~/.claude/`; all path resolution happens in Rust.

### Error model

```rust
#[derive(Serialize, ts_rs::TS, Debug)]
#[ts(export)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub cause: Option<String>,
}

#[derive(Serialize, ts_rs::TS, Debug)]
#[ts(export)]
pub enum ErrorCode {
    NotFound,
    InvalidInput,
    IoError,
    YamlError,
    JsonError,
    ClaudeCli,
    Mcp,
    Git,
    Network,
    ResourceLimit,
    PermissionDenied,
    Internal,
}
```

Frontend wrapper:

```ts
import { invoke } from '@tauri-apps/api/core'
import type { Agent, AgentInput } from '@/types/ipc'

export const createAgent = (input: AgentInput) =>
  invoke<Agent>('agents_create', { input })
```

### Command catalog

#### Agents
- `agents_list() -> Vec<Agent>`
- `agents_get(slug: String) -> Agent`
- `agents_create(input: AgentInput) -> Agent`
- `agents_update(slug: String, input: AgentInput) -> Agent`
- `agents_delete(slug: String) -> ()`
- `agents_export(slug: String) -> String`
- `agents_import(payload: AgentImport) -> Agent`
- `agents_skills(slug: String) -> Vec<Skill>`
- `agents_skill_counts() -> HashMap<String, usize>`
- `agents_history_list(slug: String) -> Vec<HistoryEntry>`
- `agents_history_get(slug: String, id: String) -> HistoryEntry`
- `agents_history_delete(slug: String, id: String) -> ()`
- `agents_improve_instructions(input: ImproveRequest) -> RequestId`
  *(streams via `claude:improve:{request_id}` events)*

#### Commands
- `commands_list() -> Vec<Command>`
- `commands_get(slug: String) -> Command`
- `commands_create(input: CommandInput) -> Command`
- `commands_update(slug: String, input: CommandInput) -> Command`
- `commands_delete(slug: String) -> ()`
- `commands_execute(slug: String, args: Option<String>, working_dir: Option<String>) -> SessionId`
  *(spawns a terminal session preloaded with the command body; returns the session id for the UI to attach to)*

#### Skills
- `skills_list() -> Vec<Skill>`
- `skills_get(slug: String) -> Skill`
- `skills_create(input: SkillInput) -> Skill`
- `skills_update(slug: String, input: SkillInput) -> Skill`
- `skills_delete(slug: String) -> ()`
- `skills_export(slug: String) -> Vec<u8>`
- `skills_import(source: SkillImportSource) -> Vec<Skill>`
  *(supports `Github { url }`, `Local { path }`)*

#### Plans
- `plans_list() -> Vec<Plan>`
- `plans_get(slug: String) -> Plan`
- `plans_create(input: PlanInput) -> Plan`
- `plans_update(slug: String, input: PlanInput) -> Plan`
- `plans_delete(slug: String) -> ()`

#### Output styles
- `output_styles_list() -> Vec<OutputStyle>`
- `output_styles_get(id: String, scope: OutputStyleScope, working_dir: Option<String>) -> OutputStyle`
- `output_styles_create(input: OutputStyleInput) -> OutputStyle`
- `output_styles_delete(id: String, scope: OutputStyleScope, working_dir: Option<String>) -> ()`

#### Plugins
- `plugins_list() -> Vec<Plugin>`
- `plugins_get(id: String) -> PluginDetail`
- `plugins_delete(id: String) -> ()`
- `plugins_set_enabled(id: String, enabled: bool) -> ()`
- `plugins_update_skills(id: String, slugs: Vec<String>) -> ()`

#### Marketplace
- `marketplace_available() -> Vec<AvailablePlugin>`
- `marketplace_install(name: String, source: String) -> RequestId`
  *(streams via `marketplace:install:{request_id}` events)*
- `marketplace_uninstall(id: String) -> ()`
- `marketplace_sources_list() -> Vec<MarketplaceSource>`
- `marketplace_sources_add(input: MarketplaceSourceInput) -> ()`
- `marketplace_sources_remove(name: String) -> ()`
- `marketplace_sources_update(name: String) -> ()`

#### MCP
- `mcp_list(scope: McpScope, working_dir: Option<String>) -> Vec<McpServer>`
- `mcp_get(name: String, scope: McpScope, working_dir: Option<String>) -> McpServer`
- `mcp_create(input: McpServerInput, scope: McpScope, working_dir: Option<String>) -> McpServer`
- `mcp_delete(name: String, scope: McpScope, working_dir: Option<String>) -> ()`
- `mcp_capabilities(name: String, scope: McpScope, working_dir: Option<String>) -> McpCapabilities`
- `mcp_import(payload: McpImportPayload) -> Vec<McpServer>`

#### Projects (Claude project history)
- `projects_list() -> Vec<Project>`
- `projects_get(name: String) -> Project`
- `projects_create(path: String) -> Project`
- `projects_delete(name: String) -> ()`
- `projects_rename(name: String, new_name: String) -> ()`
- `projects_files(name: String, sub_path: Option<String>) -> Vec<FileNode>`
- `projects_git_status(name: String) -> GitStatus`
- `projects_settings_get(name: String) -> Settings`
- `projects_settings_put(name: String, settings: Settings) -> ()`
- `projects_claude_md_get(name: String) -> String`
- `projects_claude_md_put(name: String, content: String) -> ()`
- `projects_resolve(path: String) -> ProjectInfo`

#### Sessions (CLI sessions inside a project)
- `sessions_list_for_project(name: String) -> Vec<SessionSummary>`
- `sessions_messages(session_id: String, after_index: Option<usize>, limit: Option<usize>) -> Page<Message>`
- `sessions_rename(session_id: String, new_name: String) -> ()`
- `sessions_delete(project_name: String, session_id: String) -> ()`

#### Settings / Config / Setup
- `settings_get() -> Settings`
- `settings_put(settings: Settings) -> ()`
- `config_get() -> AppConfig`
- `config_set(config: AppConfig) -> ()`
- `setup_finalize(payload: SetupPayload) -> ()`

#### Filesystem utilities
- `directories_list(parent: String) -> Vec<DirEntry>`
- `files_read(path: String) -> String`
- `reveal_in_finder(path: String) -> ()`
- `pick_folder() -> Option<String>` (uses `tauri-plugin-dialog`)

#### Terminal
- `terminal_session_create(opts: TerminalOpts) -> SessionId`
- `terminal_session_input(session_id: String, data: String) -> ()`
- `terminal_session_resize(session_id: String, cols: u16, rows: u16) -> ()`
- `terminal_session_kill(session_id: String) -> ()`
- `terminal_sessions_list() -> Vec<TerminalSession>`
- `terminal_session_get(session_id: String) -> TerminalSession`

`TerminalOpts`:

```rust
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpts {
    pub agent_slug: Option<String>,
    pub working_dir: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub model: Option<String>,
    pub permission_mode: Option<PermissionMode>,
    pub output_style_id: Option<String>,
    pub resume_session_id: Option<String>,
    pub command_template: Option<String>,   // for commands_execute
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    BypassPermissions,
    Plan,
}
```

#### Debug / utility
- `debug_claude_cli() -> ClaudeCliInfo`
- `app_version() -> String`
- `app_open_url(url: String) -> ()`

### Event catalog

| Event | Payload | Source | Consumer |
|-------|---------|--------|----------|
| `pty:output:{session_id}` | `{ data: string }` | PTY reader task | `Terminal.vue` |
| `pty:exit:{session_id}` | `{ exitCode: number }` | PTY reader task | `Terminal.vue` |
| `fs:change` | `{ path, kind: 'create' \| 'modify' \| 'delete' }` | global watcher | list pages, file tree |
| `claude:improve:{request_id}` | `{ kind: 'delta' \| 'done' \| 'error', text?, error? }` | `claude_cli::improve_instructions` | improve modal |
| `context:tokens:{session_id}` | `{ input, output, cached, cost, model }` | context monitor | `MetricsCard.vue` |
| `context:tool:{session_id}` | `ToolCall` | context monitor | `ToolTimeline.vue` |
| `marketplace:install:{request_id}` | `{ kind: 'progress' \| 'done' \| 'error', step?, error? }` | marketplace task | install modal |
| `app:claude_dir_changed` | `{ path: string }` | settings command | every page |
| `app:single_instance` | `{ args: string[] }` | `tauri-plugin-single-instance` | router (deep-link handling) |

### Concurrency rules

- Long-running tasks identified by `request_id` or `session_id`. Command call returns the id; progress arrives via events; frontend correlates by id.
- Soft caps:
  - 16 simultaneous PTY sessions
  - 4 concurrent `claude -p` subprocesses
- Beyond cap → `AppError { code: ResourceLimit }`.

### Tauri capabilities

`tauri.conf.json`:

```jsonc
{
  "permissions": [
    "core:default",
    "event:default",
    "dialog:default",
    "shell:open",
    "store:default",
    "process:default",
    "opener:default",
    "updater:default",
    "deep-link:default",
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$HOME/.claude/**" }
      ]
    }
  ]
}
```

Project working directories are added dynamically at runtime via `fs.scope.allow` (Tauri 2 supports runtime scope mutation).

The `shell` plugin is **not** allowed for arbitrary commands. PTY and `claude -p` use `tokio::process::Command` from inside Rust; only hard-coded binary names are spawned.

---

## 5. Pages & Navigation

File-based router under `frontend/src/pages/`. Default landing page: `/agents`.

### Route table

| Path | File | Purpose |
|------|------|---------|
| `/` | (redirect) | → `/agents` |
| `/agents` | `pages/agents/index.vue` | List + create agents |
| `/agents/:slug` | `pages/agents/[slug].vue` | Edit agent + embedded test terminal |
| `/commands` | `pages/commands/index.vue` | Slash command list |
| `/commands/:slug` | `pages/commands/[slug].vue` | Edit command |
| `/skills` | `pages/skills/index.vue` | Skill list (local + plugin) |
| `/skills/:slug` | `pages/skills/[slug].vue` | Edit skill |
| `/plans` | `pages/plans/index.vue` | Plan list |
| `/plans/:slug` | `pages/plans/[slug].vue` | Plan editor |
| `/mcp` | `pages/mcp/index.vue` | MCP server list |
| `/mcp/:name` | `pages/mcp/[name].vue` | MCP detail + capabilities probe |
| `/output-styles` | `pages/output-styles/index.vue` | Output style manager |
| `/plugins` | `pages/plugins/index.vue` | Installed + Discover tabs |
| `/plugins/:id` | `pages/plugins/[id].vue` | Plugin detail |
| `/sessions` | `pages/sessions/index.vue` | Project picker |
| `/sessions/project/:projectName` | `pages/sessions/project/[projectName]/index.vue` | Project view: session list + git status |
| `/sessions/project/:projectName/session/:sessionId` | `…/session/[sessionId].vue` | Session viewer + Resume terminal |
| `/sessions/project/:projectName/settings` | `…/settings.vue` | Project settings + CLAUDE.md editor |
| `/settings` | `pages/settings.vue` | Global settings |

### Cross-page components

| Component | Used in |
|-----------|---------|
| `AppShell` | Root layout: sidebar, header, content slot |
| `Sidebar` | Nav with section counts |
| `PageHeader` | Title + action row |
| `GlobalSearch` | Header search across agents/commands/skills/plans |
| `OnboardingFlow` | First-run wizard, gated by `settings.onboardingCompleted` |
| `AgentCard`, `AgentForm`, `AgentWizard` | Agent CRUD UI |
| `CommandForm`, `SkillForm` | Form components |
| `ChatTerminal` | Embedded xterm-based terminal (replaces ChatPanel) |
| `MetricsCard`, `FileTree`, `ToolTimeline`, `SessionHistory` | Context panel pieces |
| `AddMcpModal`, `AddPluginModal`, `AddOutputStyleModal`, `AddMarketplaceModal` | Creation modals |
| `MarketplaceSourceRow` | Source row in marketplace tab |
| `HelpTip`, `FeatureCallout` | Inline guidance |
| `FileImport` | Import flows |

### Page layouts

#### Agent detail (`/agents/:slug`)

Two-pane:
- **Left**: `AgentForm` — frontmatter (name, model, color, memory, skills, tools, directory) + body (markdown).
- **Right**: `ChatTerminal` mounted with the agent preloaded:
  ```ts
  await invoke('terminal_session_create', {
    opts: {
      agentSlug: agent.slug,
      workingDir: workingDir.value,
      cols: term.cols,
      rows: term.rows,
      model: agent.frontmatter.model,
      permissionMode: settings.value.defaultPermissionMode,
    }
  })
  ```

Above the terminal: a `claude --resume` tip showing the latest session id derived from `~/.claude/projects/`.

#### Session viewer

Read-only history pane backed by `sessions_messages`. A "Resume in terminal" button opens a `ChatTerminal` with `resumeSessionId` set, spawning `claude --resume <id>` in the same project's working directory.

#### Plugins

Two tabs:
- **Installed**: from `plugins_list()`
- **Discover**: from `marketplace_available()`. Sources managed inline at the top.

Install action calls `marketplace_install`, listens on `marketplace:install:{id}`, refreshes the list on `kind: 'done'`.

#### Settings

Sections:
- Claude directory (override path, persisted via `tauri-plugin-store`)
- Claude CLI binary (probe path + version via `debug_claude_cli`)
- Default permission mode for new terminals
- Default model for new agents
- Hooks editor (raw JSON)
- Output style preferences
- Onboarding redo
- Updater channel + check now

### State conventions

- Server data lives in composables — `useAgents`, `useCommands`, etc. — each wrapping `invoke()` calls and exposing `items`, `loading`, `error`.
- No optimistic updates by default. After write, refetch.
- Forms use `useUnsavedChanges` to block route navigation when dirty.
- Drafts persist to `localStorage` via `useDraftRecovery` for crash recovery.

### Sidebar counts

Each nav item shows a badge with the entity count. Counts populated at app boot from list endpoints; refreshed on `fs:change` events that match the relevant root.

---

## 6. Terminal Subsystem

The terminal IS the chat. There is no separate streaming chat layer.

### Frontend

`ChatTerminal.vue` wraps `xterm.js` + `@xterm/addon-fit` + `@xterm/addon-web-links`:

```vue
<script setup lang="ts">
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import type { TerminalOpts } from '@/types/ipc'

const props = defineProps<{ opts: TerminalOpts }>()
const sessionId = ref<string>()

onMounted(async () => {
  const term = new Terminal({ fontFamily: 'Geist Mono', fontSize: 13 })
  const fit = new FitAddon()
  term.loadAddon(fit)
  term.loadAddon(new WebLinksAddon())
  term.open(el.value!)
  fit.fit()

  const id = await invoke<string>('terminal_session_create', { opts: props.opts })
  sessionId.value = id

  const unlisten = await listen<{ data: string }>(`pty:output:${id}`, (e) => {
    term.write(e.payload.data)
  })

  term.onData((data) => invoke('terminal_session_input', { sessionId: id, data }))
  term.onResize(({ cols, rows }) => invoke('terminal_session_resize', { sessionId: id, cols, rows }))

  onBeforeUnmount(async () => {
    unlisten()
    if (sessionId.value) await invoke('terminal_session_kill', { sessionId: sessionId.value })
  })
})
</script>
```

### Backend (`pty` crate)

```rust
pub struct PtyManager {
    sessions: Arc<tokio::sync::Mutex<HashMap<Uuid, PtySession>>>,
    app: AppHandle,
}

struct PtySession {
    handle: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn std::io::Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    meta: TerminalMeta,
    output_buffer: Mutex<RingBuffer<String>>,   // last 10K lines
}

impl PtyManager {
    pub async fn create(&self, opts: TerminalOpts) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: opts.rows, cols: opts.cols, pixel_width: 0, pixel_height: 0
        })?;

        let cmd = compose_command(&opts).await?;
        let child = pair.slave.spawn_command(cmd)?;

        // Spawn reader task
        let app = self.app.clone();
        let reader = pair.master.try_clone_reader()?;
        tokio::spawn(reader_task(id, app.clone(), reader));

        // Spawn exit watcher
        tokio::spawn(exit_watcher(id, app.clone(), child_clone));

        // Insert session
        self.sessions.lock().await.insert(id, PtySession { /* ... */ });
        Ok(id)
    }

    pub async fn input(&self, id: Uuid, data: &[u8]) -> Result<()> { /* writer.write_all */ }
    pub async fn resize(&self, id: Uuid, cols: u16, rows: u16) -> Result<()> { /* handle.resize */ }
    pub async fn kill(&self, id: Uuid) -> Result<()> { /* child.kill */ }
}
```

### Command composition

```rust
async fn compose_command(opts: &TerminalOpts) -> Result<CommandBuilder> {
    if let Some(slug) = &opts.agent_slug {
        let agent = core::agents::get(slug)?;
        let claude = core::claude_cli::path()?;

        let mut cmd = CommandBuilder::new(claude);
        cmd.arg("--append-system-prompt").arg(agent.body);

        if let Some(model) = agent.frontmatter.model.or(opts.model.clone()) {
            cmd.arg("--model").arg(model);
        }
        if let Some(mode) = &opts.permission_mode {
            cmd.arg("--permission-mode").arg(mode.as_cli_flag());
        }
        if let Some(style) = &opts.output_style_id {
            cmd.arg("--output-style").arg(style);
        }
        if let Some(resume) = &opts.resume_session_id {
            cmd.arg("--resume").arg(resume);
        }
        if let Some(wd) = &opts.working_dir {
            cmd.cwd(wd);
        }
        Ok(cmd)
    } else {
        // Default shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let mut cmd = CommandBuilder::new(shell);
        if let Some(wd) = &opts.working_dir { cmd.cwd(wd); }
        Ok(cmd)
    }
}
```

### Session lifecycle

- Idle for 30 min → killed (configurable).
- Output buffered in memory (last 10K lines).
- On exit: metadata + buffer snapshotted to `~/.claude/cli-history/<id>.json`.
- App quit → all PTYs killed cleanly via `app.on_window_event(CloseRequested, ...)`.

### Permission prompts

The Claude CLI prints `[y/n]` prompts inline. The user types directly into the PTY. There is **no** out-of-band permission UI in `claude-code-gui` — we don't intercept tool calls.

For users who want to skip prompts, the launcher exposes `permissionMode: 'bypassPermissions'` in `TerminalOpts`.

### `claude_cli` crate (one-shot only)

The single non-PTY use of the CLI: `agents_improve_instructions`. Spawns `claude -p --output-format stream-json --input-format stream-json`, writes a JSON message to stdin, reads stdout line-by-line, emits `claude:improve:{request_id}` events with `kind: 'delta' | 'done' | 'error'`.

```rust
pub async fn improve_instructions(
    app: AppHandle,
    request_id: Uuid,
    input: ImproveRequest,
) -> Result<()> {
    let mut child = Command::new(claude_path()?)
        .args(["-p", "--output-format", "stream-json",
               "--input-format", "stream-json",
               "--append-system-prompt", &input.system])
        .stdin(Stdio::piped()).stdout(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    write_stream_json_message(stdin, &input.prompt).await?;

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    while let Some(line) = lines.next_line().await? {
        let event: StreamJsonEvent = serde_json::from_str(&line)?;
        match event.kind() {
            "text_delta" => emit(&app, format!("claude:improve:{request_id}"),
                                  json!({ "kind": "delta", "text": event.text })),
            "result" => { emit(&app, ..., json!({ "kind": "done" })); break; }
            _ => {}
        }
    }
    Ok(())
}
```

---

## 7. File Watcher & Context Monitor

### `watcher` crate

```rust
pub struct WatcherHandle {
    debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    subscriptions: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

pub fn start_global(app: AppHandle) -> Result<WatcherHandle>;
pub fn watch(handle: &WatcherHandle, path: &Path) -> Result<Uuid>;
pub fn unwatch(handle: &WatcherHandle, id: Uuid);
```

Powered by `notify-debouncer-mini` with a 200 ms debounce. Each filesystem event becomes:

```jsonc
// emit "fs:change"
{ "path": "/Users/foo/.claude/agents/reviewer.md", "kind": "modify" }
```

The watcher always covers `~/.claude/`. Project working directories are added dynamically when the user opens a project page.

Frontend filters by path prefix client-side. Debounce + filter keeps event volume well under the chatter threshold.

### Context monitor (inside `pty` crate)

Per-session. The PTY reader pipes each chunk through `parse_chunk`:

```rust
fn parse_chunk(chunk: &str, state: &mut MonitorState, app: &AppHandle, id: Uuid) {
    state.line_buf.push_str(chunk);

    while let Some(line) = state.line_buf.next_line() {
        // 1. tokens: <n> in, <n> out, <n> cache_read, <n> cache_write
        if let Some(tokens) = TOKEN_RE.captures(&line) {
            let usage = TokenUsage::from(tokens);
            if let Some(model) = state.last_model.as_deref() {
                let cost = compute_cost(model, &usage);
                emit(app, format!("context:tokens:{id}"), json!({
                    "input": usage.input,
                    "output": usage.output,
                    "cached": usage.cache_read,
                    "cost": cost,
                    "model": model,
                }));
            }
        }

        // 2. usage: ... model=<slug>
        if let Some(m) = MODEL_RE.captures(&line) {
            state.last_model = Some(m[1].to_string());
        }

        // 3. [tool] <Name> started / completed in <ms>ms
        if let Some(tc) = TOOL_RE.captures(&line) {
            emit(app, format!("context:tool:{id}"), tc.into_event());
        }
    }
}

fn compute_cost(model: &str, u: &TokenUsage) -> f64 {
    let m = core::models::resolve(model).unwrap_or_default();
    (u.input  as f64 * m.input_price_per_mtok        / 1e6)
  + (u.output as f64 * m.output_price_per_mtok       / 1e6)
  + (u.cache_read as f64 * m.cache_read_price_per_mtok / 1e6)
  + (u.cache_write as f64 * m.cache_write_price_per_mtok / 1e6)
}
```

Caveat: regex-based parsing is brittle to upstream Claude CLI output format changes. Each `claude` upgrade requires a smoke test of the metric panel.

### Why not parse stream-json?

The CLI doesn't emit `stream-json` in interactive PTY mode. Only `-p` does. So context monitoring during a real conversation has to be regex-based. Tradeoff accepted.

---

## 8. Distribution & Runtime

### Build

```bash
# Dev
cd src-tauri && cargo tauri dev          # auto-runs `bun --cwd ../frontend dev`

# Release
cargo tauri build                        # produces signed installers
```

### Bundle output

| Platform | Artifacts | Target size |
|----------|-----------|-------------|
| macOS    | `.dmg`, `.app.tar.gz` (universal2: `aarch64-apple-darwin` + `x86_64-apple-darwin`) | < 18 MB |
| Windows  | `.msi`, `.exe` (NSIS) | < 12 MB |
| Linux    | `.AppImage`, `.deb`, `.rpm` | < 18 MB |

### `tauri.conf.json` highlights

```jsonc
{
  "productName": "Claude Code GUI",
  "version": "0.1.0",
  "identifier": "com.anthropic.claude-code-gui",
  "build": {
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "bun --cwd ../frontend dev",
    "beforeBuildCommand": "bun --cwd ../frontend build"
  },
  "app": {
    "windows": [{
      "label": "main",
      "title": "Claude Code GUI",
      "width": 1440,
      "height": 900,
      "minWidth": 960,
      "minHeight": 600,
      "decorations": true,
      "titleBarStyle": "Overlay",
      "hiddenTitle": true,
      "visible": true
    }],
    "security": {
      "csp": "default-src 'self'; connect-src 'self' ipc: http://ipc.localhost; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'; font-src 'self' data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app", "msi", "appimage", "deb", "rpm"],
    "category": "DeveloperTool",
    "shortDescription": "Visual manager for Claude Code",
    "longDescription": "Manage Claude Code agents, commands, skills, plugins, and sessions.",
    "macOS": {
      "minimumSystemVersion": "12.0",
      "signingIdentity": "Developer ID Application: ...",
      "entitlements": "entitlements.plist"
    },
    "windows": {
      "wix": { "language": "en-US" },
      "certificateThumbprint": null
    }
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://updates.example.com/{{target}}/{{current_version}}"],
      "pubkey": "..."
    },
    "deep-link": {
      "schemes": ["claude-code-gui"]
    }
  }
}
```

### Code signing

- **macOS**: Apple Developer ID Application + notarization. Run `xcrun notarytool` post-build via CI; staple ticket back into the bundle.
- **Windows**: Authenticode signing certificate. Configure `bundle.windows.certificateThumbprint`.
- **Linux**: optional GPG signing for repo distribution. AppImage doesn't strictly require signing.

### Auto-update

`tauri-plugin-updater`:
- Update channel selection in `/settings` (`stable` / `beta`).
- Manifest endpoint returns JSON with `version`, `notes`, `pub_date`, per-target signed bundle URLs.
- Public key embedded in binary at build time. Private key stored in CI secrets.

### Single instance + deep links

`tauri-plugin-single-instance`: second invocation routes args to the existing window.

`claude-code-gui://install/<plugin-name>?source=<source>`: handled by routing to the marketplace install flow.

### Environment variables

| Var | Default | Effect |
|-----|---------|--------|
| `CLAUDE_DIR` | `~/.claude` | Override base dir. Read once at boot; UI override persisted via `tauri-plugin-store`. |
| `CLAUDE_CLI_PATH` | resolved via `which` | Path to `claude` binary |
| `RUST_LOG` | `info` | `tracing` filter |

No `PORT` / `HOST` — there is no listening socket.

### Logging

`tracing` + `tracing-appender`:
- macOS: `~/Library/Logs/com.anthropic.claude-code-gui/app.log`
- Windows: `%LOCALAPPDATA%\com.anthropic.claude-code-gui\logs\`
- Linux: `~/.local/share/com.anthropic.claude-code-gui/logs/`

Daily rotation, 14-day retention.

### Crash resilience

- Rust panics in commands caught at the IPC boundary, returned as `AppError { code: Internal }`.
- PTY tasks that panic emit `pty:exit:{id}` with `exitCode: -1` and clean up state.
- File watcher recovers from transient `notify` errors; permanent errors set the watcher to a degraded state and emit a UI banner.
- App quit cleanly kills all PTYs, awaits flush, persists `cli-history/`.

### Security model

- No network listener. IPC is in-process only.
- `fs` plugin scoped to `~/.claude/**` and user-selected project dirs. No arbitrary filesystem access.
- `shell` plugin denies arbitrary command execution. Only Rust-side `tokio::process::Command` spawns processes, with hard-coded binary names.
- Marketplace fetch uses `reqwest` with `rustls-tls` (system trust roots). No HTTP plaintext.
- Git operations use `git2` with `vendored-libgit2`; HTTPS only; GPG verification optional and per-source.
- WebView CSP locks down inline scripts and remote resources. Only fonts and data URLs allowed.
- Deep links validated against an allow-list of action verbs (`install`, `open-agent`).

---

## 9. Extensibility

### Adding a Tauri command

1. Define input/output structs in `crates/core/src/types/<domain>.rs`. Derive `Serialize, Deserialize, ts_rs::TS, Debug, Clone`.
2. Implement the function in `crates/core/src/<domain>.rs`. No Tauri imports.
3. Add a thin wrapper in `crates/app/src/commands/<domain>.rs`:

```rust
#[tauri::command]
pub async fn agents_create(
    state: State<'_, AppState>,
    input: AgentInput,
) -> Result<Agent, AppError> {
    core::agents::create(input).map_err(Into::into)
}
```

4. Register in the `tauri::generate_handler!` macro in `crates/app/src/main.rs`.
5. Frontend: add a thin invoker in `frontend/src/utils/ipc.ts`:

```ts
export const createAgent = (input: AgentInput) =>
  invoke<Agent>('agents_create', { input })
```

6. Run `cargo test --workspace` to regenerate `frontend/src/types/ipc/`.

### Adding an event

1. Define payload type in `crates/core/src/types/events.rs`. Derive `Serialize, ts_rs::TS`.
2. Add the event name constant in `crates/app/src/events.rs`.
3. Emit via `app.emit(event_name, payload)?`.
4. Frontend: subscribe via `listen(event_name, handler)` from `@tauri-apps/api/event`. Tear down on unmount.

### Adding a model

`crates/core/src/models.rs` — add to `MODEL_ALIAS` and `SERVER_MODEL_META` (alias key, full Anthropic id, pricing, context window).

Frontend `frontend/src/utils/models.ts` — add to `MODEL`, `MODEL_IDS`, `MODEL_META` (UI metadata only — label, color, tagline).

`ts-rs` regenerates `frontend/src/types/ipc/ModelMeta.ts` automatically.

Comparisons in code use named constants only:

```ts
import { MODEL } from '@/utils/models'
if (model === MODEL.SONNET) { /* ... */ }
```

```rust
use core::models::MODEL_ALIAS_KEY_SONNET;
if model == MODEL_ALIAS_KEY_SONNET { /* ... */ }
```

No raw model strings in logic. Ever.

### Adding a marketplace source type

`crates/core/src/marketplace.rs`:

```rust
pub enum SourceType {
    Github,
    Http,
    // Add new variants here
}
```

Add a fetch branch + install branch. Frontend `AddMarketplaceModal` already has a `sourceType` dropdown — add the option there.

### Adding a permission mode

1. Add a variant to `PermissionMode` in `crates/core/src/types/permission.rs`.
2. Update `PermissionMode::as_cli_flag()` to translate it to the matching `claude` CLI flag.
3. Update the picker component in the agent test panel.

### Adding a CRUD entity

To add e.g. `Snippets` stored under `~/.claude/snippets/<slug>.md`:

1. Types: `Snippet`, `SnippetFrontmatter`, `SnippetInput` in `crates/core/src/types/snippets.rs`
2. Domain logic: `crates/core/src/snippets.rs` (`list`, `get`, `create`, `update`, `delete`)
3. Commands: `crates/app/src/commands/snippets.rs`
4. Register in `generate_handler!`
5. Frontend composable: `useSnippets` wrapping the invokers
6. Pages: `frontend/src/pages/snippets/index.vue` and `[slug].vue`
7. Sidebar entry
8. (Optional) hook into `crates/core/src/relationships.rs` so snippets show up in relationship queries

### Tests

- `cargo test --workspace` covers every `core::*` function with temp-dir `CLAUDE_DIR` fixtures.
- Snapshot tests via `insta` for frontmatter round-tripping.
- Integration test that spawns `cat` as a fake PTY end-to-end.
- `fixtures/` directory copied from a real `~/.claude/` for golden tests of session JSONL parsing.
- Frontend: minimal — visual regression via Playwright on a handful of golden screenshots. Skip unit tests for Vue.

---

## 10. Implementation Roadmap

Phased delivery. Each phase is independently shippable as a private build.

### Phase 0 — Scaffolding (1 week)

- [ ] `cargo new` workspace + Tauri 2 init (`bun create tauri-app`)
- [ ] Vue 3 + Vite + vue-router + Tailwind in `frontend/`
- [ ] `ts-rs` export pipeline; first generated types committed
- [ ] CI matrix: macOS / Windows / Linux on `cargo tauri build --debug`
- [ ] No-op WebView showing the SPA
- [ ] App shell, sidebar, empty pages for every route

### Phase 1 — Read-only CRUD (2 weeks)

- [ ] `claude_dir` resolution + `tauri-plugin-store` for override persistence
- [ ] `frontmatter` parser + round-trip tests
- [ ] List/get for: agents, commands, skills, plans, output-styles, mcp, plugins, projects, sessions
- [ ] Settings + config + debug_claude_cli
- [ ] Acceptance: every list page renders correctly against an existing `~/.claude/`.

### Phase 2 — Write CRUD (2 weeks)

- [ ] Create/update/delete for all CRUD entities
- [ ] Import/export for agents, skills
- [ ] `setup_finalize` (first-run wizard)
- [ ] `directories_list`, `files_read`, `reveal_in_finder`, `pick_folder`
- [ ] Project create/rename/delete + git status

### Phase 3 — Async surfaces (1 week)

- [ ] `claude_cli::improve_instructions` + `claude:improve:{id}` events
- [ ] `marketplace_install` progress events
- [ ] File watcher + `fs:change` events
- [ ] Marketplace source CRUD + plugin install/uninstall

### Phase 4 — Terminal (2 weeks)

- [ ] `pty` crate + `portable-pty` integration
- [ ] `terminal_*` commands and `pty:output:{id}` / `pty:exit:{id}` events
- [ ] Context monitor: `context:tokens:{id}` / `context:tool:{id}` events
- [ ] `ChatTerminal.vue` component
- [ ] Embed in agent detail right pane
- [ ] Embed in session viewer (Resume mode)
- [ ] Permission mode flag wiring

### Phase 5 — MCP + relationships (1 week)

- [ ] `rmcp` integration for capability probing
- [ ] MCP CRUD pages + capability detail page
- [ ] Relationship extractor (agent ↔ skill, agent ↔ command)

### Phase 6 — Distribution (2 weeks)

- [ ] Code-signing certificates (Apple Developer ID, Windows authenticode)
- [ ] CI build + sign + notarize per platform
- [ ] Auto-updater manifest pipeline
- [ ] Deep-link handler (`claude-code-gui://install/...`)
- [ ] Single-instance plugin
- [ ] Release notes template + version bump scripts
- [ ] Public beta release

### Acceptance gates

Before shipping 1.0:

- All Goals from §1 satisfied
- A user with an existing `~/.claude/` can install the app, open it, and see all of their agents/commands/skills/plugins/sessions without any migration step
- The terminal works against `claude` versions ≥ 2.0
- Bundle size under 20 MB on all three platforms
- No runtime dependency on Node, npm, or any TS server code
- CI green on all three platforms
- Code signed and notarized
- Auto-updater verified end-to-end on at least one platform

### Out of scope for 1.0

- Image attachments in terminal (Claude CLI doesn't expose them via PTY interactively)
- Custom MCP server templates
- Cross-machine sync (no cloud component planned)
- Alternative LLM providers — `claude-code-gui` is Claude-only by design
- Embedded model gateway / local proxy
