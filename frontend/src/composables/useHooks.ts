import { useQuery } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import { hooksList } from '@/utils/ipc'

export const useHooksList = (workingDir?: string) =>
  useQuery({ queryKey: qk.hooks.list(workingDir), queryFn: () => hooksList(workingDir) })
