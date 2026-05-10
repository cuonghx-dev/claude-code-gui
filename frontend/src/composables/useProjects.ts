import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import type { Settings } from '@/types/ipc'
import {
  projectsClaudeMdGet,
  projectsClaudeMdPut,
  projectsCreate,
  projectsDelete,
  projectsFiles,
  projectsGet,
  projectsGitStatus,
  projectsList,
  projectsRename,
  projectsSettingsGet,
  projectsSettingsPut,
} from '@/utils/ipc'

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

export const useProjectGitStatus = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.projects.gitStatus(toValue(name))),
    queryFn: () => projectsGitStatus(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

export const useProjectSettings = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.projects.settings(toValue(name))),
    queryFn: () => projectsSettingsGet(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

export const useProjectClaudeMd = (name: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.projects.claudeMd(toValue(name))),
    queryFn: () => projectsClaudeMdGet(toValue(name)),
    enabled: computed(() => !!toValue(name)),
  })

export const useProjectCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (path: string) => projectsCreate(path),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.projects.all }),
  })
}

export const useProjectRename = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, newName }: { name: string; newName: string }) =>
      projectsRename(name, newName),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.projects.all }),
  })
}

export const useProjectDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => projectsDelete(name),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.projects.all }),
  })
}

export const useProjectSettingsPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, settings }: { name: string; settings: Settings }) =>
      projectsSettingsPut(name, settings),
    onSuccess: (_d, { name }) =>
      qc.invalidateQueries({ queryKey: qk.projects.settings(name) }),
  })
}

export const useProjectClaudeMdPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, content }: { name: string; content: string }) =>
      projectsClaudeMdPut(name, content),
    onSuccess: (_d, { name }) =>
      qc.invalidateQueries({ queryKey: qk.projects.claudeMd(name) }),
  })
}
