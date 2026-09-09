import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  hooksCreate,
  hooksDelete,
  hooksGet,
  hooksList,
  hooksRawGet,
  hooksRawPut,
  hooksUpdate,
} from '@/utils/ipc'
import type { HookInput, SettingsScope } from '@/types/ipc'

export const useHooksList = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.hooks.list(toValue(workingDir))),
    queryFn: () => hooksList(toValue(workingDir)),
  })

export const useHook = (
  id: MaybeRefOrGetter<string>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.hooks.get(toValue(id), toValue(workingDir))),
    queryFn: () => hooksGet(toValue(id), toValue(workingDir)),
    enabled: computed(() => !!toValue(id)),
  })

export const useHooksRaw = (
  scope: MaybeRefOrGetter<SettingsScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.hooks.raw(toValue(scope), toValue(workingDir))),
    queryFn: () => hooksRawGet(toValue(scope), toValue(workingDir)),
  })

export const useHookCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: HookInput) => hooksCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.hooks.all }),
  })
}

export const useHookUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; input: HookInput }) => hooksUpdate(v.id, v.input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.hooks.all }),
  })
}

export const useHookDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; workingDir?: string }) => hooksDelete(v.id, v.workingDir),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.hooks.all }),
  })
}

export const useHooksRawPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      scope: SettingsScope
      content: string
      workingDir?: string
      expectedMtimeMs?: number
    }) => hooksRawPut(v.scope, v.content, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.hooks.all }),
  })
}
