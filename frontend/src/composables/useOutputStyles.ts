import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import { outputStylesCreate, outputStylesDelete, outputStylesList } from '@/utils/ipc'
import type { OutputStyleInput, OutputStyleScope } from '@/types/ipc'

export const useOutputStylesList = (workingDir?: string) =>
  useQuery({ queryKey: qk.outputStyles.list(), queryFn: () => outputStylesList(workingDir) })

export const useOutputStyleCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: OutputStyleInput) => outputStylesCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.outputStyles.all }),
  })
}

export const useOutputStyleDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      id,
      scope,
      workingDir,
    }: {
      id: string
      scope: OutputStyleScope
      workingDir?: string
    }) => outputStylesDelete(id, scope, workingDir),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.outputStyles.all }),
  })
}
