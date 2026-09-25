# Handoff: Claude Code GUI — Desktop UI

## Tổng quan
Mockup cho **Claude Code GUI**, một app desktop để quản lý trực quan nội dung trong `~/.claude` (agents, commands, skills, MCP, sessions, usage, settings…). Spec chức năng đầy đủ nằm trong tài liệu "Hướng dẫn sử dụng Claude Code GUI". README này chỉ mô tả **giao diện**.

## Về file thiết kế
`Claude Code GUI.dc.html` là **bản tham chiếu thiết kế viết bằng HTML**: prototype thể hiện giao diện và hành vi mong muốn, **không phải code production**. Nhiệm vụ của bạn là dựng lại giao diện trong stack của app (ví dụ Tauri/Electron + React), theo pattern và thư viện sẵn có. Nếu chưa có stack, gợi ý dùng Tauri + React + CodeMirror 6 + xterm.js.

Mở file trực tiếp trong trình duyệt để xem. Nav ở sidebar bấm được, ⌘K mở palette tìm kiếm.

## Độ trung thực
**Hi-fi** cho màu, typography, spacing, radius và layout. Riêng biểu đồ Relationships và biểu đồ Usage là minh hoạ: hãy dùng thư viện chart/graph thật nhưng giữ đúng style.

---

## Khung chung (App shell)
- Cửa sổ 1280×800 (kích thước tham chiếu; bản thật thì fluid). Nền `#FBFAF7`.
- **Title bar** cao 38px, nền `#EFECE6`, viền dưới `1px #E1DDD4`. Traffic lights 12px (`#EC6A5E` `#F4BF4F` `#61C554`), gap 8px. Tiêu đề 12px/500 `#6B6860` ở giữa. Bên phải: đường dẫn Claude dir, mono 11px `#8A867C`.
- **Sidebar** rộng 220px, nền `#F3F1EC`, viền phải `1px #E1DDD4`, padding 12px 10px, gap giữa các nhóm 14px.
  - Nút Search: cao 32px, nền trắng, viền `#DEDAD1`, radius 7px, chữ 13px `#8A867C`, badge "⌘K" mono 11px `#A29E94`.
  - Nhãn nhóm (Authoring / Extend / Activity / Config): 10.5px/600, uppercase, letter-spacing .08em, `#A29E94`, padding `0 10px 4px`.
  - Nav item: cao 28px, padding 0 10px, radius 6px, 13px. Count bên phải: mono 11px `#A29E94`. Hover: `#E8E5DE`. Active: nền `#E4E0D8`, weight 600.
  - Cuối sidebar: Settings (cùng style nav item), rồi dòng trạng thái CLI (viền trên `#E1DDD4`, chấm 7px `#4E9A5B` cùng "claude 2.4.1" mono 11px). Nếu không tìm thấy CLI thì chấm đổi sang đỏ `#B03A2E`.
- **Header trang**: padding `20px 28px 16px`, viền dưới `1px #ECE9E2`. Tiêu đề 20px/600, letter-spacing -.01em. Đường dẫn file dưới tiêu đề: mono 12px `#8A867C`.

## Các màn hình

### 1. Danh sách Agents
- Header: tiêu đề "Agents", phụ đề `~/.claude/agents/**/*.md`. Bên phải: ô Filter… (220×32), nút **Import** (secondary), **+ New** (primary).
- Lưới 3 cột `repeat(3, minmax(0,1fr))`, gap 12px, padding 20px 28px.
- **Card**: nền trắng, viền `#E6E2DA`, radius 9px, padding 14px 16px, gap 10px. Hover: viền `#C9C3B7`, shadow `0 2px 8px rgba(40,30,20,.06)`.
  - Dòng 1: chấm màu 9px (lấy từ `color:` trong frontmatter), tên (mono 13.5px/500, 1 dòng, cắt bằng ellipsis), badge model (mono 10.5px/500, nền `#F1EEE8`, chữ `#6B6860`, radius 4px, padding 2px 6px).
  - Mô tả: 13px/1.45 `#5E5B54`, min-height 38px.
  - 3 tool đầu tiên: chip mono 11px, viền `#ECE9E2`, chữ `#8A867C`, radius 4px.
