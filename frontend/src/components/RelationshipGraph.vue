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
        cx: 200,
        cy: 130,
        href: `/agents/${encodeURIComponent(props.agentSlug)}`,
      }
    : props.commandSlug
      ? {
          id: `command:${props.commandSlug}`,
          kind: 'command',
          label: `/${props.commandSlug}`,
          cx: 200,
          cy: 130,
          href: `/commands/${encodeURIComponent(props.commandSlug)}`,
        }
      : props.skillSlug
        ? {
            id: `skill:${props.skillSlug}`,
            kind: 'skill',
            label: props.skillSlug,
            cx: 200,
            cy: 130,
            href: `/skills/${encodeURIComponent(props.skillSlug)}`,
          }
        : null
  if (!center) return { nodes: [], edges: [] }
  nodes.push(center)

  const ringPlace = (
    items: Array<{ id: string; kind: Node['kind']; label: string; href: string }>,
    radius: number,
    startAngle: number,
  ) => {
    if (!items.length) return
    const step = items.length === 1 ? 0 : Math.PI / Math.max(items.length - 1, 1)
    items.forEach((it, i) => {
      const angle = startAngle + i * step
      nodes.push({
        ...it,
        cx: center.cx + radius * Math.cos(angle),
        cy: center.cy + radius * Math.sin(angle),
      })
      edges.push({ from: center.id, to: it.id })
    })
  }

  if (props.agentSlug) {
    const skills = g.agentSkills[props.agentSlug] ?? []
    const commands = g.agentCommands[props.agentSlug] ?? []
    ringPlace(
      skills.map((s) => ({
        id: `skill:${s}`,
        kind: 'skill',
        label: s,
        href: `/skills/${encodeURIComponent(s)}`,
      })),
      120,
      Math.PI, // left half
    )
    ringPlace(
      commands.map((c) => ({
        id: `command:${c}`,
        kind: 'command',
        label: `/${c}`,
        href: `/commands/${encodeURIComponent(c)}`,
      })),
      120,
      0, // right half
    )
  } else if (props.commandSlug) {
    const agentSlug = g.commandAgent[props.commandSlug]
    if (agentSlug) {
      ringPlace(
        [
          {
            id: `agent:${agentSlug}`,
            kind: 'agent',
            label: agentSlug,
            href: `/agents/${encodeURIComponent(agentSlug)}`,
          },
        ],
        120,
        0,
      )
    }
  } else if (props.skillSlug) {
    const agents = g.skillAgents[props.skillSlug] ?? []
    ringPlace(
      agents.map((a) => ({
        id: `agent:${a}`,
        kind: 'agent',
        label: a,
        href: `/agents/${encodeURIComponent(a)}`,
      })),
      120,
      0,
    )
  }

  return { nodes, edges }
})

const colors: Record<Node['kind'], string> = {
  agent: '#7c3aed', // violet-600
  command: '#0ea5e9', // sky-500
  skill: '#10b981', // emerald-500
}
</script>

<template>
  <div class="relative">
    <svg viewBox="0 0 400 260" class="h-64 w-full">
      <line
        v-for="(e, i) in layout.edges"
        :key="i"
        :x1="layout.nodes.find((n) => n.id === e.from)?.cx ?? 0"
        :y1="layout.nodes.find((n) => n.id === e.from)?.cy ?? 0"
        :x2="layout.nodes.find((n) => n.id === e.to)?.cx ?? 0"
        :y2="layout.nodes.find((n) => n.id === e.to)?.cy ?? 0"
        stroke="currentColor"
        stroke-width="1"
        opacity="0.3"
      />
      <g v-for="n in layout.nodes" :key="n.id">
        <circle :cx="n.cx" :cy="n.cy" r="14" :fill="colors[n.kind]" />
        <text
          :x="n.cx"
          :y="n.cy + 32"
          text-anchor="middle"
          font-size="10"
          fill="currentColor"
          class="font-mono"
        >
          {{ n.label.length > 14 ? n.label.slice(0, 13) + '…' : n.label }}
        </text>
      </g>
    </svg>
    <ul class="mt-2 space-y-1 text-xs">
      <li v-for="n in layout.nodes" :key="n.id" class="flex items-baseline gap-2">
        <span
          class="inline-block h-2 w-2 rounded-full"
          :style="{ backgroundColor: colors[n.kind] }"
        />
        <RouterLink :to="n.href" class="hover:underline">{{ n.label }}</RouterLink>
        <span class="text-neutral-500 dark:text-neutral-400">({{ n.kind }})</span>
      </li>
    </ul>
  </div>
</template>
