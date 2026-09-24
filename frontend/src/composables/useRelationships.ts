import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { relationshipsGraph } from '@/utils/ipc'

/**
 * Cross-reference graph across agents/commands/skills. The same
 * `fs:change` listener that invalidates each domain's list query
 * also fires the graph rebuild — no bespoke wiring needed.
 */
export const useRelationshipsGraph = () =>
  useQuery({
    queryKey: ['relationships', 'graph'] as const,
    queryFn: relationshipsGraph,
    staleTime: 30_000,
  })

export type RelatedKind = 'agent' | 'command' | 'skill'

/** How many nodes link to one agent/command/skill; drives whether a page offers the graph. */
export const useRelatedCount = (kind: RelatedKind, slug: MaybeRefOrGetter<string>) => {
  const graph = useRelationshipsGraph()
  return computed(() => {
    const g = graph.data.value
    const s = toValue(slug)
    if (!g || !s) return 0
    if (kind === 'agent') {
      return (g.agentSkills[s]?.length ?? 0) + (g.agentCommands[s]?.length ?? 0)
    }
    if (kind === 'command') return g.commandAgent[s] ? 1 : 0
    return g.skillAgents[s]?.length ?? 0
  })
}
