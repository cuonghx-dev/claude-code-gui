<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import FormField from '@/components/forms/FormField.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import {
  useProject,
  useProjectClaudeMd,
  useProjectClaudeMdPut,
  useProjectDelete,
  useProjectRename,
  useProjectSettings,
  useProjectSettingsPut,
} from '@/composables/useProjects'
import type { Settings } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const projectName = computed(() => (route.params as { projectName: string }).projectName)

const project = useProject(projectName)
const settings = useProjectSettings(projectName)
const claudeMd = useProjectClaudeMd(projectName)
const settingsPut = useProjectSettingsPut()
const claudeMdPut = useProjectClaudeMdPut()
const rename = useProjectRename()
const remove = useProjectDelete()

const sLocal = ref({ defaultModel: '', defaultPermissionMode: '' })
const mdLocal = ref('')
const status = ref('')
const errorMessage = ref('')
const renaming = ref(false)
const renameTo = ref('')
const confirmingDelete = ref(false)

watchEffect(() => {
  const s = settings.data.value
  if (s) {
    sLocal.value = {
      defaultModel: s.defaultModel ?? '',
      defaultPermissionMode: s.defaultPermissionMode ?? '',
    }
  }
  if (typeof claudeMd.data.value === 'string') {
    mdLocal.value = claudeMd.data.value
  }
})

async function saveSettings() {
  errorMessage.value = ''
  const next: Settings = {
    ...(settings.data.value ?? { extra: {} }),
    defaultModel: sLocal.value.defaultModel || null,
    defaultPermissionMode: sLocal.value.defaultPermissionMode || null,
  } as Settings
  try {
    await settingsPut.mutateAsync({ name: projectName.value, settings: next })
    status.value = 'project settings.json updated'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function saveClaudeMd() {
  errorMessage.value = ''
  try {
    await claudeMdPut.mutateAsync({ name: projectName.value, content: mdLocal.value })
    status.value = 'CLAUDE.md updated'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function doRename() {
  if (!renameTo.value.trim()) {
    renaming.value = false
    return
  }
  errorMessage.value = ''
  try {
    const next = renameTo.value.trim()
    await rename.mutateAsync({ name: projectName.value, newName: next })
    renaming.value = false
    router.replace(`/sessions/project/${encodeURIComponent(next)}/settings`)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function doDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(projectName.value)
    router.push('/sessions')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader
    :title="project.data.value?.workingDir ?? projectName"
    subtitle="Project settings + CLAUDE.md editor"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="renaming = true; renameTo = projectName">
        Rename
      </button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">
        Delete
      </button>
    </template>
  </PageHeader>

  <section v-if="renaming" class="mx-6 mt-4 rounded-md border border-neutral-200 bg-white p-3 dark:border-neutral-800 dark:bg-neutral-900">
    <div class="flex items-center gap-2">
      <input
        v-model="renameTo"
        :placeholder="projectName"
        class="ccg-input flex-1 font-mono text-xs"
      />
      <button type="button" class="ccg-btn-primary" @click="doRename">Save</button>
      <button type="button" class="ccg-btn-ghost" @click="renaming = false">Cancel</button>
    </div>
  </section>
  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>
  <p
    v-if="status"
    class="mx-6 mt-4 rounded-md border border-emerald-300 bg-emerald-50 p-3 text-sm text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200"
  >
    {{ status }}
  </p>
  <QueryStateBoundary
    :is-pending="settings.isPending.value"
    :is-error="settings.isError.value"
    :error="settings.error.value"
    :data="settings.data.value"
  >
    <template #default>
      <section class="p-6 space-y-6">
        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Project settings.json</h3>
          <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2">
            <FormField label="Default model">
              <select v-model="sLocal.defaultModel" class="ccg-input">
                <option value="">— inherit —</option>
                <option value="opus">opus</option>
                <option value="sonnet">sonnet</option>
                <option value="haiku">haiku</option>
              </select>
            </FormField>
            <FormField label="Default permission mode">
              <select v-model="sLocal.defaultPermissionMode" class="ccg-input">
                <option value="">— inherit —</option>
                <option value="default">default</option>
                <option value="acceptEdits">acceptEdits</option>
                <option value="bypassPermissions">bypassPermissions</option>
                <option value="plan">plan</option>
              </select>
            </FormField>
          </div>
          <button
            type="button"
            class="mt-3 ccg-btn-primary"
            :disabled="settingsPut.isPending.value"
            @click="saveSettings"
          >
            {{ settingsPut.isPending.value ? 'Saving…' : 'Save settings' }}
          </button>
        </div>

        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">CLAUDE.md</h3>
          <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            Project conventions document. Loaded by the CLI on session start.
          </p>
          <div class="mt-3">
            <MarkdownEditor v-model="mdLocal" min-height="280px" />
          </div>
          <button
            type="button"
            class="mt-3 ccg-btn-primary"
            :disabled="claudeMdPut.isPending.value"
            @click="saveClaudeMd"
          >
            {{ claudeMdPut.isPending.value ? 'Saving…' : 'Save CLAUDE.md' }}
          </button>
        </div>
      </section>
    </template>
  </QueryStateBoundary>

  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete project entry?"
    :message="`This removes ~/.claude/projects/${projectName}/ permanently. The actual working directory is not touched.`"
    confirm-label="Delete"
    danger
    @confirm="doDelete"
  />
</template>
