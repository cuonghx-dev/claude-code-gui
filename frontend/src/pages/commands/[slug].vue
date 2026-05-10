<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import CommandForm from '@/components/forms/CommandForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useCommand,
  useCommandDelete,
  useCommandUpdate,
} from '@/composables/useCommands'
import type { CommandInput } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useCommand(slug)
const update = useCommandUpdate()
const remove = useCommandDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)

async function onSubmit(input: CommandInput) {
  errorMessage.value = ''
  try {
    const next = await update.mutateAsync({ slug: slug.value, input })
    if (next.slug !== slug.value) {
      router.replace(`/commands/${encodeURIComponent(next.slug)}`)
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    router.push('/commands')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="`/${slug}`" :subtitle="data?.frontmatter?.description ?? undefined">
    <template #actions>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: cmd }">
      <section v-if="cmd" class="p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <CommandForm
          :draft-key="`command:${cmd.slug}`"
          :initial="{
            slug: cmd.slug,
            directory: cmd.directory,
            frontmatter: cmd.frontmatter,
            body: cmd.body,
          }"
          :submitting="update.isPending.value"
          submit-label="Save changes"
          @submit="onSubmit"
          @cancel="router.push('/commands')"
        />
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete command?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
