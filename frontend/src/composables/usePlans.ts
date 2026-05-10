import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { plansGet, plansList } from '@/utils/ipc'

export const usePlansList = () => useQuery({ queryKey: qk.plans.list(), queryFn: plansList })

export const usePlan = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.plans.get(toValue(slug))),
    queryFn: () => plansGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })
