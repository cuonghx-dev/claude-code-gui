<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import SkillForm from '@/components/forms/SkillForm.vue'
import { useSkillCreate } from '@/composables/useSkills'
import type { SkillInput } from '@/types/ipc'

const router = useRouter()
const create = useSkillCreate()
const errorMessage = ref('')

async function onSubmit(input: SkillInput) {
  errorMessage.value = ''
  try {
    const s = await create.mutateAsync(input)
    router.push(`/skills/${encodeURIComponent(s.slug)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="New skill" subtitle="Create a skill directory under ~/.claude/skills/" />
  <section class="p-6">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <SkillForm
      draft-key="skill:new"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="router.push('/skills')"
    />
  </section>
</template>
