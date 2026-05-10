import { useQuery } from '@tanstack/vue-query'
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
