import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import type { AppConfig, SetupPayload, Settings } from '@/types/ipc'
import {
  configGet,
  configSet,
  debugClaudeCli,
  settingsGet,
  settingsPut,
  setupFinalize,
} from '@/utils/ipc'

export const useSettings = () => useQuery({ queryKey: qk.settings(), queryFn: settingsGet })

export const useConfig = () => useQuery({ queryKey: qk.config(), queryFn: configGet })

export const useClaudeCliInfo = () =>
  useQuery({ queryKey: qk.debug.claudeCli(), queryFn: debugClaudeCli })

export const useSettingsPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (settings: Settings) => settingsPut(settings),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.settings() }),
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
      qc.invalidateQueries({ queryKey: qk.settings() })
      qc.invalidateQueries({ queryKey: qk.config() })
    },
  })
}
