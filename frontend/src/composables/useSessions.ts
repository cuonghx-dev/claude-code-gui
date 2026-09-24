import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  sessionsDelete,
  sessionsListForProject,
  sessionsMessages,
  sessionsRename,
  sessionsThreadMessages,
  sessionsThreads,
} from '@/utils/ipc'

/** Messages fetched per page. Large enough that scrolling rarely waits. */
const PAGE_SIZE = 200

export const useSessionsForProject = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.sessions.listFor(toValue(name))),
    queryFn: () => sessionsListForProject(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

/**
 * Paginated transcript. The Rust side seeks to the page through a cached byte
 * index, so a deep page costs the same as an early one.
 */
export const useSessionMessages = (
  projectName: MaybeRefOrGetter<string>,
  sessionId: MaybeRefOrGetter<string>,
) =>
  useInfiniteQuery({
    queryKey: computed(() => qk.sessions.messages(toValue(sessionId))),
    initialPageParam: 0 as number,
    queryFn: ({ pageParam }) =>
      sessionsMessages(toValue(projectName), toValue(sessionId), pageParam, PAGE_SIZE),
    getNextPageParam: (last) => last.nextAfter ?? undefined,
    enabled: computed(() => !!toValue(sessionId) && !!toValue(projectName)),
  })

export const useSessionThreads = (
  projectName: MaybeRefOrGetter<string>,
  sessionId: MaybeRefOrGetter<string>,
) =>
  useQuery({
    queryKey: computed(() => qk.sessions.threads(toValue(sessionId))),
    queryFn: () => sessionsThreads(toValue(projectName), toValue(sessionId)),
    enabled: computed(() => !!toValue(sessionId) && !!toValue(projectName)),
  })

/** A subagent's own transcript, fetched only when its card is opened. */
export const useThreadMessages = (
  projectName: MaybeRefOrGetter<string>,
  sessionId: MaybeRefOrGetter<string>,
  threadId: MaybeRefOrGetter<string>,
  enabled: MaybeRefOrGetter<boolean>,
) =>
  useQuery({
    queryKey: computed(() => qk.sessions.threadMessages(toValue(sessionId), toValue(threadId))),
    queryFn: () =>
      sessionsThreadMessages(toValue(projectName), toValue(sessionId), toValue(threadId)),
    enabled: computed(() => toValue(enabled) && !!toValue(threadId)),
    staleTime: Infinity,
  })

export const useSessionRename = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { projectName: string; sessionId: string; newName: string }) =>
      sessionsRename(v.projectName, v.sessionId, v.newName),
    onSuccess: (_d, v) => qc.invalidateQueries({ queryKey: qk.sessions.listFor(v.projectName) }),
  })
}

export const useSessionDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { projectName: string; sessionId: string }) =>
      sessionsDelete(v.projectName, v.sessionId),
    onSuccess: (_d, v) => {
      qc.invalidateQueries({ queryKey: qk.sessions.listFor(v.projectName) })
      qc.removeQueries({ queryKey: qk.sessions.messages(v.sessionId) })
      qc.invalidateQueries({ queryKey: qk.projects.all })
    },
  })
}
