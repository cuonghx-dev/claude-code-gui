import { useInfiniteQuery, useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { sessionsListForProject, sessionsMessages, sessionsThreads } from '@/utils/ipc'

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
