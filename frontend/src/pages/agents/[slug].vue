<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import AgentForm from '@/components/forms/AgentForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ChatTerminal from '@/components/ChatTerminal.vue'
import ContextPanel from '@/components/ContextPanel.vue'
import RelationshipGraph from '@/components/RelationshipGraph.vue'
import {
  useAgent,
  useAgentDelete,
  useAgentExport,
  useAgentUpdate,
} from '@/composables/useAgents'
import { useRelationshipsGraph } from '@/composables/useRelationships'
import { useSettings } from '@/composables/useSettings'
import type { AgentInput, TerminalOpts } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useAgent(slug)
const update = useAgentUpdate()
const remove = useAgentDelete()
const exportMut = useAgentExport()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const showTerminal = ref(false)
const showRelationships = ref(false)
const terminalSessionId = ref<string>('')
const settings = useSettings()
const relationships = useRelationshipsGraph()

const skillCount = computed(
  () => relationships.data.value?.agentSkills[slug.value]?.length ?? 0,
)
const commandCount = computed(
  () => relationships.data.value?.agentCommands[slug.value]?.length ?? 0,
)

const terminalOpts = computed<TerminalOpts | null>(() => {
  if (!showTerminal.value || !data.value) return null
  return {
    agentSlug: data.value.slug,
    cols: 100,
    rows: 32,
    workingDir: null,
    model: data.value.frontmatter.model ?? null,
    permissionMode: (settings.data.value?.defaultPermissionMode as TerminalOpts['permissionMode']) ?? null,
    outputStyleId: null,
    resumeSessionId: null,
    commandTemplate: null,
  } as TerminalOpts
})

async function onSubmit(input: AgentInput) {
  errorMessage.value = ''
  try {
    const next = await update.mutateAsync({ slug: slug.value, input })
    if (next.slug !== slug.value) {
      router.replace(`/agents/${encodeURIComponent(next.slug)}`)
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    router.push('/agents')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onExport() {
  errorMessage.value = ''
  try {
    const raw = await exportMut.mutateAsync(slug.value)
    const blob = new Blob([raw], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${slug.value}.md`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader
    :title="data?.frontmatter?.name ?? slug"
    :subtitle="`${data?.filePath ?? ''} · used by ${commandCount} command${commandCount === 1 ? '' : 's'} · has ${skillCount} skill${skillCount === 1 ? '' : 's'}`"
  >
    <template #actions>
      <button
        type="button"
        class="ccg-btn-ghost"
        @click="showRelationships = !showRelationships"
      >
        {{ showRelationships ? 'Hide graph' : 'Show graph' }}
      </button>
      <button
        type="button"
        class="ccg-btn-ghost"
        @click="showTerminal = !showTerminal"
      >
        {{ showTerminal ? 'Hide terminal' : 'Test in terminal' }}
      </button>
      <button type="button" class="ccg-btn-ghost" @click="onExport">Export</button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: agent }">
      <section v-if="agent" class="grid h-[calc(100vh-65px)] grid-cols-1 gap-0" :class="showTerminal ? 'lg:grid-cols-2' : ''">
        <div class="overflow-auto p-6">
          <p
            v-if="errorMessage"
            class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
          >
            {{ errorMessage }}
          </p>
          <section
            v-if="showRelationships"
            class="mb-6 rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900"
          >
            <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Relationships</h3>
            <RelationshipGraph :agent-slug="agent.slug" />
          </section>
          <AgentForm
            :draft-key="`agent:${agent.slug}`"
            :initial="{
              slug: agent.slug,
              directory: agent.directory,
              frontmatter: agent.frontmatter,
              body: agent.body,
            }"
            :submitting="update.isPending.value"
            submit-label="Save changes"
            @submit="onSubmit"
            @cancel="router.push('/agents')"
          />
        </div>
        <div v-if="showTerminal && terminalOpts" class="grid grid-rows-[1fr_auto] border-l border-neutral-200 dark:border-neutral-800">
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
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete agent?"
    :message="`This will permanently remove '${slug}.md' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
