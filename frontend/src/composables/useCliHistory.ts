import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { cliHistoryGet, cliHistoryList } from '@/utils/ipc'

export const useTerminalsList = () =>
  useQuery({ queryKey: qk.terminals.list(), queryFn: cliHistoryList })

export const useTerminalReplay = (id: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.terminals.get(toValue(id))),
    queryFn: () => cliHistoryGet(toValue(id)),
    enabled: computed(() => !!toValue(id)),
    // Snapshots are written once when a session exits and never change.
    staleTime: Infinity,
  })
