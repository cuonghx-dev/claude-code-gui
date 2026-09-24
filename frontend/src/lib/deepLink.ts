import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrent } from '@tauri-apps/plugin-deep-link'
import type { Router } from 'vue-router'
import { toast } from 'vue-sonner'

/**
 * Deep-link handler. Two ingress paths:
 *
 * 1. `app:single_instance` — fired by `tauri-plugin-single-instance` when a
 *    second invocation runs (`claude-code-gui://...` from Finder /
 *    Spotlight goes here on macOS once the app is registered).
 * 2. `deep-link://new-url` — emitted by `tauri-plugin-deep-link` when the
 *    OS routes a registered scheme into the running process.
 * 3. `getCurrent()` — the URL that launched the app from cold, which fired
 *    before any listener existed.
 *
 * The same URL can arrive on more than one path, so each is handled once
 * within a short window.
 *
 * Allow-list (locked from SPEC §8): only `install` and `open-agent` are
 * accepted. Anything else is logged + toasted and dropped.
 */

interface DeepLinkAction {
  verb: 'install' | 'open-agent'
  /** First path segment after the verb. */
  target: string
  /** URL search params. */
  params: URLSearchParams
}

const ALLOWED: ReadonlySet<DeepLinkAction['verb']> = new Set([
  'install',
  'open-agent',
])

function parse(rawUrl: string): DeepLinkAction | undefined {
  let url: URL
  try {
    url = new URL(rawUrl)
  } catch {
    return undefined
  }
  if (url.protocol !== 'claude-code-gui:') return undefined
  // `claude-code-gui://install/foo?source=bar`
  // → host = 'install', pathname = '/foo'
  const verb = url.host as DeepLinkAction['verb']
  if (!ALLOWED.has(verb)) return undefined
  const target = decodeURIComponent(url.pathname.replace(/^\/+/, ''))
  if (!target) return undefined
  return { verb, target, params: url.searchParams }
}

function dispatch(router: Router, action: DeepLinkAction) {
  switch (action.verb) {
    case 'open-agent':
      void router.push(`/agents/${encodeURIComponent(action.target)}`)
      return
    case 'install': {
      const source = action.params.get('source')
      if (!source) {
        toast.error('install link missing ?source=')
        return
      }
      const params = new URLSearchParams({
        autoInstall: action.target,
        source,
      })
      void router.push(`/plugins?${params.toString()}`)
      return
    }
  }
}

const DEDUPE_MS = 3000
const recent = new Map<string, number>()

/** True the first time `url` is seen within `DEDUPE_MS`. */
function firstSighting(url: string): boolean {
  const now = Date.now()
  for (const [u, t] of recent) if (now - t > DEDUPE_MS) recent.delete(u)
  if (recent.has(url)) return false
  recent.set(url, now)
  return true
}

function handle(router: Router, url: string) {
  if (!firstSighting(url)) return
  const action = parse(url)
  if (action) dispatch(router, action)
  else toast.warning(`Ignored deep link: ${url}`)
}

let unlistenSingle: UnlistenFn | undefined
let unlistenDeep: UnlistenFn | undefined

export async function attachDeepLinkListener(router: Router): Promise<() => void> {
  unlistenSingle = await listen<{ args: string[] }>('app:single_instance', (e) => {
    // args also carries the binary path and flags; only scheme URLs matter.
    for (const arg of e.payload.args) {
      if (arg.startsWith('claude-code-gui://')) handle(router, arg)
    }
  })

  // tauri-plugin-deep-link emits `deep-link://new-url` (Tauri 2 docs).
  unlistenDeep = await listen<string[]>('deep-link://new-url', (e) => {
    for (const url of e.payload) handle(router, url)
  })

  try {
    for (const url of (await getCurrent()) ?? []) handle(router, url)
  } catch {
    // Not launched by a link, or the plugin is unavailable (plain web preview).
  }

  return () => {
    unlistenSingle?.()
    unlistenDeep?.()
    unlistenSingle = undefined
    unlistenDeep = undefined
  }
}
