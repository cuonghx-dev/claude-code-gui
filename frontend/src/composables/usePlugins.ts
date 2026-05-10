import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  pluginsDelete,
  pluginsGet,
  pluginsList,
  pluginsSetEnabled,
  pluginsUpdateSkills,
} from '@/utils/ipc'

export const usePluginsList = () =>
  useQuery({ queryKey: qk.plugins.list(), queryFn: pluginsList })

export const usePlugin = (id: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.plugins.get(toValue(id))),
    queryFn: () => pluginsGet(toValue(id)),
    enabled: computed(() => !!toValue(id)),
  })

export const usePluginDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => pluginsDelete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plugins.all }),
  })
}

export const usePluginSetEnabled = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      pluginsSetEnabled(id, enabled),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plugins.all }),
  })
}

export const usePluginUpdateSkills = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, slugs }: { id: string; slugs: string[] }) =>
      pluginsUpdateSkills(id, slugs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plugins.all }),
  })
}
