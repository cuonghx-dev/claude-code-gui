<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { CircleCheck, CircleDot } from 'lucide-vue-next'

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
  <aside class="flex flex-col gap-3 text-[12.5px]">
    <section class="ccg-card p-3">
      <h3 class="ccg-section-label">Context</h3>
      <p v-if="!tokens" class="mt-2 text-[12px]" style="color: var(--ccg-subtle);">Waiting for first usage line…</p>
      <dl v-else class="mt-2 grid grid-cols-2 gap-x-3 gap-y-1 text-[12px]">
        <dt style="color: var(--ccg-muted);">Model</dt>
        <dd class="font-mono text-ink">{{ tokens.model ?? '—' }}</dd>
        <dt style="color: var(--ccg-muted);">Input</dt>
        <dd class="font-mono tabular-nums text-ink">{{ tokens.input.toLocaleString() }}</dd>
        <dt style="color: var(--ccg-muted);">Output</dt>
        <dd class="font-mono tabular-nums text-ink">{{ tokens.output.toLocaleString() }}</dd>
        <dt style="color: var(--ccg-muted);">Cached</dt>
        <dd class="font-mono tabular-nums text-ink">{{ tokens.cached.toLocaleString() }}</dd>
        <dt style="color: var(--ccg-muted);">Cost</dt>
        <dd class="font-mono tabular-nums text-ink">${{ tokens.cost.toFixed(4) }}</dd>
      </dl>
    </section>

    <section class="ccg-card p-3">
      <h3 class="ccg-section-label">Tool calls</h3>
      <p v-if="!tools.length" class="mt-2 text-[12px]" style="color: var(--ccg-subtle);">No tool calls yet.</p>
      <ol v-else class="mt-2 max-h-72 space-y-1 overflow-auto text-[12px]">
        <li
          v-for="(t, i) in tools"
          :key="`${t.name}-${i}`"
          class="flex items-baseline justify-between gap-2 font-mono"
        >
          <span class="flex min-w-0 items-center gap-1.5 truncate text-ink">
            <component
              :is="t.state === 'completed' ? CircleCheck : CircleDot"
              class="h-3 w-3 shrink-0"
              :stroke-width="1.5"
              :style="{ color: t.state === 'completed' ? 'var(--ccg-success)' : 'var(--ccg-purple)' }"
              :aria-label="t.state === 'completed' ? 'completed' : 'running'"
            />
            <span class="truncate">{{ t.name }}</span>
          </span>
          <span class="tabular-nums" style="color: var(--ccg-subtle);">
            {{ t.state }}<span v-if="t.durationMs"> · {{ t.durationMs }}ms</span>
          </span>
        </li>
      </ol>
    </section>
  </aside>
</template>
