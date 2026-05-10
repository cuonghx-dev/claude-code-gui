<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import PlanForm from '@/components/forms/PlanForm.vue'
import { usePlanCreate } from '@/composables/usePlans'
import type { PlanInput } from '@/types/ipc'

const router = useRouter()
const create = usePlanCreate()
const errorMessage = ref('')

async function onSubmit(input: PlanInput) {
  errorMessage.value = ''
  try {
    const p = await create.mutateAsync(input)
    router.push(`/plans/${encodeURIComponent(p.slug)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="New plan" subtitle="Plain markdown under ~/.claude/plans/" />
  <section class="p-6">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <PlanForm
      draft-key="plan:new"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="router.push('/plans')"
    />
  </section>
</template>
