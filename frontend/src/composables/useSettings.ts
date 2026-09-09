import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import type { AppConfig, SettingsScope, SetupPayload } from '@/types/ipc'
import {
  configGet,
  configSet,
  debugClaudeCli,
  settingsEffective,
  settingsGet,
  settingsPatch,
  settingsRawGet,
  settingsRawPut,
  settingsScopes,
  setupFinalize,
} from '@/utils/ipc'

export const useSettings = () => useQuery({ queryKey: qk.settings.typed(), queryFn: settingsGet })

export const useConfig = () => useQuery({ queryKey: qk.config(), queryFn: configGet })

export const useClaudeCliInfo = () =>
  useQuery({ queryKey: qk.debug.claudeCli(), queryFn: debugClaudeCli })

export const useSettingsScopes = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.settings.scopes(toValue(workingDir))),
    queryFn: () => settingsScopes(toValue(workingDir)),
  })

export const useSettingsRaw = (
  scope: MaybeRefOrGetter<SettingsScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.settings.raw(toValue(scope), toValue(workingDir))),
    queryFn: () => settingsRawGet(toValue(scope), toValue(workingDir)),
  })

export const useSettingsEffective = (workingDir?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.settings.effective(toValue(workingDir))),
    queryFn: () => settingsEffective(toValue(workingDir)),
  })

/**
 * Writes go through a merge patch carrying only the keys the caller owns, so a
 * save can never drop a setting this app does not model.
 */
export const useSettingsPatch = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      scope: SettingsScope
      patch: Record<string, unknown>
      workingDir?: string
      expectedMtimeMs?: number
    }) => settingsPatch(v.scope, v.patch, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.settings.all }),
  })
}

export const useSettingsRawPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      scope: SettingsScope
      content: string
      workingDir?: string
      expectedMtimeMs?: number
    }) => settingsRawPut(v.scope, v.content, v.workingDir, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.settings.all }),
  })
}

export const useConfigSet = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (config: AppConfig) => configSet(config),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.config() }),
  })
}

export const useSetupFinalize = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (payload: SetupPayload) => setupFinalize(payload),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.settings.all })
      qc.invalidateQueries({ queryKey: qk.config() })
    },
  })
}
