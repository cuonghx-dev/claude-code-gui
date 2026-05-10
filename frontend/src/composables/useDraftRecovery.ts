import { onBeforeUnmount, watch, type WatchSource } from 'vue'

const NS = 'ccg-draft:'
const TTL_MS = 7 * 24 * 60 * 60 * 1000 // 7 days

interface Envelope<T> {
  v: 1
  ts: number
  data: T
}

export interface DraftRecovery<T> {
  load(): T | undefined
  clear(): void
}

/**
 * Persist `state` to localStorage under a stable key whenever it changes.
 * Returns `{ load, clear }` so the caller can hydrate before mount and
 * scrub on a successful save.
 *
 * Drafts older than 7 days are dropped on read.
 */
export function useDraftRecovery<T>(
  key: string,
  source: WatchSource<T> | object,
): DraftRecovery<T> {
  const fullKey = NS + key

  const save = (value: T) => {
    try {
      const env: Envelope<T> = { v: 1, ts: Date.now(), data: value }
      localStorage.setItem(fullKey, JSON.stringify(env))
    } catch {
      // Quota errors and serialization failures are fine to drop here:
      // draft recovery is an availability convenience, not durability.
    }
  }

  watch(source as WatchSource<T>, (v) => save(v as T), { deep: true })

  onBeforeUnmount(() => {
    // Keep the draft on unmount; clear() is the explicit way to drop it.
  })

  return {
    load() {
      try {
        const raw = localStorage.getItem(fullKey)
        if (!raw) return undefined
        const env = JSON.parse(raw) as Envelope<T>
        if (env.v !== 1 || Date.now() - env.ts > TTL_MS) {
          localStorage.removeItem(fullKey)
          return undefined
        }
        return env.data
      } catch {
        return undefined
      }
    },
    clear() {
      try {
        localStorage.removeItem(fullKey)
      } catch {
        /* ignore */
      }
    },
  }
}
