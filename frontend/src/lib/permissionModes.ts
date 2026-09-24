import type { PermissionMode } from '@/types/ipc'

/** `claude --permission-mode` choices, in the order the pickers show them. */
export const PERMISSION_MODES: readonly PermissionMode[] = [
  'default',
  'acceptEdits',
  'plan',
  'auto',
  'dontAsk',
  'manual',
  'bypassPermissions',
]

/** Narrow a settings value to a mode the terminal can pass, or `null` to let the CLI decide. */
export function asPermissionMode(value: unknown): PermissionMode | null {
  return PERMISSION_MODES.includes(value as PermissionMode) ? (value as PermissionMode) : null
}
