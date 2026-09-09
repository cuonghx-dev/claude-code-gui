import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  memoryAgentList,
  memoryDelete,
  memoryGet,
  memoryList,
  memoryPreview,
  memoryPut,
} from '@/utils/ipc'

export const useMemoryList = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.memory.list(toValue(workingDir))),
    queryFn: () => memoryList(toValue(workingDir)),
  })

export const useAgentMemoryList = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.memory.agent(toValue(workingDir))),
    queryFn: () => memoryAgentList(toValue(workingDir)),
  })

export const useMemoryDoc = (
  id: MaybeRefOrGetter<string>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.memory.get(toValue(id), toValue(workingDir))),
    queryFn: () => memoryGet(toValue(id), toValue(workingDir)),
    enabled: computed(() => !!toValue(id)),
  })

export const useMemoryPreview = (
  id: MaybeRefOrGetter<string>,
  workingDir: MaybeRefOrGetter<string | undefined>,
  enabled: MaybeRefOrGetter<boolean>,
) =>
  useQuery({
    queryKey: computed(() => qk.memory.preview(toValue(id), toValue(workingDir))),
    queryFn: () => memoryPreview(toValue(id), toValue(workingDir)),
    enabled: computed(() => toValue(enabled) && !!toValue(id)),
  })

export const useMemoryPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      id: string
      content: string
      workingDir?: string
      expectedMtimeMs?: number
    }) => memoryPut(v.id, v.content, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.memory.all }),
  })
}

export const useMemoryDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; workingDir?: string }) => memoryDelete(v.id, v.workingDir),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.memory.all }),
  })
}
