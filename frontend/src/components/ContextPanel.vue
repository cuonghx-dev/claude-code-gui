<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface TokenPayload {
  input: number
  output: number
  cached: number
  cacheWrite?: number
  cost: number
  model?: string | null
}
interface ToolPayload {
  name: string
  state: string
  durationMs?: number | null
  timestamp: string
}

const props = defineProps<{ sessionId: string | undefined }>()

const tokens = ref<TokenPayload | null>(null)
const tools = ref<ToolPayload[]>([])

let unlistenTokens: UnlistenFn | undefined
let unlistenTool: UnlistenFn | undefined

async function bind(id: string) {
  await unbind()
  tokens.value = null
  tools.value = []
  unlistenTokens = await listen<TokenPayload>(`context:tokens:${id}`, (e) => {
    tokens.value = e.payload
  })
  unlistenTool = await listen<ToolPayload>(`context:tool:${id}`, (e) => {
    tools.value = [e.payload, ...tools.value].slice(0, 50)
  })
}
async function unbind() {
  unlistenTokens?.()
  unlistenTool?.()
  unlistenTokens = undefined
  unlistenTool = undefined
}

watch(
  () => props.sessionId,
  (id) => {
    if (id) void bind(id)
    else void unbind()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  void unbind()
})
</script>

<template>
  <aside class="flex flex-col gap-3 text-sm">
    <section class="rounded-lg border border-neutral-200 bg-white p-3 dark:border-neutral-800 dark:bg-neutral-900">
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Context</h3>
      <p v-if="!tokens" class="mt-2 text-xs text-neutral-500">Waiting for first usage line…</p>
      <dl v-else class="mt-2 grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
        <dt class="text-neutral-500">Model</dt>
        <dd class="font-mono">{{ tokens.model ?? '—' }}</dd>
        <dt class="text-neutral-500">Input</dt>
        <dd class="tabular-nums">{{ tokens.input.toLocaleString() }}</dd>
        <dt class="text-neutral-500">Output</dt>
        <dd class="tabular-nums">{{ tokens.output.toLocaleString() }}</dd>
        <dt class="text-neutral-500">Cached</dt>
        <dd class="tabular-nums">{{ tokens.cached.toLocaleString() }}</dd>
        <dt class="text-neutral-500">Cost</dt>
        <dd class="tabular-nums">${{ tokens.cost.toFixed(4) }}</dd>
      </dl>
    </section>

    <section class="rounded-lg border border-neutral-200 bg-white p-3 dark:border-neutral-800 dark:bg-neutral-900">
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Tool calls</h3>
      <p v-if="!tools.length" class="mt-2 text-xs text-neutral-500">No tool calls yet.</p>
      <ol v-else class="mt-2 max-h-72 overflow-auto space-y-1 text-xs">
        <li
          v-for="(t, i) in tools"
          :key="`${t.name}-${i}`"
          class="flex items-baseline justify-between gap-2 font-mono"
        >
          <span class="truncate">
            <span :class="t.state === 'completed' ? 'text-emerald-600 dark:text-emerald-400' : 'text-violet-600 dark:text-violet-400'">●</span>
            {{ t.name }}
          </span>
          <span class="tabular-nums text-neutral-500">
            {{ t.state }}<span v-if="t.durationMs">  · {{ t.durationMs }}ms</span>
          </span>
        </li>
      </ol>
    </section>
  </aside>
</template>
