import { listen, type UnlistenFn } from '@tauri-apps/api/event'
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

let unlistenSingle: UnlistenFn | undefined
let unlistenDeep: UnlistenFn | undefined

export async function attachDeepLinkListener(router: Router): Promise<() => void> {
  unlistenSingle = await listen<{ args: string[] }>('app:single_instance', (e) => {
    for (const arg of e.payload.args) {
      const action = parse(arg)
      if (action) dispatch(router, action)
      else if (arg.startsWith('claude-code-gui://')) {
        toast.warning(`Ignored deep link: ${arg}`)
      }
    }
  })

  // tauri-plugin-deep-link emits `deep-link://new-url` (Tauri 2 docs).
  unlistenDeep = await listen<string[]>('deep-link://new-url', (e) => {
    for (const url of e.payload) {
      const action = parse(url)
      if (action) dispatch(router, action)
      else toast.warning(`Ignored deep link: ${url}`)
    }
  })

  return () => {
    unlistenSingle?.()
    unlistenDeep?.()
    unlistenSingle = undefined
    unlistenDeep = undefined
  }
}
