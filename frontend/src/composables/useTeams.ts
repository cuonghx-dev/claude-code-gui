import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { teamsGet, teamsList } from '@/utils/ipc'

export const useTeamsList = () => useQuery({ queryKey: qk.teams.list(), queryFn: teamsList })

export const useTeam = (id: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.teams.get(toValue(id))),
    queryFn: () => teamsGet(toValue(id)),
    enabled: computed(() => !!toValue(id)),
  })
