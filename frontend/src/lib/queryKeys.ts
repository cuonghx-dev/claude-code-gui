// Central query-key factory. No string-literal query keys anywhere else.
// Code review enforces. See implementation plan §C.

export const qk = {
  agents: {
    all: ['agents'] as const,
    list: () => ['agents', 'list'] as const,
    get: (slug: string) => ['agents', 'get', slug] as const,
    skills: (slug: string) => ['agents', 'skills', slug] as const,
    skillCounts: () => ['agents', 'skillCounts'] as const,
    history: (slug: string) => ['agents', 'history', slug] as const,
    historyEntry: (slug: string, id: string) => ['agents', 'history', slug, id] as const,
  },
  commands: {
    all: ['commands'] as const,
    list: () => ['commands', 'list'] as const,
    get: (slug: string) => ['commands', 'get', slug] as const,
  },
  skills: {
    all: ['skills'] as const,
    list: () => ['skills', 'list'] as const,
    get: (slug: string) => ['skills', 'get', slug] as const,
  },
  plans: {
    all: ['plans'] as const,
    list: () => ['plans', 'list'] as const,
    get: (slug: string) => ['plans', 'get', slug] as const,
  },
  workflows: {
    all: ['workflows'] as const,
    list: () => ['workflows', 'list'] as const,
    get: (slug: string) => ['workflows', 'get', slug] as const,
  },
  terminals: {
    all: ['terminals'] as const,
    list: () => ['terminals', 'list'] as const,
    get: (id: string) => ['terminals', 'get', id] as const,
  },
  checkpoints: {
    all: ['checkpoints'] as const,
    list: (sessionId: string) => ['checkpoints', 'list', sessionId] as const,
    diff: (sessionId: string, left: string, right: string) =>
      ['checkpoints', 'diff', sessionId, left, right] as const,
  },
  usage: {
    all: ['usage'] as const,
    rollup: (query: string) => ['usage', 'rollup', query] as const,
    activity: (days: number) => ['usage', 'activity', days] as const,
  },
  jobs: {
    all: ['jobs'] as const,
    list: () => ['jobs', 'list'] as const,
    get: (id: string) => ['jobs', 'get', id] as const,
  },
  teams: {
    all: ['teams'] as const,
    list: () => ['teams', 'list'] as const,
    get: (id: string) => ['teams', 'get', id] as const,
  },
  outputStyles: {
    all: ['outputStyles'] as const,
    list: () => ['outputStyles', 'list'] as const,
  },
  hooks: {
    all: ['hooks'] as const,
    list: (wd?: string) => ['hooks', 'list', wd ?? ''] as const,
    get: (id: string, wd?: string) => ['hooks', 'get', id, wd ?? ''] as const,
    raw: (scope: string, wd?: string) => ['hooks', 'raw', scope, wd ?? ''] as const,
  },
  plugins: {
    all: ['plugins'] as const,
    list: () => ['plugins', 'list'] as const,
    get: (id: string) => ['plugins', 'get', id] as const,
  },
  marketplace: {
    available: () => ['marketplace', 'available'] as const,
    sources: () => ['marketplace', 'sources'] as const,
  },
  mcp: {
    all: ['mcp'] as const,
    list: (scope: string, wd?: string) => ['mcp', 'list', scope, wd ?? ''] as const,
    get: (name: string, scope: string, wd?: string) =>
      ['mcp', 'get', name, scope, wd ?? ''] as const,
    capabilities: (name: string, scope: string, wd?: string) =>
      ['mcp', 'capabilities', name, scope, wd ?? ''] as const,
  },
  claudeDirectory: {
    all: ['claudeDirectory'] as const,
    tree: (projectPath?: string) => ['claudeDirectory', 'tree', projectPath ?? ''] as const,
    children: (path: string, projectPath?: string) =>
      ['claudeDirectory', 'children', path, projectPath ?? ''] as const,
  },
  projects: {
    all: ['projects'] as const,
    list: () => ['projects', 'list'] as const,
    get: (n: string) => ['projects', 'get', n] as const,
    files: (n: string, sub?: string) => ['projects', 'files', n, sub ?? ''] as const,
    gitStatus: (n: string) => ['projects', 'gitStatus', n] as const,
    settings: (n: string) => ['projects', 'settings', n] as const,
    claudeMd: (n: string) => ['projects', 'claudeMd', n] as const,
  },
  sessions: {
    all: ['sessions'] as const,
    listFor: (n: string) => ['sessions', 'listFor', n] as const,
    messages: (id: string) => ['sessions', 'messages', id] as const,
    threads: (id: string) => ['sessions', 'threads', id] as const,
    threadMessages: (id: string, threadId: string) =>
      ['sessions', 'threadMessages', id, threadId] as const,
  },
  settings: {
    all: ['settings'] as const,
    scopes: (wd?: string) => ['settings', 'scopes', wd ?? ''] as const,
    raw: (scope: string, wd?: string) => ['settings', 'raw', scope, wd ?? ''] as const,
    effective: (wd?: string) => ['settings', 'effective', wd ?? ''] as const,
    typed: () => ['settings', 'typed'] as const,
  },
  permissions: {
    all: ['permissions'] as const,
    get: (scope: string, wd?: string) => ['permissions', 'get', scope, wd ?? ''] as const,
    effective: (wd?: string) => ['permissions', 'effective', wd ?? ''] as const,
  },
  memory: {
    all: ['memory'] as const,
    list: (wd?: string) => ['memory', 'list', wd ?? ''] as const,
    get: (id: string, wd?: string) => ['memory', 'get', id, wd ?? ''] as const,
    preview: (id: string, wd?: string) => ['memory', 'preview', id, wd ?? ''] as const,
    agent: (wd?: string) => ['memory', 'agent', wd ?? ''] as const,
  },
  statusline: {
    all: ['statusline'] as const,
    get: (scope: string, wd?: string) => ['statusline', 'get', scope, wd ?? ''] as const,
  },
  keybindings: {
    all: ['keybindings'] as const,
    doc: () => ['keybindings', 'doc'] as const,
    raw: () => ['keybindings', 'raw'] as const,
  },
  config: () => ['config'] as const,
  debug: {
    claudeCli: () => ['debug', 'claudeCli'] as const,
  },
} as const
