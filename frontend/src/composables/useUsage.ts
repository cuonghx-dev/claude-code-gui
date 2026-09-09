import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { usageActivity, usageRefresh, usageRollup } from '@/utils/ipc'
import type { UsageQuery } from '@/types/ipc'

export const useUsageRollup = (query: MaybeRefOrGetter<UsageQuery>) =>
  useQuery({
    queryKey: computed(() => qk.usage.rollup(JSON.stringify(toValue(query)))),
    queryFn: () => usageRollup(toValue(query)),
    // The rollup rescans changed transcripts, so it is not free to repeat.
    staleTime: 60_000,
  })

export const useUsageActivity = (days: MaybeRefOrGetter<number>) =>
  useQuery({
    queryKey: computed(() => qk.usage.activity(toValue(days))),
    queryFn: () => usageActivity(toValue(days)),
    staleTime: 60_000,
  })

export const useUsageRefresh = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: usageRefresh,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.usage.all }),
  })
}
