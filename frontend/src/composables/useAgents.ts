import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { agentsGet, agentsList, agentsSkillCounts } from '@/utils/ipc'

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
