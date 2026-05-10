import { onBeforeUnmount, onMounted, watch, type MaybeRefOrGetter, toValue } from 'vue'
import { unwatchPath, watchProjectDir } from '@/utils/ipc'

/**
 * Subscribe the backend watcher to a project working directory while the
 * caller is mounted. Re-subscribes if `path` changes; tears down on unmount.
 *
 * Backend dedupes by canonical path so re-mounting the same project page
 * is a no-op other than refreshing the subscription id we hold locally.
 */
export function useProjectWatcher(path: MaybeRefOrGetter<string | undefined>) {
  let currentId: string | undefined

  async function subscribe(p?: string) {
    if (!p) return
    try {
      currentId = await watchProjectDir(p)
    } catch (e) {
      console.warn('watch_project_dir failed', e)
    }
  }
  async function unsubscribe() {
    if (!currentId) return
    const id = currentId
    currentId = undefined
    try {
      await unwatchPath(id)
    } catch (e) {
      console.warn('unwatch_path failed', e)
    }
  }

  onMounted(() => {
    void subscribe(toValue(path))
  })

  watch(
    () => toValue(path),
    async (next) => {
      await unsubscribe()
      await subscribe(next)
    },
  )

  onBeforeUnmount(() => {
    void unsubscribe()
  })
}
