import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { commandsGet, commandsList } from '@/utils/ipc'

export const useCommandsList = () =>
  useQuery({ queryKey: qk.commands.list(), queryFn: commandsList })

export const useCommand = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.commands.get(toValue(slug))),
    queryFn: () => commandsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })
