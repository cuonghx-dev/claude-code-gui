import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { sessionsListForProject, sessionsMessages } from '@/utils/ipc'

export const useSessionsForProject = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.sessions.listFor(toValue(name))),
    queryFn: () => sessionsListForProject(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

export const useSessionMessages = (
  projectName: MaybeRefOrGetter<string>,
  sessionId: MaybeRefOrGetter<string>,
  afterIndex?: MaybeRefOrGetter<number | undefined>,
  limit?: MaybeRefOrGetter<number | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.sessions.messages(toValue(sessionId))),
    queryFn: () =>
      sessionsMessages(toValue(projectName), toValue(sessionId), toValue(afterIndex), toValue(limit)),
    enabled: computed(() => !!toValue(sessionId) && !!toValue(projectName)),
  })