- Bấm card để mở trang chi tiết.

### 2. Chi tiết Agent (editor)
- Toolbar: padding 14px 20px. Breadcrumb "Agents / " (13px `#8A867C`, bấm được), chấm màu, slug (mono 14px/500), chỉ báo **Unsaved** (12px `#B4502B` kèm chấm 6px, chỉ hiện khi có thay đổi). Bên phải: **Relationships (N)** (toggle, nền `#EFECE6` khi bật), **Export**, **Delete** (viền `#EBCFC3`, chữ `#B03A2E`), **Save** (primary, disabled khi không có thay đổi).
- Editor (CodeMirror 6): nền trắng, mono 13px/22px. Gutter rộng 48px, nền `#FAF9F6`, số dòng `#C2BDB2`, viền phải `#F0EDE7`. Syntax colors:
  - Key YAML `#B4502B`, string `#3F7A4E`, dấu `---` `#A29E94`
  - Heading markdown `#2F5E9E`/500, inline code `#7A4FA0`
  - Dòng hiện tại: nền `#FFF6E8`
- Panel Relationships: rộng 320px, nền `#F7F5F0`, viền trái `#ECE9E2`. Đồ thị có node trung tâm (nền `#1F1E1B`, chữ trắng, radius 8px). Node skill có viền `#D9E4DB`, chữ `#3F7A4E`; node command có viền `#E3D7EC`, chữ `#7A4FA0`. Cạnh là đường cong `#D6D0C4` 1.5px. Có legend, và một hộp note về việc slug cố định.
- Status bar: cao 26px, nền `#F7F5F0`, mono 11px `#8A867C`. Hiện trạng thái YAML (hợp lệ: `#3F7A4E`; lỗi: `#B03A2E`), Ln/Col và đường dẫn file.

### 3. Sessions: xem một session
- Cột trái rộng 260px, nền `#F9F8F4`: tên project (15px/600) và working dir (mono 11px). Thanh git gồm branch, ↑/↓ và "N changed" (màu `#B4502B`; khi clean thì dùng `#3F7A4E`). Danh sách session: item padding 9px 10px, radius 7px, tiêu đề 13px/500 (1 dòng), meta 11.5px `#8A867C`. Active: `#E9E5DD`. Hover: `#EFECE5`, hiện icon rename/delete.
- Header: tiêu đề và "N messages · N subagents · $cost". Toggle **Subagent messages** (28×16). Nút **Checkpoints**. Nhóm nút ghép: dropdown permission mode (mono 11.5px) + **Resume in terminal** (primary), viền `#1F1E1B`.
- Transcript: max-width 860px, gap 18px.
  - Dòng meta mỗi lượt: 12px `#8A867C`. Tên người nói đậm: You `#1F1E1B`, Claude `#B4502B`. Model · tokens · cached · $ viết bằng mono.
  - Thinking: 12.5px italic, thu gọn (▸).
  - Thẻ tool: viền `#E6E2DA`, radius 8px, header mono 12px. Khi mở rộng, phần kết quả có nền `#FAF9F6`, dòng cuối là "… N lines · size" màu `#A29E94`. Diff stats `+N` `#3F7A4E`, `−N` `#B03A2E`.
  - Thẻ subagent: nền `#FAF6FC`, viền `#E3D7EC`, badge "subagent" nền `#7A4FA0` chữ trắng.

### 4. MCP servers
- Header: segmented control **User / Project**, nút **+ New**. Segmented: track `#EFECE6` padding 3px radius 8px; item đang chọn nền trắng, radius 6px, shadow `0 1px 2px rgba(0,0,0,.08)`.
- Cột danh sách rộng 300px. Item gồm tên (mono 13px/500), badge transport, command/URL (mono 11.5px, ellipsis). Item đang chọn: nền trắng, viền `#DEDAD1`.
- Chi tiết: tên (mono 17px/600), nút **↻ Probe capabilities**, **Delete**.
  - Configuration: khối JSON chỉ đọc (mono 12.5px/1.65, nền trắng, viền `#E6E2DA`).
  - Cảnh báo `claude/channel`: nền `#F6F0FA`, viền `#E3D7EC`, chữ `#5A3A78`.
  - Tabs Tools / Resources / Prompts kèm số lượng. Tab active có gạch dưới 2px `#1F1E1B`. Bên phải là "probed Ns ago · cached 60s".
  - Dòng tool: grid `200px 1fr`, tên mono 12.5px, mô tả `#6B6860`.
  - Lỗi 401/403: dùng hộp cảnh báo cùng style như ở Permissions (vàng), nội dung "server requires sign-in; authenticate it with /mcp in claude".

