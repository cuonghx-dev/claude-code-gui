import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import {
  skillsCreate,
  skillsDelete,
  skillsExport,
  skillsGet,
  skillsImport,
  skillsList,
  skillsUpdate,
} from '@/utils/ipc'
import type { SkillImportSource, SkillInput } from '@/types/ipc'

export const useSkillsList = () => useQuery({ queryKey: qk.skills.list(), queryFn: skillsList })

export const useSkill = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.skills.get(toValue(slug))),
    queryFn: () => skillsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })

export const useSkillCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: SkillInput) => skillsCreate(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.skills.all }),
  })
}

export const useSkillUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ slug, input }: { slug: string; input: SkillInput }) =>
      skillsUpdate(slug, input),
    onSuccess: (_d, { slug }) => {
      qc.invalidateQueries({ queryKey: qk.skills.all })
      qc.invalidateQueries({ queryKey: qk.skills.get(slug) })
    },
  })
}

export const useSkillDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => skillsDelete(slug),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.skills.all }),
  })
}

export const useSkillExport = () =>
  useMutation({ mutationFn: (slug: string) => skillsExport(slug) })

export const useSkillImport = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (source: SkillImportSource) => skillsImport(source),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.skills.all }),
  })
}
