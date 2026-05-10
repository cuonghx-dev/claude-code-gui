import { useQuery } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import { debugClaudeCli, settingsGet } from '@/utils/ipc'

export const useSettings = () => useQuery({ queryKey: qk.settings(), queryFn: settingsGet })

export const useClaudeCliInfo = () =>
  useQuery({ queryKey: qk.debug.claudeCli(), queryFn: debugClaudeCli })