### 5. Usage
- Header: segmented khoảng thời gian **7d / 30d / 90d / All**, segmented nhóm theo **Day / Project / Model**, nút **Rescan**.
- Dải KPI: grid 5 cột trong 1 card (radius 10px), giữa các ô có vạch ngăn `#F0EDE7`. Label 12px `#8A867C`, số 24px/600, letter-spacing -.02em. Các ô: Est. cost, Turns, Input / Output, Cache read, Cache write.
- Biểu đồ "Cost by day": cột chồng theo model (opus `#C2552D`, sonnet `#E7B9A3`), cao 170px, gap 5px. Ghi chú bắt buộc: "Estimate at public API prices · not a bill".
- Bảng: grid `2fr repeat(4,1fr)`. Header nền `#FAF9F6`, chữ 11.5px `#8A867C`. Số căn phải, dùng `tabular-nums`.

### 6. Settings → Permissions
- Header: segmented scope **User / Project / Project local / Managed** (Managed chữ `#A29E94`, chỉ đọc), dropdown project, thanh tab (General, Permissions, Status line, Hooks, Keybindings, Effective, Raw JSON) có gạch dưới 2px ở tab active. Đường dẫn file mono ở bên phải.
- Nội dung: grid `1fr 320px`, gap 24px.
  - 4 danh sách rule (Allow / Ask / Deny / Additional directories). Mỗi danh sách là 1 card, rule là dòng mono 12.5px có nút × ở cuối, dòng cuối là input thêm rule (Enter để thêm).
  - Cảnh báo inline (vàng): nền `#FDF3DC`, chữ `#9A6A12`. Lỗi (đỏ, chặn lưu): chữ `#B03A2E`, nền `#FBE9E6`.
  - Panel **In force everywhere**: nền `#F3F1EC`, radius 10px. Mỗi dòng gồm loại (allow `#3F7A4E` / ask `#9A6A12` / deny `#B03A2E`), rule, badge nguồn.

### 7. Command palette (⌘K)
- Overlay `rgba(31,30,27,.18)` phủ dưới title bar. Hộp rộng 600px, cách trên 90px, radius 12px, shadow `0 24px 60px rgba(40,30,20,.3)` cộng viền `0 0 0 1px rgba(0,0,0,.08)`.
- Input 16px. Kết quả gồm cột loại (mono 10.5px uppercase, rộng 64px), tên (mono 13px/500), mô tả (ellipsis). Kết quả đang chọn: nền `#F3F1EC`.
- Footer: "↑↓ move · ↵ open" và phạm vi tìm "Agents · Commands · Skills · Plans".

## Tương tác & hành vi
- Nav ở sidebar đổi route. Mỗi route cần deep-linkable (scope MCP và Settings lưu trên URL).
- ⌘K / Ctrl+K bật/tắt palette; Esc đóng; bấm ra ngoài overlay cũng đóng. Không bắt phím tắt khi focus đang ở terminal nhúng.
- Nút Save disabled khi không có thay đổi. Rời trang khi còn thay đổi chưa lưu thì hỏi "Discard unsaved changes?".
- Delete luôn có hộp xác nhận (xoá vĩnh viễn).
- Hover: card/item đổi nền hoặc viền như mô tả ở trên. Transition gợi ý: 120ms ease-out cho background, border-color, box-shadow.
- Loading: dùng skeleton cùng kích thước card/dòng, nền `#F1EEE8`. Probe MCP: nút hiện spinner, disabled.
- Empty state: 1 dòng mô tả (13px `#8A867C`) kèm nút primary tương ứng (ví dụ "+ New").

