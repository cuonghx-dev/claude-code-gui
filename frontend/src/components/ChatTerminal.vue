<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { useDebounceFn } from '@vueuse/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  terminalSessionCreate,
  terminalSessionInput,
  terminalSessionKill,
  terminalSessionResize,
} from '@/utils/ipc'
import type { TerminalOpts } from '@/types/ipc'

const props = defineProps<{
  /**
   * Terminal launch options. The component creates a session on mount and
   * kills it on unmount. Re-rendering with different opts spawns a new
   * session and discards the old one.
   */
  opts: TerminalOpts
}>()

const emit = defineEmits<{
  ready: [sessionId: string]
  exit: [exitCode: number]
  error: [message: string]
}>()

const host = ref<HTMLDivElement>()
const errorMessage = ref('')
const sessionId = ref<string>('')

let term: Terminal | undefined
let fit: FitAddon | undefined
let unlistenOutput: UnlistenFn | undefined
let unlistenExit: UnlistenFn | undefined

async function start() {
  errorMessage.value = ''
  sessionId.value = ''

  term = new Terminal({
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
    fontSize: 13,
    cursorBlink: true,
    theme: { background: '#0a0a0a' },
  })
  fit = new FitAddon()
  term.loadAddon(fit)
  term.loadAddon(new WebLinksAddon())
  if (host.value) {
    term.open(host.value)
    fit.fit()
    term.focus()
  }

  let id: string
  try {
    const sid = await terminalSessionCreate({
      ...props.opts,
      cols: term.cols,
      rows: term.rows,
    })
    // SessionId is `Uuid` serialized as a string via #[serde(transparent)].
    id = String(sid)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
    emit('error', errorMessage.value)
    return
  }
  sessionId.value = id
  emit('ready', id)

  unlistenOutput = await listen<{ data: string }>(`pty:output:${id}`, (e) => {
    term?.write(e.payload.data)
  })
  unlistenExit = await listen<{ exitCode: number }>(`pty:exit:${id}`, (e) => {
    emit('exit', e.payload.exitCode)
    term?.writeln(`\r\n\x1b[33m[session exited code=${e.payload.exitCode}]\x1b[0m`)
  })

  term.onData((data) => {
    void terminalSessionInput(id, data)
  })

  // Coalesce resize events; xterm fires onResize on every grid recompute.
  const debouncedResize = useDebounceFn(
    (cols: number, rows: number) => terminalSessionResize(id, cols, rows),
    100,
  )
  term.onResize(({ cols, rows }) => debouncedResize(cols, rows))

  // Refit on container resize.
  const ro = new ResizeObserver(() => fit?.fit())
  if (host.value) ro.observe(host.value)
  // Stash for cleanup
  ;(term as any).__ro = ro
}

async function stop() {
  unlistenOutput?.()
  unlistenExit?.()
  unlistenOutput = undefined
  unlistenExit = undefined
  if (sessionId.value) {
    try {
      await terminalSessionKill(sessionId.value)
    } catch {
      // session may already have exited
    }
  }
  if (term) {
    const ro = (term as any).__ro as ResizeObserver | undefined
    ro?.disconnect()
    term.dispose()
    term = undefined
    fit = undefined
  }
}

onMounted(() => {
  void start()
})
onBeforeUnmount(() => {
  void stop()
})

// Re-launch only when meaningful fields change. The parent often passes a
// fresh opts object literal on background refetches (tanstack-query window
// focus, etc.); reference-only watching would kill the live PTY mid-keystroke.
function optsEqual(a: TerminalOpts, b: TerminalOpts) {
  return (
    a.agentSlug === b.agentSlug &&
    a.workingDir === b.workingDir &&
    a.model === b.model &&
    a.permissionMode === b.permissionMode &&
    a.outputStyleId === b.outputStyleId &&
    a.resumeSessionId === b.resumeSessionId &&
    a.commandTemplate === b.commandTemplate
  )
}

watch(
  () => props.opts,
  async (next, prev) => {
    if (prev && optsEqual(prev, next)) return
    await stop()
    await start()
  },
  { deep: false },
)

defineExpose({ sessionId, errorMessage })
</script>

<template>
  <div class="flex h-full flex-col">
    <p
      v-if="errorMessage"
      class="m-2 rounded-md border border-red-300 bg-red-50 p-2 text-xs text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <div ref="host" class="min-h-0 flex-1 overflow-hidden bg-neutral-950" />
  </div>
</template>
