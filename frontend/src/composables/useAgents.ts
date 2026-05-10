import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  agentsCreate,
  agentsDelete,
  agentsExport,
  agentsGet,
  agentsImport,
  agentsList,
  agentsSkillCounts,
  agentsUpdate,
} from '@/utils/ipc'
import type { AgentImport, AgentInput } from '@/types/ipc'

export const useAgentsList = () =>
  useQuery({ queryKey: qk.agents.list(), queryFn: agentsList })

export const useAgent = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.agents.get(toValue(slug))),
    queryFn: () => agentsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })

export const useAgentSkillCounts = () =>
  useQuery({ queryKey: qk.agents.skillCounts(), queryFn: agentsSkillCounts })

export const useAgentCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: AgentInput) => agentsCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.agents.all }),
  })
}

export const useAgentUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, input }: { slug: string; input: AgentInput }) =>
      agentsUpdate(slug, input),
    onSuccess: (_data, { slug }) => {
      qc.invalidateQueries({ queryKey: qk.agents.all })
      qc.invalidateQueries({ queryKey: qk.agents.get(slug) })
    },
  })
}

export const useAgentDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => agentsDelete(slug),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.agents.all }),
  })
}

export const useAgentExport = () =>
  useMutation({ mutationFn: (slug: string) => agentsExport(slug) })

export const useAgentImport = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (payload: AgentImport) => agentsImport(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.agents.all }),
  })
}
