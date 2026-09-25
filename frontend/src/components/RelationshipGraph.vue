<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useRelationshipsGraph } from '@/composables/useRelationships'

const props = defineProps<{
  /** Center node — pick exactly one. */
  agentSlug?: string
  commandSlug?: string
  skillSlug?: string
}>()

const graph = useRelationshipsGraph()

interface Node {
  id: string
  kind: 'agent' | 'command' | 'skill'
  label: string
  cx: number
  cy: number
  href: string
}
interface Edge {
  from: string
  to: string
}

const layout = computed<{ nodes: Node[]; edges: Edge[] }>(() => {
  const g = graph.data.value
  if (!g) return { nodes: [], edges: [] }
  const nodes: Node[] = []
  const edges: Edge[] = []

  const center: Node | null = props.agentSlug
    ? {
        id: `agent:${props.agentSlug}`,
        kind: 'agent',
        label: props.agentSlug,
        cx: 0,
        cy: 0,
        href: `/agents/${encodeURIComponent(props.agentSlug)}`,
      }
    : props.commandSlug
      ? {
          id: `command:${props.commandSlug}`,
          kind: 'command',
          label: `/${props.commandSlug}`,
          cx: 0,
          cy: 0,
          href: `/commands/${encodeURIComponent(props.commandSlug)}`,
        }
      : props.skillSlug
        ? {
            id: `skill:${props.skillSlug}`,
            kind: 'skill',
            label: props.skillSlug,
            cx: 0,
            cy: 0,
            href: `/skills/${encodeURIComponent(props.skillSlug)}`,
          }
        : null
  if (!center) return { nodes: [], edges: [] }
  nodes.push(center)

  // Upper arc for skills (and agents around a skill/command), lower arc for
  // commands — mirrors the handoff graph.
  const W = 296
  const H = 280
  center.cx = W / 2
  center.cy = H / 2
  const arc = (
    items: Array<{ id: string; kind: Node['kind']; label: string; href: string }>,
    upper: boolean,
  ) => {
    items.forEach((it, i) => {
      const k = items.length
      const t = k === 1 ? 0.5 : i / (k - 1)
      const deg = upper ? 205 + t * 130 : 155 - t * 130
      const rad = (deg * Math.PI) / 180
      nodes.push({
        ...it,
        cx: center.cx + 86 * Math.cos(rad),
        cy: center.cy + 96 * Math.sin(rad),
      })
      edges.push({ from: center.id, to: it.id })
    })
  }

  if (props.agentSlug) {
    const skills = g.agentSkills[props.agentSlug] ?? []
    const commands = g.agentCommands[props.agentSlug] ?? []
    arc(
      skills.map((s) => ({ id: `skill:${s}`, kind: 'skill', label: s, href: `/skills/${encodeURIComponent(s)}` })),
      true,
    )
    arc(
      commands.map((c) => ({ id: `command:${c}`, kind: 'command', label: `/${c}`, href: `/commands/${encodeURIComponent(c)}` })),
      false,
    )
  } else if (props.commandSlug) {
    const agentSlug = g.commandAgent[props.commandSlug]
    if (agentSlug) {
      arc([{ id: `agent:${agentSlug}`, kind: 'agent', label: agentSlug, href: `/agents/${encodeURIComponent(agentSlug)}` }], true)
    }
  } else if (props.skillSlug) {
    const agents = g.skillAgents[props.skillSlug] ?? []
    arc(
      agents.map((a) => ({ id: `agent:${a}`, kind: 'agent', label: a, href: `/agents/${encodeURIComponent(a)}` })),
      true,
    )
  }

  return { nodes, edges }
})

const center = computed(() => layout.value.nodes[0])
const leaves = computed(() => layout.value.nodes.slice(1))

/** Curved edge leaving the center horizontally, landing on the leaf. */
function edgePath(n: Node): string {
  const c = center.value!
  const c1x = c.cx + (n.cx - c.cx) * 0.6
  const c2x = n.cx + (c.cx - n.cx) * 0.1
  const c2y = n.cy + (c.cy - n.cy) * 0.12
  return `M${c.cx} ${c.cy} C ${c1x} ${c.cy} ${c2x} ${c2y} ${n.cx} ${n.cy}`
}

const leafStyle: Record<Node['kind'], string> = {
  agent: 'border-color:#DEDAD1;color:#1F1E1B',
  skill: 'border-color:#D9E4DB;color:#3F7A4E',
  command: 'border-color:#E3D7EC;color:#7A4FA0',
}
</script>

<template>
  <div v-if="center" class="relative h-[280px] w-[296px] self-center">
    <svg width="296" height="280" class="absolute inset-0" aria-hidden="true">
      <g stroke="#D6D0C4" stroke-width="1.5" fill="none">
        <path v-for="n in leaves" :key="n.id" :d="edgePath(n)" />
      </g>
    </svg>
    <RouterLink
      :to="center.href"
      class="absolute flex h-9 max-w-[140px] -translate-x-1/2 -translate-y-1/2 items-center justify-center truncate rounded-[8px] px-3 font-mono text-[11.5px] font-medium text-white"
      :style="{ left: `${center.cx}px`, top: `${center.cy}px`, background: '#1F1E1B', minWidth: '112px' }"
    >
      {{ center.label }}
    </RouterLink>
    <RouterLink
      v-for="n in leaves"
      :key="n.id"
      :to="n.href"
      class="absolute max-w-[120px] -translate-x-1/2 -translate-y-1/2 truncate rounded-[7px] border bg-white px-[9px] py-1.5 font-mono text-[11px] transition-shadow hover:shadow-[0_2px_8px_rgba(40,30,20,.08)]"
      :style="`left:${n.cx}px;top:${n.cy}px;${leafStyle[n.kind]}`"
      :title="n.label"
    >
      {{ n.label }}
    </RouterLink>
    <p
      v-if="!leaves.length"
      class="absolute inset-x-0 bottom-6 text-center text-[12px]"
      style="color: var(--ccg-subtle);"
    >
      No links yet.
    </p>
  </div>
</template>
