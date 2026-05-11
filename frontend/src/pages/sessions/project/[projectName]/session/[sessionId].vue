<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import ChatTerminal from '@/components/ChatTerminal.vue'
import { useProject } from '@/composables/useProjects'
import { useSettings } from '@/composables/useSettings'
import type { TerminalOpts } from '@/types/ipc'

const route = useRoute()
const projectName = computed(() => (route.params as { projectName: string }).projectName)
const sessionId = computed(() => (route.params as { sessionId: string }).sessionId)

const project = useProject(projectName)
const settings = useSettings()

const resuming = ref(false)

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
  <section class="flex min-h-0 flex-1 flex-col">
    <div
      v-if="!resuming"
      class="flex flex-1 flex-col items-center justify-center px-6 text-center"
    >
      <div
        class="mb-6 flex h-20 w-20 items-center justify-center rounded-2xl bg-amber-500/10 text-3xl"
      >
        <span class="font-mono text-amber-500">&gt;_</span>
      </div>
      <h2 class="text-2xl font-semibold text-neutral-900 dark:text-neutral-100">
        Claude Code CLI
      </h2>
      <p class="mt-3 max-w-md text-sm text-neutral-500 dark:text-neutral-400">
        Resume session <span class="font-mono">{{ sessionId }}</span> in a terminal to continue the
        conversation.
      </p>
      <button
        type="button"
        class="ccg-btn-primary mt-6 px-6 py-3 text-base"
        :disabled="!project.data.value"
        @click="resuming = true"
      >
        + Resume in terminal
      </button>
      <p class="mt-3 text-xs text-neutral-500 dark:text-neutral-400">
        Browse your project history in the left sidebar
      </p>
    </div>
    <ChatTerminal
      v-if="resuming && terminalOpts"
      :opts="terminalOpts"
      class="min-h-0 flex-1"
    />
  </section>
</template>
