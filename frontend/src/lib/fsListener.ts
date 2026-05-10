import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { queryClient } from './queryClient'
import { qk } from './queryKeys'

interface FsChangePayload {
  path: string
  kind: 'create' | 'modify' | 'delete'
}

interface FsFloodPayload {
  subscriptionId: string
  root: string
  eventsPerSec: number
}

const RULES: Array<{ test: (path: string) => boolean; invalidate: () => void }> = [
  { test: (p) => p.includes('/.claude/agents/'),       invalidate: () => queryClient.invalidateQueries({ queryKey: qk.agents.all }) },
  { test: (p) => p.includes('/.claude/commands/'),     invalidate: () => queryClient.invalidateQueries({ queryKey: qk.commands.all }) },
  { test: (p) => p.includes('/.claude/skills/'),       invalidate: () => queryClient.invalidateQueries({ queryKey: qk.skills.all }) },
  { test: (p) => p.includes('/.claude/plans/'),        invalidate: () => queryClient.invalidateQueries({ queryKey: qk.plans.all }) },
  { test: (p) => p.includes('/.claude/output-styles/'),invalidate: () => queryClient.invalidateQueries({ queryKey: qk.outputStyles.all }) },
  { test: (p) => p.includes('/.claude/plugins/'),      invalidate: () => queryClient.invalidateQueries({ queryKey: qk.plugins.all }) },
  { test: (p) => p.endsWith('/.mcp.json'),             invalidate: () => queryClient.invalidateQueries({ queryKey: qk.mcp.all }) },
  { test: (p) => p.endsWith('/.claude/settings.json'), invalidate: () => queryClient.invalidateQueries({ queryKey: qk.settings() }) },
  {
    test: (p) => p.includes('/.claude/projects/'),
    invalidate: () => {
      queryClient.invalidateQueries({ queryKey: qk.projects.all })
      queryClient.invalidateQueries({ queryKey: qk.sessions.all })
    },
  },
]

let unlistenFsChange: UnlistenFn | undefined
let unlistenFsFlood: UnlistenFn | undefined
let unlistenClaudeDirChanged: UnlistenFn | undefined

/**
 * Single global `fs:change` listener. Component-level listeners are
 * forbidden (see implementation plan §C). Wired in App.vue::onMounted.
 */
export async function attachFsListener(): Promise<() => void> {
  unlistenFsChange = await listen<FsChangePayload>('fs:change', (e) => {
    for (const rule of RULES) {
      if (rule.test(e.payload.path)) rule.invalidate()
    }
  })

  unlistenFsFlood = await listen<FsFloodPayload>('fs:flood', (e) => {
    // Phase 3: surface as a non-blocking toast banner.
    console.warn('fs:flood', e.payload)
  })

  unlistenClaudeDirChanged = await listen<{ path: string }>('app:claude_dir_changed', () => {
    // Rare event; nuke everything and let consumers refetch.
    queryClient.invalidateQueries()
  })

  return detachFsListener
}

export function detachFsListener(): void {
  unlistenFsChange?.()
  unlistenFsFlood?.()
  unlistenClaudeDirChanged?.()
  unlistenFsChange = undefined
  unlistenFsFlood = undefined
  unlistenClaudeDirChanged = undefined
}
