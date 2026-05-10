<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import CommandForm from '@/components/forms/CommandForm.vue'
import { useCommandCreate } from '@/composables/useCommands'
import type { CommandInput } from '@/types/ipc'

const router = useRouter()
const create = useCommandCreate()
const errorMessage = ref('')

async function onSubmit(input: CommandInput) {
  errorMessage.value = ''
  try {
    const c = await create.mutateAsync(input)
    router.push(`/commands/${encodeURIComponent(c.slug)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="New command" subtitle="Create a slash command under ~/.claude/commands/" />
  <section class="p-6">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <CommandForm
      draft-key="command:new"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="router.push('/commands')"
    />
  </section>
</template>
