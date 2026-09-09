import type { InjectionKey, Ref } from 'vue'
import type { SettingsScope } from '@/types/ipc'

/**
 * Scope and project selection, owned by the settings layout and read by every
 * tab under it.
 */
export interface SettingsContext {
  scope: Ref<SettingsScope>
  workingDir: Ref<string | undefined>
}

export const SETTINGS_CONTEXT: InjectionKey<SettingsContext> = Symbol('settings-context')
