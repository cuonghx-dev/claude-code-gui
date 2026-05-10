import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { skillsGet, skillsList } from '@/utils/ipc'

export const useSkillsList = () => useQuery({ queryKey: qk.skills.list(), queryFn: skillsList })

export const useSkill = (slug: MaybeRefOrGetter<string>) =>
  useQuery({
    queryKey: computed(() => qk.skills.get(toValue(slug))),
    queryFn: () => skillsGet(toValue(slug)),
    enabled: computed(() => !!toValue(slug)),
  })
