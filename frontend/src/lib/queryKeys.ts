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
  outputStyles: {
    all: ['outputStyles'] as const,
    list: () => ['outputStyles', 'list'] as const,
  },
  hooks: {
    all: ['hooks'] as const,
    list: (wd?: string) => ['hooks', 'list', wd ?? ''] as const,
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
  },
  settings: () => ['settings'] as const,
  config: () => ['config'] as const,
  debug: {
    claudeCli: () => ['debug', 'claudeCli'] as const,
  },
} as const
