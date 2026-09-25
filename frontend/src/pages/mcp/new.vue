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
  <div class="flex h-full flex-col">
    <PageHeader title="New MCP server" :subtitle="fileLabel" />
    <section class="max-w-[640px] px-7 py-5">
      <p v-if="errorMessage" class="ccg-alert-error mb-4 px-3 py-2 text-[12.5px]" role="alert">
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
  </div>
</template>
