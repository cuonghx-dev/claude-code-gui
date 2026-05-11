<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import AgentForm from '@/components/forms/AgentForm.vue'
import { useAgentCreate } from '@/composables/useAgents'
import type { AgentInput } from '@/types/ipc'

const router = useRouter()
const create = useAgentCreate()
const errorMessage = ref<string>('')

async function onSubmit(input: AgentInput) {
  errorMessage.value = ''
  try {
    const agent = await create.mutateAsync(input)
    router.push(`/agents/${encodeURIComponent(agent.slug)}`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <PageHeader title="New agent" subtitle="Create a markdown agent under ~/.claude/agents/" />
    <section class="flex min-h-0 flex-1 flex-col p-6">
      <p
        v-if="errorMessage"
        class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ errorMessage }}
      </p>
      <AgentForm
        class="min-h-0 flex-1"
        draft-key="agent:new"
        :submitting="create.isPending.value"
        submit-label="Create"
        @submit="onSubmit"
        @cancel="router.push('/agents')"
      />
    </section>
  </div>
</template>