## State cần có
- `route` (screen + params), `paletteOpen`, `paletteQuery`, `paletteIndex`
- Editor: `content`, `savedContent` (để suy ra `dirty`), `yamlError`, `showRelationships`
- Sessions: `selectedProject`, `selectedSession`, `showSubagentMessages`, `permissionMode`
- MCP: `scope`, `selectedServer`, `probeResult` (cache 60s), `probeState`
- Usage: `range`, `groupBy`, dữ liệu tổng hợp (cache)
- Settings: `scope`, `project`, `activeTab`, danh sách rule cùng kết quả validate
- Dữ liệu lấy từ filesystem, có file watcher để tự refresh (xem tài liệu chức năng)

## Design tokens

**Màu**
| Token | Hex | Dùng cho |
|---|---|---|
| bg-app | `#FBFAF7` | nền nội dung |
| bg-sidebar | `#F3F1EC` | sidebar, panel phụ |
| bg-chrome | `#EFECE6` | title bar, track segmented |
| bg-subtle | `#FAF9F6` / `#F7F5F0` / `#F9F8F4` | gutter, status bar, cột phụ |
| surface | `#FFFFFF` | card, input |
| hover | `#E8E5DE` | nav hover |
| active | `#E4E0D8` | nav active |
| border | `#E6E2DA` | viền card |
| border-strong | `#DEDAD1` | viền input/nút |
| divider | `#ECE9E2` / `#F0EDE7` / `#F3F0EA` | vạch ngăn |
| ink | `#1F1E1B` | chữ chính, nút primary |
| ink-2 | `#5E5B54` | mô tả |
| muted | `#6B6860` | chữ phụ |
| subtle | `#8A867C` | meta |
| faint | `#A29E94` | placeholder, nhãn nhóm |
| accent | `#B4502B` (chữ) / `#C2552D` (fill) | Claude, unsaved, key YAML |
| success | `#3F7A4E` / nền `#DDEBDF` | valid, allow, skill |
| warning | `#9A6A12` / nền `#FDF3DC` | cảnh báo |
| danger | `#B03A2E` / viền `#EBCFC3` | delete, deny |
| purple | `#7A4FA0` / nền `#FAF6FC`, `#F6F0FA` / viền `#E3D7EC` | subagent, command, channel |
| blue | `#2F5E9E` | heading markdown |

**Typography**: Sans **IBM Plex Sans** (400/500/600); Mono **JetBrains Mono** (400/500), dùng cho mọi slug, đường dẫn, rule, code, token/cost.
Scale: 10.5 / 11 / 11.5 / 12 / 12.5 / 13 (body) / 13.5 / 14 / 15 / 17 / 20 (tiêu đề trang) / 24 (KPI).

**Spacing**: 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 28 (padding ngang của trang).

**Radius**: 3–4 (badge), 6 (nav item), 7 (nút/input), 8 (thẻ tool, khối code), 9 (card), 10 (card lớn), 12 (cửa sổ, palette).

**Kích thước control**: nút/input 30–32px, nav item 28px, status bar 26px, title bar 38px.

**Shadow**: card hover `0 2px 8px rgba(40,30,20,.06)`; segmented item `0 1px 2px rgba(0,0,0,.08)`; palette `0 24px 60px rgba(40,30,20,.3)`.

**Nút**
- Primary: nền `#1F1E1B`, chữ trắng, 13px/500, radius 7px, padding 0 14px
- Secondary: nền trắng, viền `#DEDAD1`
- Danger: nền trắng, viền `#EBCFC3`, chữ `#B03A2E`

## Assets
Không có ảnh hay icon riêng. Font lấy từ Google Fonts. Chỗ cần icon (sidebar, reveal, rename, delete) có thể dùng bộ icon line 16px, stroke 1.5, ví dụ Lucide. Mockup hiện chưa đặt icon.

## Files
- `Claude Code GUI.dc.html`: prototype chứa toàn bộ màn hình (mở bằng trình duyệt). Toàn bộ dữ liệu mẫu nằm trong class `Component` → `renderVals()`.

## Chưa được thiết kế
Commands, Skills, Plans, Workflows, Output styles, Hooks, Plugins, Jobs, Memory, .claude, Onboarding, các tab Settings còn lại, terminal nhúng, Checkpoints diff. Hãy dùng lại pattern có sẵn: danh sách dùng card như Agents, editor như chi tiết Agent, form như Settings.
