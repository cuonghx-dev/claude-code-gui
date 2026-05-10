<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import McpForm from '@/components/forms/McpForm.vue'
import { useMcpCreate } from '@/composables/useMcp'
import type { McpServerInput } from '@/types/ipc'

const router = useRouter()
const create = useMcpCreate()
const errorMessage = ref('')

async function onSubmit(input: McpServerInput) {
  errorMessage.value = ''
  try {
    const s = await create.mutateAsync({ input, scope: 'global' })
    router.push(`/mcp/${encodeURIComponent(s.name)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="New MCP server" subtitle="Adds an entry to ~/.claude/.mcp.json" />
  <section class="p-6">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <McpForm
      draft-key="mcp:new"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="router.push('/mcp')"
    />
  </section>
</template>
