import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  commandsCreate,
  commandsDelete,
  commandsGet,
  commandsList,
  commandsUpdate,
} from '@/utils/ipc'
import type { CommandInput } from '@/types/ipc'

export const useCommandsList = () =>
  useQuery({ queryKey: qk.commands.list(), queryFn: commandsList })

export const useCommand = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.commands.get(toValue(slug))),
    queryFn: () => commandsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })

export const useCommandCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: CommandInput) => commandsCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.commands.all }),
  })
}

export const useCommandUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, input }: { slug: string; input: CommandInput }) =>
      commandsUpdate(slug, input),
    onSuccess: (_d, { slug }) => {
      qc.invalidateQueries({ queryKey: qk.commands.all })
      qc.invalidateQueries({ queryKey: qk.commands.get(slug) })
    },
  })
}

export const useCommandDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => commandsDelete(slug),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.commands.all }),
  })
}
