import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { plansCreate, plansDelete, plansGet, plansList, plansUpdate } from '@/utils/ipc'
import type { PlanInput } from '@/types/ipc'

export const usePlansList = () => useQuery({ queryKey: qk.plans.list(), queryFn: plansList })

export const usePlan = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.plans.get(toValue(slug))),
    queryFn: () => plansGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })

export const usePlanCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: PlanInput) => plansCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plans.all }),
  })
}

export const usePlanUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, input }: { slug: string; input: PlanInput }) =>
      plansUpdate(slug, input),
    onSuccess: (_d, { slug }) => {
      qc.invalidateQueries({ queryKey: qk.plans.all })
      qc.invalidateQueries({ queryKey: qk.plans.get(slug) })
    },
  })
}

export const usePlanDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => plansDelete(slug),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plans.all }),
  })
}
