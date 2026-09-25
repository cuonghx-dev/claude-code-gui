# Hướng dẫn sử dụng Claude Code GUI

Claude Code GUI là ứng dụng desktop để quản lý trực quan mọi thứ mà Claude Code CLI lưu trên đĩa: agents, slash commands, skills, plans, workflows, output styles, MCP servers, hooks, plugins, memory, settings và lịch sử session. App đọc/ghi **đúng các file mà CLI dùng**, nên mọi thay đổi trong app có hiệu lực ngay với `claude` và ngược lại.

> Trong tài liệu này, `~/.claude` là thư mục Claude. Mặc định là `$HOME/.claude`; có thể đổi bằng biến môi trường `CLAUDE_DIR` hoặc trong **Settings → General → Claude directory override**.

## Mục lục

1. [Bắt đầu](#1-bắt-đầu)
2. [Giao diện chung](#2-giao-diện-chung)
3. [Authoring](#3-authoring) — Agents, Commands, Skills, Plans, Workflows, Output styles
4. [Extend](#4-extend) — MCP, Hooks, Plugins
5. [Activity](#5-activity) — Sessions, Jobs, Usage
6. [Config](#6-config) — Memory, .claude
7. [Settings](#7-settings)
8. [Deep links](#8-deep-links)
9. [Phím tắt](#9-phím-tắt)
10. [Bảng file app đọc/ghi](#10-bảng-file-app-đọcghi)
11. [Hạn chế hiện tại](#11-hạn-chế-hiện-tại)

---

## 1. Bắt đầu

### Yêu cầu

- `claude` CLI có trong `PATH`. Trên macOS app tự kế thừa `PATH` của login shell, nên cài qua Homebrew / npm global / nvm đều chạy được.
- Cài đặt: xem [README](../README.md#install-macos-pre-built).

### Onboarding (lần chạy đầu)

Lần đầu mở app, một hộp thoại **Welcome to Claude Code GUI** gồm 3 bước (Back / Next / Finish):

| Bước | Nội dung |
|---|---|
| 1. Default model | `opus` / `sonnet` (mặc định) / `haiku` — dùng khi agent không chỉ định model. |
| 2. Permission mode & Theme | Permission mode mặc định khi mở terminal mới (`default`, `acceptEdits`, `plan`, `auto`, `dontAsk`, `manual`, `bypassPermissions`); theme: theo hệ thống / sáng / tối. |
| 3. Claude directory (nâng cao) | Đường dẫn thư mục Claude nếu không dùng `~/.claude` (có nút **Browse…**). |

Khi bấm **Finish**:
- `defaultModel`, `defaultPermissionMode`, `onboardingCompleted: true` được ghi vào `~/.claude/settings.json` (chỉ các key này, các key khác giữ nguyên).
- Theme và thư mục override lưu trong config riêng của app.

Muốn chạy lại onboarding: **Settings → General → Replay onboarding**, rồi reload app.

---

## 2. Giao diện chung

### Sidebar

| Nhóm | Mục |
|---|---|
| Authoring | Agents, Commands, Skills, Plans, Workflows, Output styles |
| Extend | MCP, Hooks, Plugins |
| Activity | Sessions, Jobs, Usage |
| Config | Memory, .claude |
| (cuối) | Settings |

Mỗi mục hiển thị số lượng phần tử hiện có. Nút **Search** ở đầu sidebar mở tìm kiếm toàn cục.

### Tìm kiếm toàn cục (⌘K / Ctrl+K)

- Tìm trong **agents, commands, skills, plans** (không tìm sessions, workflows, output styles).
- Xếp hạng: khớp đầu tên/slug → tên chứa từ khoá → description → nội dung. Tối đa 30 kết quả.
- `↑` / `↓` di chuyển, `Enter` mở, `Esc` đóng.
- Phím tắt bị tắt khi con trỏ đang ở trong terminal nhúng (Ctrl+K thuộc về Claude TUI).

### Tự động làm mới

App theo dõi filesystem: sửa file trong `~/.claude/` bằng editor khác hay bằng CLI thì danh sách trong app tự cập nhật, không cần khởi động lại.

### Hành vi chung của các trang soạn thảo

- **Editor**: CodeMirror 6 — số dòng, tự xuống dòng, undo/redo. Markdown highlight (workflows dùng JavaScript). Sửa **toàn bộ file dạng text**, không có form frontmatter hay preview.
- **Cảnh báo thay đổi chưa lưu**: rời trang (hoặc đóng cửa sổ) khi còn thay đổi → hỏi "Discard unsaved changes?". Nút **Save** chỉ sáng khi có thay đổi.
- **Khôi phục nháp** (chỉ trang *New*): nội dung đang gõ được tự lưu; mở lại trang *New* sẽ tự khôi phục. Nháp quá 7 ngày bị xoá; bấm **Create** thành công hoặc **Cancel** thì nháp bị xoá.
- **Xoá**: luôn có hộp thoại xác nhận. Xoá là **vĩnh viễn** (không vào Thùng rác).
- **Kiểm tra khi lưu**:
  - Frontmatter (nếu có) phải là YAML hợp lệ.
  - Slug: 1–64 ký tự, chỉ `a-z A-Z 0-9 - _`, không bắt đầu/kết thúc bằng `-`.
  - Tạo mới trùng slug → lỗi "... already exists", không ghi đè.
  - Ghi file theo kiểu atomic — không bao giờ để lại file ghi dở.
- **Đổi `name:` khi sửa không đổi tên file.** Slug được cố định lúc tạo.

---

## 3. Authoring

### 3.1 Agents

File: `~/.claude/agents/**/*.md` (đọc cả thư mục con; tạo mới luôn ở cấp gốc `~/.claude/agents/<slug>.md`).

**Danh sách**
- Mỗi thẻ: chấm màu (`color`), tên, mô tả, badge model, 3 tool đầu tiên.
- Ô **Filter…**: lọc theo slug / tên / mô tả.
- **Import**: chọn file `.md` → copy nguyên văn vào `~/.claude/agents/<tên-file>.md` (slug lấy từ **tên file**, không phải `name:`).
- **+ New**: tạo agent mới.

**Tạo mới** — editor có sẵn template:

```markdown
---
name: my-agent
description: "Describe when this agent should be used."
tools: Read, Write, Edit, Bash, Glob, Grep
model: sonnet
---
```

Slug suy ra từ `name:` trong frontmatter: chuyển chữ thường, ký tự ngoài `a-z0-9_-` thành `-`, cắt tối đa 64 ký tự. Không có `name:` → `untitled-<timestamp>`.

**Chi tiết** — nút:
- **Relationships (N)**: panel đồ thị liên kết (skills agent dùng, commands gắn với agent). Bấm node để mở.
- **Export**: tải về `<slug>.md`.
- **Delete**, **Save**.

### 3.2 Commands (slash commands)

File: `~/.claude/commands/**/*.md`.

- Danh sách hiển thị `/slug`, `argument-hint`, mô tả.
- Template tạo mới gồm `name`, `description`, `argument-hint: "[args]"`, `allowed-tools: [Read, Bash]` và body `Run the command using {{args}}.`
- Trang chi tiết: **Relationships**, **Delete**, **Save** (không có Export).

### 3.3 Skills

File:
- Skill local: `~/.claude/skills/<slug>/SKILL.md` (cùng các file khác trong thư mục).
- Skill từ plugin: `~/.claude/plugins/<plugin-id>/skills/<slug>/SKILL.md` — **chỉ đọc**.

**Danh sách** hiển thị badge `local` hoặc `plugin: <id>`. Ba cách thêm skill:

| Cách | Mô tả |
|---|---|
| **+ New** | Template `name`, `description`, `context`. App tạo thư mục và `SKILL.md`. |
| **Import folder** | Chọn thư mục có `SKILL.md`; tên thư mục thành slug; copy toàn bộ vào `~/.claude/skills/<slug>/`. |
| **Import from GitHub** | Dán URL: `https://github.com/owner/repo`, `.../tree/<ref>/<path>` hoặc `.../blob/<ref>/<path>/SKILL.md`. Giới hạn 200 file / 5 MiB. Repo public không cần token; nếu có biến `GH_TOKEN`/`GITHUB_TOKEN` app sẽ dùng (tăng rate limit). Import lỗi không để lại rác. Lịch sử import ghi ở `~/.claude/.imports.json`. |

> Branch có dấu `/` trong tên chưa được hỗ trợ khi import GitHub.

**Chi tiết**
- Skill local: **Export** (tải cả thư mục dạng `<slug>.tar`), **Delete** (xoá cả thư mục), **Save**, **Relationships** (các agent tham chiếu skill này).
- Skill plugin: banner vàng "read-only", không có nút lưu/xoá — sửa ở plugin gốc.

### 3.4 Plans

File: `~/.claude/plans/*.md` — markdown thuần, không parse frontmatter.

- Danh sách sắp xếp theo thời gian sửa mới nhất; hiển thị tiêu đề (dòng `# ` đầu tiên), ngày sửa, tên file.
- Tạo mới: **bắt buộc có dòng `# Heading`** — heading được chuyển thành slug (tên file). Thiếu heading → lỗi "Add a `# Heading`; it becomes the slug."
- Chi tiết: **Delete**, **Save**.

### 3.5 Workflows

File: `~/.claude/workflows/*.js` — script điều phối nhiều subagent của Claude Code (chạy bằng lệnh `/<tên>` trong CLI).

- Danh sách: `/<meta.name>`, ngày, mô tả, các phase — đọc từ khối `export const meta = {...}` trong file.
- Tạo mới: editor JavaScript với template mẫu (`meta`, `agent(...)`, `pipeline(...)`). Yêu cầu:
  1. Có `export const meta = {`.
  2. Có `name:` là chuỗi trong dấu nháy — thành tên file và tên lệnh `/`.
- Chi tiết: mô tả + phase chips phía trên editor; **Delete**, **Save**.
- App **chỉ soạn thảo**, không chạy workflow. Chạy trong Claude Code.

> Mẹo: trong màn `/workflows` của Claude Code CLI, nhấn `s` để lưu script của một lần chạy vào thư mục này.

### 3.6 Output styles

Một trang duy nhất. Nguồn:
- **Built-in** (Default, Explanatory, Learning) — cố định, không sửa/xoá được.
- **Global**: `~/.claude/output-styles/<id>.md`.
- **Project**: `<working dir>/.claude/output-styles/<id>.md`.

Mỗi thẻ hiển thị tên, badge scope, mô tả và nội dung (chỉ đọc). Style không phải built-in có link **Delete**.

**+ New style** mở form:

| Trường | Ghi chú |
|---|---|
| ID (bắt buộc) | Thành tên file; 1–64 ký tự `^[a-z0-9_]+(?:-[a-z0-9_]+)*$` |
| Scope | `global` (mặc định) hoặc `project` |
| Working dir | Bắt buộc khi scope `project` |
| Name | Tuỳ chọn |
| Keep coding instructions | Giữ phần hướng dẫn coding của style gốc |
| Description | Tuỳ chọn |
| Body | Nội dung markdown |

Không có chức năng sửa — muốn sửa thì xoá rồi tạo lại, hoặc sửa file bằng editor ngoài.

---

## 4. Extend

### 4.1 MCP servers

**Scope** (chọn ở đầu trang, lưu trên URL):

| Scope | File |
|---|---|
| User | `~/.claude.json` (giống `claude mcp add --scope user`) |
| Project | `<working dir>/.mcp.json` — cần chọn project trong dropdown trước |

**Danh sách**: tên, transport (`stdio` / `httpSse`), command hoặc URL.

**Thêm server (+ New)**:
- **Name** (bắt buộc): 1–64 ký tự, bắt đầu bằng chữ/số, chỉ `. - _`.
- **Transport**:
  - `stdio`: Command (bắt buộc), Args (cách nhau bởi dấu cách), Env (mỗi dòng `KEY=VALUE`).
  - `HTTP/SSE`: URL (bắt buộc, hợp lệ), Headers (mỗi dòng `KEY=VALUE`).
- Dòng env/header thiếu `=` bị bỏ qua. Trùng tên → lỗi.
- Các key khác trong file JSON được giữ nguyên khi ghi.

**Chi tiết server**:
- **Configuration**: cấu hình dạng JSON (chỉ đọc).
- **Probe capabilities**: kết nối thật tới server (stdio thì khởi chạy process), bắt tay MCP và liệt kê **Tools / Resources / Prompts**. Kết quả cache 60 giây.
  - Server khai báo `claude/channel` → hộp tím cảnh báo server có thể đẩy sự kiện vào session (và trả lời permission prompt từ bên ngoài nếu có).
  - HTTP trả 400/404/405 → tự thử lại bằng giao thức SSE cũ.
  - HTTP **401/403** → "server requires sign-in; authenticate it with /mcp in claude". App không giữ OAuth token — hãy đăng nhập qua `/mcp` trong CLI.
- **Delete**: xoá server khỏi file tương ứng.

### 4.2 Hooks (tổng quan)

Trang `/hooks` **chỉ đọc**: liệt kê hook từ managed settings, `~/.claude/settings.json` và `~/.claude/settings.local.json`, nhóm theo event (PreToolUse, PostToolUse, …). Mỗi hook hiển thị tên rút gọn, matcher, timeout, status message và command đầy đủ.

Để thêm/sửa/xoá hook, và để xem hook cấp project: **Settings → Hooks** (xem [7.4](#74-hooks)).

### 4.3 Plugins

**Tab Installed**
- Nhóm theo marketplace (không có marketplace → "(local)").
- Công tắc **bật/tắt** mỗi plugin → ghi `enabledPlugins.<id>` vào `~/.claude/settings.json`.
- Hiển thị tên, version, mô tả, số skill, ngày cài. Bấm để vào trang chi tiết.

**Tab Discover**
- **Marketplaces** (đọc `~/.claude/plugins/known_marketplaces.json`):
  - **Add Marketplace**: Name, Type (`github` hoặc `http` JSON manifest), URL. GitHub chấp nhận `git@github.com:o/n`, `https://github.com/o/n(.git)` hoặc `o/n`.
  - **Update**: chỉ cập nhật mốc `lastUpdated` (việc kéo dữ liệu mới do CLI đảm nhiệm).
  - **Remove**: gỡ marketplace và cache của nó; plugin đã cài không bị ảnh hưởng.
- **Available plugins**:
  - **Install** (khi plugin có URL `http(s)://` hoặc `git@`): git clone vào `~/.claude/plugins/<id>/`, có cửa sổ tiến trình (cloning → installing → done).
  - Plugin không có URL cài được → hiển thị lệnh `claude plugins install <id>` để chạy trong CLI.
  - Plugin đã cài → nút **Installed** bị vô hiệu.

**Trang chi tiết plugin**: mô tả, skills, trạng thái, đường dẫn, README; nút **Enable/Disable** và **Uninstall** (xoá thư mục plugin, entry trong `installed_plugins.json` và cờ `enabledPlugins`).

---

## 5. Activity

### 5.1 Sessions

#### Danh sách project

Mỗi dòng là một thư mục trong `~/.claude/projects/`: tên, đường dẫn working dir, số session, lần hoạt động cuối.

- **+ Add project**: chọn thư mục → tạo `~/.claude/projects/<đường-dẫn-mã-hoá>/` (ví dụ `/Users/foo/app` → `-Users-foo-app`).
- **Rename** (bút chì) / **Delete** (thùng rác): chỉ tác động thư mục trong `~/.claude/projects/`, **không đụng tới mã nguồn** của bạn. Delete xoá vĩnh viễn toàn bộ transcript của project.

#### Trang project

Cột trái:
- Header: **Reveal in file manager**, **Worktrees**, **Project settings**, **Refresh sessions**.
- Thanh git: branch hiện tại, số commit ahead ↑ / behind ↓, `clean` hoặc `N changed`.
- Danh sách session: tiêu đề (custom title → tiêu đề AI của CLI → tin nhắn đầu → ID), số message, thời gian. Rê chuột để:
  - **Rename** inline (Enter lưu, Esc huỷ) — ghi giống lệnh `/rename` của CLI.
  - **Delete** — xoá transcript cùng transcript subagent (không hoàn tác được).
- Trong lúc trang mở, app theo dõi working dir: project settings, `.mcp.json`, `CLAUDE.md`, git status tự làm mới.

#### Xem session

- Header: tiêu đề, số message, số subagent.
- **Subagent messages**: bật để hiển thị lượt của subagent ngay trong transcript chính.
- **Transcript**:
  - Mỗi lượt: người nói (You / Claude / System), thời gian, model · tokens · cached · chi phí $.
  - **Thinking** thu gọn, bấm để mở.
  - **Tool use**: tên tool + tham số chính; mở rộng để xem toàn bộ input JSON.
  - **Tool result**: thu gọn, hiển thị dòng đầu, số dòng, kích thước; kết quả quá lớn bị cắt.
  - **Subagent**: thẻ tím, mở ra để tải transcript của subagent.
  - Danh sách tự tải thêm khi cuộn gần cuối.
- **Team panel**: hiện khi session là lead của một agent team (`~/.claude/teams/*/config.json`) — tên team, thành viên, agent type, tmux pane.

**Checkpoints** (nút *Checkpoints*): danh sách file Claude đã sửa trong session, nhóm theo checkpoint.
- Bấm một file để xem **diff** giữa bản đã lưu (−) và file hiện tại trên đĩa (+).
- **Restore**: ghi đè file hiện tại bằng bản đã lưu (hỏi xác nhận; nội dung hiện tại **không** được giữ lại). Restore từng file một.
- "(blob missing)" nghĩa là bản sao đã mất, không khôi phục được.

**Resume in terminal**:
1. Chọn **Permission mode** (mặc định lấy từ `defaultPermissionMode`, hoặc "CLI default").
2. Bấm **Resume in terminal** → app chạy `claude --resume <sessionId>` trong working dir của project, trong terminal nhúng (xterm, link bấm được).
3. Bên cạnh là **Context panel**: model, tokens input/output/cached, chi phí, danh sách tool call gần nhất.
4. **Close terminal** hoặc rời trang sẽ kết thúc process.

Khi terminal kết thúc, 10.000 dòng output cuối được lưu vào `~/.claude/cli-history/<id>.json`, xem lại qua **Terminal replay**.

**Terminal replay** (nút xuất hiện nếu session từng chạy trong app): phát lại output có màu, chỉ đọc. **Plain text** để chuyển sang text thuần có thể copy.

#### Project settings

- **Rename / Delete** project.
- **Project settings.json**: Default model và Default permission mode (hoặc *inherit*). **Save settings** chỉ ghi 2 key này.
- **CLAUDE.md**: editor cho `<working dir>/CLAUDE.md`, nút **Save CLAUDE.md**.

#### Worktrees (chỉ đọc)

- Liệt kê git worktree: tên, badge (main checkout / current / locked / prunable), branch hoặc HEAD, đường dẫn.
- Hiển thị nội dung `.worktreeinclude` và danh sách file khớp.
- App không tạo hay xoá worktree.

### 5.2 Jobs (chỉ đọc)

Background job mà CLI chạy, đọc từ `~/.claude/jobs/`.

- Danh sách: ★ nếu được ghim, tên, trạng thái, thời gian, working dir.
- Chi tiết: State, Created, Updated, Tokens, Working directory, Prompt, "Waiting on", Artifacts, Output, link session, Timeline sự kiện. Biến môi trường provider chỉ hiện **tên key**, không bao giờ hiện giá trị.

### 5.3 Usage

Thống kê token và chi phí ước tính từ toàn bộ transcript.

- **Khoảng thời gian**: 7 / 30 (mặc định) / 90 ngày / All time.
- **Nhóm theo**: ngày, project, model. Nút **Rescan** quét lại.
- Hiển thị: ô Cost, Turns, Input/Output tokens, Cache read/write; biểu đồ "Cost by …" + bảng chi tiết; biểu đồ "Prompts per day".

Cách tính:
- Quét `~/.claude/projects/*/*.jsonl` và transcript subagent, đọc `message.usage` của từng phản hồi.
- **Mỗi API response chỉ đếm một lần** (theo `message.id` + `requestId`) — tránh đếm trùng khi CLI ghi nhiều dòng cho một phản hồi hoặc khi resume session.
- Ngày tính theo **múi giờ máy**.
- Chi phí = token × giá API công khai hiện tại (standard tier) — **là ước tính, không phải hoá đơn**. Model không có giá → cảnh báo và không tính vào cost.
- "Prompts per day" đếm từ `~/.claude/history.jsonl` (chỉ đếm, không đọc nội dung).
- Kết quả được cache trong thư mục cache của app; file chỉ đọc lại khi thay đổi.

---

## 6. Config

### 6.1 Memory

Quản lý các file `CLAUDE.md` và rules.

- **Scope**: "User scope only" hoặc chọn một project.
- File hiển thị:
  - User: `~/.claude/CLAUDE.md`, `~/.claude/rules/**/*.md`.
  - Project: `<project>/CLAUDE.md`, `<project>/CLAUDE.local.md`, `<project>/.claude/rules/**/*.md`.
- Mỗi dòng: tiêu đề/đường dẫn, loại (CLAUDE.md / CLAUDE.local.md / rule), scope, số import, "not created" nếu file chưa tồn tại.
- **Agent memory**: file agent tự ghi — chỉ đọc.

**Trang sửa**:
- Editor markdown; **Save** tạo file nếu chưa có.
- **Imports sidebar**: danh sách import `@path`, đánh dấu **missing** (thiếu file) hoặc **cycle** (vòng lặp), độ sâu tối đa 5.
- **Preview flattened**: xem nội dung sau khi gộp mọi import (cắt ở 1 MB).
- Nếu file đã bị sửa bên ngoài kể từ lúc mở → Save bị từ chối, cần reload.

### 6.2 .claude (Claude directory)

Trình duyệt **chỉ đọc** cây `~/.claude` (và `.claude` của project nếu chọn project).

- Mỗi dòng: icon, mô tả một dòng, kích thước/số item, "not present" nếu chưa có, badge **undocumented** cho mục không nằm trong tài liệu chính thức.
- Bấm file → panel xem trước nội dung, link **Docs**, và nút **Edit** chuyển tới trình soạn phù hợp:

| File | Edit mở |
|---|---|
| `settings.json`, `settings.local.json` | Settings → Raw JSON |
| `keybindings.json` | Settings → Keybindings |
| `CLAUDE.md`, `CLAUDE.local.md`, `rules/*` | Memory |
| `.mcp.json` | MCP |
| `.worktreeinclude` | Worktrees của project |

---

## 7. Settings

### Scope chung cho mọi tab

Chọn **scope** và **project** ở đầu trang (giữ nguyên khi chuyển tab). Đường dẫn file hiện bên dưới, kèm "not created yet" nếu chưa có.

| Scope | File |
|---|---|
| User | `~/.claude/settings.json` |
| Project | `<wd>/.claude/settings.json` |
| Project local | `<wd>/.claude/settings.local.json` |
| Managed (chỉ đọc) | macOS `/Library/Application Support/ClaudeCode/managed-settings.json`; Linux `/etc/claude-code/managed-settings.json`; Windows `C:\Program Files\ClaudeCode\managed-settings.json` |

Các tab có form chỉ ghi **những key mà tab đó quản lý** (merge patch); mọi key khác trong file được giữ nguyên.

### 7.1 General

- **Claude CLI**: đường dẫn và version của `claude` (hoặc "not found").
- **Session defaults**: Default model, Default permission mode → **Save** (chọn *inherit* để xoá key).
- **Replay onboarding**.
- **App preferences** (lưu bởi app, không phải Claude Code): Theme, Claude directory override, Updater channel (stable/beta) → **Save preferences**.
- **Check for updates** → nếu có bản mới: **Install <version> and restart** kèm release notes.

### 7.2 Permissions

- 4 danh sách: **Allow** (vd. `Bash(npm run test:*)`), **Ask**, **Deny** (vd. `Read(./.env)`), **Additional directories** (vd. `../shared-lib`). Thêm bằng nút hoặc Enter.
- **Default mode**: inherit / default / acceptEdits / plan / auto / dontAsk / bypassPermissions.
- Kiểm tra trực tiếp:
  - Lỗi (đỏ): ngoặc không cân, specifier rỗng, thiếu/sai tên tool → chặn lưu.
  - Cảnh báo (vàng): rule trùng, rule vừa allow vừa deny (deny thắng), domain không viết dạng `domain:…`.
- **In force everywhere**: rule đã gộp từ mọi scope kèm badge nguồn, và mode đang có hiệu lực.

### 7.3 Status line

- **Command** (vd. `~/bin/claude-statusline`) và **Padding** → lưu vào key `statusLine`. **Remove** để xoá.
- **Test run**: chạy thật command với JSON giống Claude Code gửi vào stdin (tối đa 5 giây); hiển thị output, exit code, thời gian, stderr.

### 7.4 Hooks

- Liệt kê hook từ mọi scope kèm badge scope; **Edit** / **Delete** từng nhóm.
- **+ New hook**:
  - **Event**: gợi ý PreToolUse, PostToolUse, UserPromptSubmit, SessionStart, SessionEnd, Notification, Stop, SubagentStop, PreCompact.
  - **Matcher**: regex (để trống = khớp tất cả), vd. `Bash|Edit`.
  - Một hoặc nhiều **command**, mỗi cái có Timeout (1–600 giây) và Status message.
- Trước khi lưu, app hiển thị **xác nhận nguy hiểm** liệt kê chính xác các command sẽ được Claude Code chạy.
- App không tự chạy hook.

### 7.5 Keybindings

Luôn là `~/.claude/keybindings.json` (không phụ thuộc scope).

- Chưa có file → nút **Create keybindings.json**.
- Bảng Context / Keys / Action; để trống Action để bỏ gán phím mặc định.
- Thêm phím: bấm ô "Press a shortcut…", nhấn tổ hợp phím, rồi **Add binding**. Phím không hợp lệ hoặc dành riêng (Ctrl+C, Ctrl+D, Ctrl+M, Ctrl+I, Ctrl+H) báo lỗi ngay.

### 7.6 Effective

Bảng Key / Value / From cho mọi key ở mọi scope, theo thứ tự ưu tiên. Badge xanh = scope thắng; badge xám gạch ngang = scope bị ghi đè. `permissions` được gộp từ nhiều file (nhãn "merged").

### 7.7 Raw JSON

- Editor JSON cho file của scope đang chọn; trạng thái **Valid JSON** hoặc lỗi parse; nút **Format**.
- Khi lưu: phải là JSON object hợp lệ, file không bị sửa bên ngoài kể từ lúc mở; app tạo bản sao lưu `<file>.ccg.bak` trước khi ghi.

---

## 8. Deep links

Scheme `claude-code-gui://` (hoạt động cả khi app đang chạy hoặc chưa mở):

| Link | Tác dụng |
|---|---|
| `claude-code-gui://open-agent/<slug>` | Mở agent |
| `claude-code-gui://install/<plugin>?source=<src>` | Mở Plugins → Discover và bắt đầu cài plugin |

Link khác bị bỏ qua kèm cảnh báo.

---

## 9. Phím tắt

| Phím | Ở đâu | Tác dụng |
|---|---|---|
| ⌘K / Ctrl+K | Mọi nơi trừ trong terminal | Mở/đóng tìm kiếm |
| ↑ / ↓ / Enter / Esc | Hộp tìm kiếm | Di chuyển / mở / đóng |
| Enter / Esc | Đổi tên session inline | Lưu / huỷ |
| Esc | Hộp thoại xác nhận | Huỷ |

Mọi phím gõ trong terminal nhúng được gửi thẳng tới Claude. Phím tắt của **Claude CLI** chỉnh ở **Settings → Keybindings**.

---

## 10. Bảng file app đọc/ghi

| Đường dẫn | Chức năng |
|---|---|
| `~/.claude/agents/`, `commands/`, `skills/`, `plans/`, `workflows/`, `output-styles/` | Authoring (đọc/ghi) |
| `~/.claude.json`, `<wd>/.mcp.json` | MCP (đọc/ghi) |
| `~/.claude/settings.json`, `settings.local.json`, `<wd>/.claude/settings*.json` | Settings, Hooks, Plugins enable (đọc/ghi) |
| Managed settings | Chỉ đọc |
| `~/.claude/keybindings.json` | Keybindings (đọc/ghi) |
| `~/.claude/plugins/` | Plugins (cài/gỡ) |
| `~/.claude/CLAUDE.md`, `rules/`, `<wd>/CLAUDE*.md`, `<wd>/.claude/rules/` | Memory (đọc/ghi) |
| `~/.claude/projects/` | Sessions (đọc; đổi tên, xoá) |
| `~/.claude/file-history/` | Checkpoints (đọc; restore ghi vào file project) |
| `~/.claude/cli-history/` | Terminal replay (app ghi) |
| `~/.claude/jobs/`, `teams/`, `history.jsonl` | Chỉ đọc |

---

## 11. Hạn chế hiện tại

- **New Chat** trên trang project chưa hoạt động — terminal nhúng hiện chỉ mở qua **Resume in terminal**, và chỉ chọn được permission mode.
- **MCP**: chưa có trang sửa server, chưa bật/tắt từng server, chưa có scope `local`. Muốn sửa: xoá và thêm lại, hoặc dùng Raw editor / CLI.
- **Output styles**: không sửa được; style scope `project` tạo được nhưng chưa hiển thị/xoá được trên trang.
- **Memory**: chưa có nút xoá file.
- **Hooks** (`/hooks`): chỉ hiện hook cấp user/managed; hook cấp project xem ở Settings → Hooks.
- **Settings → General**: ô Session defaults luôn hiển thị giá trị của scope User, bất kể scope đang chọn.
- **Checkpoints**: khôi phục từng file, chưa có "rewind" cả session.
- **Context panel** trong terminal có thể trống nếu output của Claude không in dòng usage/tool theo định dạng app nhận diện.
