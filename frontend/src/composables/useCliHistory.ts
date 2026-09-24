import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { cliHistoryGet, cliHistoryList } from '@/utils/ipc'

export const useTerminalsList = () =>
  useQuery({ queryKey: qk.terminals.list(), queryFn: cliHistoryList })

/// Snapshots of terminals that ran a given Claude Code session, newest first.
/// Fresh launches pin `--session-id` to the PTY id and resumes record the
/// resumed id, so a session may have several replays.
export const useTerminalReplaysForSession = (sessionId: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: qk.terminals.list(),
    queryFn: cliHistoryList,
    select: (items) => items.filter((t) => t.claudeSessionId === toValue(sessionId)),
  })

export const useTerminalReplay = (id: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.terminals.get(toValue(id))),
    queryFn: () => cliHistoryGet(toValue(id)),
    enabled: computed(() => !!toValue(id)),
    // Snapshots are written once when a session exits and never change.
    staleTime: Infinity,
  })
