import { useQuery } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import { outputStylesList } from '@/utils/ipc'

export const useOutputStylesList = () =>
  useQuery({ queryKey: qk.outputStyles.list(), queryFn: () => outputStylesList() })
