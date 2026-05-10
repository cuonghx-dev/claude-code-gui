import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { projectsFiles, projectsGet, projectsList } from '@/utils/ipc'

export const useProjectsList = () =>
  useQuery({ queryKey: qk.projects.list(), queryFn: projectsList })

export const useProject = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.projects.get(toValue(name))),
    queryFn: () => projectsGet(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

export const useProjectFiles = (
  name: MaybeRefOrGetter<string>,
  subPath?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.projects.files(toValue(name), toValue(subPath))),
    queryFn: () => projectsFiles(toValue(name), toValue(subPath)),
    enabled: computed(() => !!toValue(name)),
  })
