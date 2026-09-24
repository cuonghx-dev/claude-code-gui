<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import McpForm from '@/components/forms/McpForm.vue'
import { useMcpCreate, useMcpScope } from '@/composables/useMcp'
import type { McpServerInput } from '@/types/ipc'

const router = useRouter()
const create = useMcpCreate()
const { scope, workingDir, ready, query, fileLabel } = useMcpScope()
const errorMessage = ref('')

async function onSubmit(input: McpServerInput) {
  errorMessage.value = ''
  if (!ready.value) {
    errorMessage.value = 'Choose a project on the MCP page first.'
    return
  }
  try {
    const s = await create.mutateAsync({ input, scope: scope.value, workingDir: workingDir.value })
    router.push({ path: `/mcp/${encodeURIComponent(s.name)}`, query: query.value })
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="New MCP server" :subtitle="`Adds an entry to ${fileLabel}`" />
  <section class="p-6">
    <p
      v-if="errorMessage"
      class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
    >
      {{ errorMessage }}
    </p>
    <McpForm
      :draft-key="`mcp:new:${scope}`"
      :submitting="create.isPending.value"
      submit-label="Create"
      @submit="onSubmit"
      @cancel="router.push({ path: '/mcp', query })"
    />
  </section>
</template>
