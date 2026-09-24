<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Bot, Map, Search, Slash, Sparkles } from 'lucide-vue-next'
import { useAgentsList } from '@/composables/useAgents'
import { useCommandsList } from '@/composables/useCommands'
import { useSkillsList } from '@/composables/useSkills'
import { usePlansList } from '@/composables/usePlans'

/**
 * ⌘K / Ctrl+K palette over agents, commands, skills and plans (SPEC §5).
 * Filters the list queries the sidebar already holds, so opening it costs no
 * IPC round trip.
 */

type Kind = 'agent' | 'command' | 'skill' | 'plan'

interface Entry {
  kind: Kind
  label: string
  slug: string
  description: string
  body: string
  to: string
}

interface Hit extends Entry {
  rank: number
}

const MAX_HITS = 30

const router = useRouter()
const agents = useAgentsList()
const commands = useCommandsList()
const skills = useSkillsList()
const plans = usePlansList()

const open = ref(false)
const query = ref('')
const active = ref(0)
const input = ref<HTMLInputElement | null>(null)
let lastFocused: HTMLElement | null = null

const icons = { agent: Bot, command: Slash, skill: Sparkles, plan: Map } as const

const entries = computed<Entry[]>(() => [
  ...(agents.data.value ?? []).map((a) => ({
    kind: 'agent' as const,
    label: a.frontmatter.name ?? a.slug,
    slug: a.slug,
    description: a.frontmatter.description ?? '',
    body: a.body,
    to: `/agents/${encodeURIComponent(a.slug)}`,
  })),
  ...(commands.data.value ?? []).map((c) => ({
    kind: 'command' as const,
    label: `/${c.slug}`,
    slug: c.slug,
    description: c.frontmatter.description ?? '',
    body: c.body,
    to: `/commands/${encodeURIComponent(c.slug)}`,
  })),
  ...(skills.data.value ?? []).map((s) => ({
    kind: 'skill' as const,
    label: s.frontmatter.name ?? s.slug,
    slug: s.slug,
    description: s.frontmatter.description ?? '',
    body: s.body,
    to: `/skills/${encodeURIComponent(s.slug)}`,
  })),
  ...(plans.data.value ?? []).map((p) => ({
    kind: 'plan' as const,
    label: p.title,
    slug: p.slug,
    description: '',
    body: p.body,
    to: `/plans/${encodeURIComponent(p.slug)}`,
  })),
])

/** Lower rank sorts first: name prefix, name, description, then body. */
function rankOf(e: Entry, q: string): number | null {
  const label = e.label.toLowerCase().replace(/^\//, '')
  if (label.startsWith(q) || e.slug.toLowerCase().startsWith(q)) return 0
  if (label.includes(q) || e.slug.toLowerCase().includes(q)) return 1
  if (e.description.toLowerCase().includes(q)) return 2
  if (e.body.toLowerCase().includes(q)) return 3
  return null
}

const hits = computed<Hit[]>(() => {
  const q = query.value.trim().toLowerCase().replace(/^\//, '')
  if (!q) return []
  const out: Hit[] = []
  for (const e of entries.value) {
    const rank = rankOf(e, q)
    if (rank !== null) out.push({ ...e, rank })
  }
  out.sort((a, b) => a.rank - b.rank || a.label.localeCompare(b.label))
  return out.slice(0, MAX_HITS)
})

watch(query, () => {
  active.value = 0
})

async function show() {
  lastFocused = document.activeElement as HTMLElement | null
  open.value = true
  await nextTick()
  input.value?.focus()
  input.value?.select()
}

function close() {
  open.value = false
  lastFocused?.focus?.()
}

function go(hit: Hit | undefined) {
  if (!hit) return
  close()
  query.value = ''
  void router.push(hit.to)
}

function onInputKey(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    active.value = Math.min(active.value + 1, hits.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    active.value = Math.max(active.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    go(hits.value[active.value])
  } else if (e.key === 'Escape') {
    e.preventDefault()
    close()
  }
}

watch(active, async (i) => {
  await nextTick()
  document.getElementById(`gs-hit-${i}`)?.scrollIntoView({ block: 'nearest' })
})

function onGlobalKey(e: KeyboardEvent) {
  // Inside a terminal the keys belong to claude's TUI (Ctrl+K kills a line).
  if ((e.target as HTMLElement | null)?.closest?.('.xterm')) return
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    if (open.value) close()
    else void show()
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onGlobalKey))

const isMac = typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform)
</script>

<template>
  <button
    type="button"
    class="mx-1 mb-3 flex items-center gap-2 rounded-md border px-2.5 py-1.5 text-xs text-neutral-500 transition-colors hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800"
    style="border-color: var(--ccg-hairline);"
    aria-haspopup="dialog"
    @click="show"
  >
    <Search class="h-3.5 w-3.5" aria-hidden="true" />
    <span class="flex-1 text-left">Search</span>
    <kbd class="font-mono text-[10px]">{{ isMac ? '⌘K' : 'Ctrl K' }}</kbd>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-start justify-center bg-black/40 px-4 pt-[12vh]"
      @mousedown.self="close"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Search agents, commands, skills and plans"
        class="w-full max-w-xl overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-2xl dark:border-neutral-800 dark:bg-neutral-900"
      >
        <div class="flex items-center gap-2 border-b border-neutral-200 px-3 dark:border-neutral-800">
          <Search class="h-4 w-4 shrink-0 text-neutral-400" aria-hidden="true" />
          <input
            ref="input"
            v-model="query"
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls="gs-results"
            :aria-activedescendant="hits.length ? `gs-hit-${active}` : undefined"
            class="w-full bg-transparent py-3 text-sm outline-none placeholder:text-neutral-400"
            placeholder="Search agents, commands, skills, plans…"
            @keydown="onInputKey"
          />
        </div>
        <ul id="gs-results" role="listbox" class="max-h-[50vh] overflow-y-auto py-1">
          <li v-if="query.trim() && !hits.length" class="px-4 py-6 text-center text-sm text-neutral-500">
            No matches for “{{ query.trim() }}”
          </li>
          <li
            v-for="(h, i) in hits"
            :id="`gs-hit-${i}`"
            :key="`${h.kind}:${h.slug}`"
            role="option"
            :aria-selected="i === active"
            class="flex cursor-pointer items-start gap-3 px-3 py-2"
            :class="i === active ? 'bg-neutral-100 dark:bg-neutral-800' : ''"
            @mousemove="active = i"
            @click="go(h)"
          >
            <component :is="icons[h.kind]" class="mt-0.5 h-4 w-4 shrink-0 text-neutral-400" aria-hidden="true" />
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-medium">{{ h.label }}</div>
              <div v-if="h.description" class="truncate text-xs text-neutral-500 dark:text-neutral-400">
                {{ h.description }}
              </div>
            </div>
            <span class="shrink-0 text-[10px] uppercase tracking-wide text-neutral-400">{{ h.kind }}</span>
          </li>
        </ul>
      </div>
    </div>
  </Teleport>
</template>
