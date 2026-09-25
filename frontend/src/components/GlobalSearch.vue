<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
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
    class="flex h-8 flex-none items-center gap-2 rounded-[7px] border bg-white px-2.5 text-left text-[13px] transition-colors hover:border-[#C9C3B7]"
    style="border-color: var(--ccg-hairline-strong); color: var(--ccg-subtle);"
    aria-haspopup="dialog"
    @click="show"
  >
    <span class="flex-1">Search…</span>
    <kbd class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">{{ isMac ? '⌘K' : 'Ctrl K' }}</kbd>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-x-0 bottom-0 top-[38px] z-50 flex justify-center px-4 pt-[90px]"
      style="background: rgba(31, 30, 27, .18);"
      @mousedown.self="close"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Search agents, commands, skills and plans"
        class="w-full max-w-[600px] self-start overflow-hidden rounded-[12px] bg-white"
        style="box-shadow: 0 24px 60px rgba(40, 30, 20, .3), 0 0 0 1px rgba(0, 0, 0, .08);"
      >
        <div class="flex items-center gap-2.5 border-b px-4 py-3.5" style="border-color: var(--ccg-hairline-soft);">
          <input
            ref="input"
            v-model="query"
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls="gs-results"
            :aria-activedescendant="hits.length ? `gs-hit-${active}` : undefined"
            class="w-full bg-transparent text-[16px] text-ink outline-none placeholder:text-[#A29E94]"
            placeholder="Search agents, commands, skills, plans…"
            @keydown="onInputKey"
          />
          <span class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">esc</span>
        </div>
        <ul id="gs-results" role="listbox" class="max-h-[50vh] overflow-y-auto p-1.5">
          <li
            v-if="!query.trim()"
            class="px-2.5 py-5 text-center text-[13px]"
            style="color: var(--ccg-subtle);"
          >
            Type to search by name, description or body.
          </li>
          <li
            v-else-if="!hits.length"
            class="px-2.5 py-5 text-center text-[13px]"
            style="color: var(--ccg-subtle);"
          >
            No matches for “{{ query.trim() }}”
          </li>
          <li
            v-for="(h, i) in hits"
            :id="`gs-hit-${i}`"
            :key="`${h.kind}:${h.slug}`"
            role="option"
            :aria-selected="i === active"
            class="flex cursor-pointer items-center gap-2.5 rounded-[7px] px-2.5 py-[9px]"
            :style="i === active ? 'background: var(--ccg-sidebar);' : ''"
            @mousemove="active = i"
            @click="go(h)"
          >
            <span
              class="w-16 flex-none font-mono text-[10.5px] uppercase tracking-[.04em]"
              style="color: var(--ccg-subtle);"
            >{{ h.kind }}</span>
            <span class="flex-none font-mono text-[13px] font-medium text-ink">{{ h.label }}</span>
            <span
              v-if="h.description"
              class="min-w-0 flex-1 truncate text-[12.5px]"
              style="color: var(--ccg-subtle);"
            >{{ h.description }}</span>
          </li>
        </ul>
        <div
          class="flex gap-3.5 border-t px-4 py-2 text-[11.5px]"
          style="border-color: var(--ccg-hairline-soft); background: var(--ccg-canvas-soft); color: var(--ccg-subtle);"
        >
          <span>↑↓ move</span><span>↵ open</span><span class="flex-1" /><span>Agents · Commands · Skills · Plans</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>
