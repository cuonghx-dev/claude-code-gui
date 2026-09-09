import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { statuslineDelete, statuslineGet, statuslinePreview, statuslinePut } from '@/utils/ipc'
import type { SettingsScope, StatusLine } from '@/types/ipc'

export const useStatusline = (
  scope: MaybeRefOrGetter<SettingsScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.statusline.get(toValue(scope), toValue(workingDir))),
    queryFn: () => statuslineGet(toValue(scope), toValue(workingDir)),
  })

export const useStatuslinePut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      scope: SettingsScope
      statusLine: StatusLine
      workingDir?: string
      expectedMtimeMs?: number
    }) => statuslinePut(v.scope, v.statusLine, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.statusline.all })
      qc.invalidateQueries({ queryKey: qk.settings.all })
    },
  })
}

export const useStatuslineDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { scope: SettingsScope; workingDir?: string }) =>
      statuslineDelete(v.scope, v.workingDir),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.statusline.all })
      qc.invalidateQueries({ queryKey: qk.settings.all })
    },
  })
}

/** Runs the command on this machine, so it is a mutation and never a query. */
export const useStatuslinePreview = () =>
  useMutation({
    mutationFn: (v: { statusLine: StatusLine; workingDir?: string }) =>
      statuslinePreview(v.statusLine, v.workingDir),
  })
