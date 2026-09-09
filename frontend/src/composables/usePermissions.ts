import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { permissionsEffective, permissionsGet, permissionsPut, permissionsValidate } from '@/utils/ipc'
import type { Permissions, SettingsScope } from '@/types/ipc'

export const usePermissions = (
  scope: MaybeRefOrGetter<SettingsScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.permissions.get(toValue(scope), toValue(workingDir))),
    queryFn: () => permissionsGet(toValue(scope), toValue(workingDir)),
  })

export const useEffectivePermissions = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.permissions.effective(toValue(workingDir))),
    queryFn: () => permissionsEffective(toValue(workingDir)),
  })

export const usePermissionsPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      scope: SettingsScope
      permissions: Permissions
      workingDir?: string
      expectedMtimeMs?: number
    }) => permissionsPut(v.scope, v.permissions, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.permissions.all })
      qc.invalidateQueries({ queryKey: qk.settings.all })
    },
  })
}

/** Rust is authoritative; this backs the inline feedback while typing. */
export const useValidatePermissions = () =>
  useMutation({ mutationFn: (permissions: Permissions) => permissionsValidate(permissions) })
