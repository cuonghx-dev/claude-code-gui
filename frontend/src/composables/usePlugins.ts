import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { pluginsGet, pluginsList } from '@/utils/ipc'

export const usePluginsList = () =>
  useQuery({ queryKey: qk.plugins.list(), queryFn: pluginsList })

export const usePlugin = (id: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.plugins.get(toValue(id))),
    queryFn: () => pluginsGet(toValue(id)),
    enabled: computed(() => !!toValue(id)),
  })
