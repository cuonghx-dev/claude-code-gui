import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  workflowsCreate,
  workflowsDelete,
  workflowsGet,
  workflowsList,
  workflowsUpdate,
} from '@/utils/ipc'
import type { WorkflowInput } from '@/types/ipc'

export const useWorkflowsList = () =>
  useQuery({ queryKey: qk.workflows.list(), queryFn: workflowsList })

export const useWorkflow = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.workflows.get(toValue(slug))),
    queryFn: () => workflowsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })

export const useWorkflowCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: WorkflowInput) => workflowsCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.workflows.all }),
  })
}

export const useWorkflowUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, input }: { slug: string; input: WorkflowInput }) =>
      workflowsUpdate(slug, input),
    onSuccess: (_d, { slug }) => {
      qc.invalidateQueries({ queryKey: qk.workflows.all })
      qc.invalidateQueries({ queryKey: qk.workflows.get(slug) })
    },
  })
}

export const useWorkflowDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => workflowsDelete(slug),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.workflows.all }),
  })
}
