<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ChatTerminal from '@/components/ChatTerminal.vue'
import ContextPanel from '@/components/ContextPanel.vue'
import { useSessionMessages } from '@/composables/useSessions'
import { useProject } from '@/composables/useProjects'
import { useSettings } from '@/composables/useSettings'
import type { TerminalOpts } from '@/types/ipc'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const { isPending, isError, error, data } = useSessionMessages(projectName, sessionId, undefined, 200)
const project = useProject(projectName)
const settings = useSettings()

const resuming = ref(false)
const terminalSessionId = ref('')

const terminalOpts = computed<TerminalOpts | null>(() => {
  if (!resuming.value || !project.data.value) return null
  return {
    agentSlug: null,
    cols: 100,
    rows: 32,
    workingDir: project.data.value.workingDir,
    model: null,
    permissionMode: (settings.data.value?.defaultPermissionMode as TerminalOpts['permissionMode']) ?? null,
    outputStyleId: null,
    resumeSessionId: sessionId.value,
    commandTemplate: null,
  } as TerminalOpts
})
</script>

<template>
  <PageHeader
    :title="`Session ${sessionId.slice(0, 12)}…`"
    :subtitle="`${data?.total ?? 0} messages (showing first 200)`"
  >
    <template #actions>
      <button
        type="button"
        class="ccg-btn-primary"
        :disabled="!project.data.value"
        @click="resuming = !resuming"
      >
        {{ resuming ? 'Hide terminal' : 'Resume in terminal' }}
      </button>
    </template>
  </PageHeader>
  <section class="grid h-[calc(100vh-65px)] grid-cols-1" :class="resuming ? 'lg:grid-cols-2' : ''">
    <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
      <template #default="{ data: page }">
        <div v-if="page" class="overflow-auto p-6 space-y-3">
          <div
            v-for="m in page.items"
            :key="m.id || Math.random()"
            class="rounded-md border border-neutral-200 bg-white p-3 text-sm dark:border-neutral-800 dark:bg-neutral-900"
          >
            <div class="mb-1 flex items-baseline gap-2 text-[11px] text-neutral-400">
              <span class="rounded bg-neutral-100 px-1.5 py-0.5 uppercase tracking-wide dark:bg-neutral-800">{{ m.kind }}</span>
              <span v-if="m.role">{{ m.role }}</span>
              <span v-if="m.timestamp" class="ml-auto">{{ m.timestamp }}</span>
            </div>
            <pre v-if="m.content" class="whitespace-pre-wrap break-words text-sm text-neutral-800 dark:text-neutral-200">{{ m.content }}</pre>
            <pre v-else-if="m.toolInput" class="whitespace-pre-wrap break-words font-mono text-xs text-neutral-600 dark:text-neutral-400">tool_input: {{ JSON.stringify(m.toolInput, null, 2) }}</pre>
          </div>
        </div>
      </template>
    </QueryStateBoundary>
    <div
      v-if="resuming && terminalOpts"
      class="grid grid-rows-[1fr_auto] border-l border-neutral-200 dark:border-neutral-800"
    >
      <ChatTerminal
        :opts="terminalOpts"
        class="min-h-0"
        @ready="(id: string) => (terminalSessionId = id)"
        @exit="() => (terminalSessionId = '')"
      />
      <div class="border-t border-neutral-200 p-3 dark:border-neutral-800">
        <ContextPanel :session-id="terminalSessionId || undefined" />
      </div>
    </div>
  </section>
</template>
