import { invoke } from '@tauri-apps/api/core'

// Mirror of TerminalOpts (will be replaced by ts-rs generated type once
// crates/core/src/types/terminal.rs lands in Phase 4).
export interface TerminalOpts {
  agentSlug?: string
  workingDir?: string
  cols: number
  rows: number
  model?: string
  permissionMode?: 'default' | 'acceptEdits' | 'bypassPermissions' | 'plan'
  outputStyleId?: string
  resumeSessionId?: string
  commandTemplate?: string
}

/**
 * Defensive wrapper. The terminal subsystem is strictly a Claude wrapper
 * (SPEC §6 invariant): every PTY must launch `claude` with a target.
 *
 * **Direct `invoke('terminal_session_create', …)` calls are forbidden** —
 * use this wrapper at every call site. A custom ESLint rule (or pre-commit
 * grep) enforces.
 */
export async function createTerminal(opts: TerminalOpts): Promise<string> {
  if (!opts.agentSlug && !opts.resumeSessionId && !opts.commandTemplate) {
    throw new Error(
      'createTerminal requires agentSlug, resumeSessionId, or commandTemplate. ' +
        'The terminal subsystem is a Claude wrapper; bare shells are not supported.',
    )
  }
  return invoke<string>('terminal_session_create', { opts })
}
