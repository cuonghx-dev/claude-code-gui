<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EditorPage from '@/components/EditorPage.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { usePlan, usePlanDelete, usePlanUpdate } from '@/composables/usePlans'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = usePlan(slug)
const update = usePlanUpdate()
const remove = usePlanDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const body = ref('')
const initial = ref('')

const dirty = computed(() => body.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  (p) => {
    if (!p) return
    body.value = p.body
    initial.value = p.body
  },
  { immediate: true },
)

async function onSave() {
  errorMessage.value = ''
  try {
    await update.mutateAsync({
      slug: slug.value,
      input: { slug: slug.value, body: body.value },
    })
    initial.value = body.value
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    initial.value = body.value
    router.push('/plans')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: plan }">
      <EditorPage
        v-if="plan"
        v-model="body"
        section="Plans"
        section-to="/plans"
        :name="plan.title"
        :dirty="dirty"
        :file-path="plan.filePath"
        :error="errorMessage"
        :frontmatter="false"
      >
        <template #actions>
          <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirmingDelete = true">Delete</button>
          <button
            type="button"
            class="ccg-btn-primary ccg-btn-sm"
            :disabled="!dirty || update.isPending.value"
            @click="onSave"
          >
            {{ update.isPending.value ? 'Saving…' : 'Save' }}
          </button>
        </template>
      </EditorPage>
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
