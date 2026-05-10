<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import PlanForm from '@/components/forms/PlanForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { usePlan, usePlanDelete, usePlanUpdate } from '@/composables/usePlans'
import type { PlanInput } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = usePlan(slug)
const update = usePlanUpdate()
const remove = usePlanDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)

async function onSubmit(input: PlanInput) {
  errorMessage.value = ''
  try {
    const next = await update.mutateAsync({ slug: slug.value, input })
    if (next.slug !== slug.value) {
      router.replace(`/plans/${encodeURIComponent(next.slug)}`)
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    router.push('/plans')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="data?.title ?? slug" :subtitle="data?.filename">
    <template #actions>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: plan }">
      <section v-if="plan" class="p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <PlanForm
          :draft-key="`plan:${plan.slug}`"
          :initial="{ slug: plan.slug, body: plan.body }"
          :submitting="update.isPending.value"
          submit-label="Save changes"
          @submit="onSubmit"
          @cancel="router.push('/plans')"
        />
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete plan?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
