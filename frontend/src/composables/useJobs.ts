import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { jobsGet, jobsList } from '@/utils/ipc'

export const useJobsList = () => useQuery({ queryKey: qk.jobs.list(), queryFn: jobsList })

export const useJob = (jobId: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.jobs.get(toValue(jobId))),
    queryFn: () => jobsGet(toValue(jobId)),
    enabled: computed(() => !!toValue(jobId)),
  })
