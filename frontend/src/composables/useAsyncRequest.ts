import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onBeforeUnmount, ref, type Ref } from 'vue'

/**
 * Subscribe to a request-scoped event topic and surface state for the UI.
 *
 * Pattern: backend command returns a `RequestId`; progress/done/error
 * payloads then arrive on `<topic>:<request_id>`. The composable handles
 * teardown automatically and resets state when `start` is called again.
 *
 * Payload contract: `{ kind: 'progress'|'delta'|'done'|'error', ... }`.
 */
export interface AsyncRequestEvent {
  kind: 'progress' | 'delta' | 'done' | 'error'
  step?: string
  percent?: number | null
  text?: string
  error?: string
}

export interface AsyncRequest {
  inFlight: Ref<boolean>
  step: Ref<string>
  percent: Ref<number | null>
  buffer: Ref<string>
  errorMessage: Ref<string>
  /**
   * Wire to a topic + request id. Returns a Promise that resolves on `done`
   * and rejects on `error`. The promise also resolves if the user calls
   * `cancel()` (the backend job keeps running, but the UI stops listening).
   */
  start(topic: string, requestId: string): Promise<void>
  cancel(): void
}

export function useAsyncRequest(): AsyncRequest {
  const inFlight = ref(false)
  const step = ref('')
  const percent = ref<number | null>(null)
  const buffer = ref('')
  const errorMessage = ref('')

  let unlisten: UnlistenFn | undefined
  let resolveDone: (() => void) | undefined
  let rejectDone: ((e: Error) => void) | undefined

  function teardown() {
    unlisten?.()
    unlisten = undefined
    inFlight.value = false
  }

  function reset() {
    step.value = ''
    percent.value = null
    buffer.value = ''
    errorMessage.value = ''
  }

  async function start(topic: string, requestId: string) {
    teardown()
    reset()
    inFlight.value = true
    return new Promise<void>(async (resolve, reject) => {
      resolveDone = resolve
      rejectDone = reject
      unlisten = await listen<AsyncRequestEvent>(`${topic}:${requestId}`, (e) => {
        const p = e.payload
        switch (p.kind) {
          case 'progress':
            step.value = p.step ?? ''
            percent.value = p.percent ?? null
            break
          case 'delta':
            buffer.value += p.text ?? ''
            break
          case 'done':
            teardown()
            resolveDone?.()
            break
          case 'error':
            errorMessage.value = p.error ?? 'unknown error'
            teardown()
            rejectDone?.(new Error(errorMessage.value))
            break
        }
      })
    })
  }

  function cancel() {
    teardown()
    resolveDone?.()
  }

  onBeforeUnmount(teardown)

  return { inFlight, step, percent, buffer, errorMessage, start, cancel }
}
