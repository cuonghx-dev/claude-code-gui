<script setup lang="ts">
import { reactive, ref, watchEffect } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import FormField from '@/components/forms/FormField.vue'
import {
  useClaudeCliInfo,
  useConfig,
  useConfigSet,
  useSettings,
  useSettingsPut,
} from '@/composables/useSettings'
import type { AppConfig, Settings } from '@/types/ipc'

const { isPending, isError, error, data: settings } = useSettings()
const { data: cli } = useClaudeCliInfo()
const { data: config } = useConfig()
const settingsMut = useSettingsPut()
const configMut = useConfigSet()

const sLocal = reactive({
  defaultModel: '' as string,
  defaultPermissionMode: '' as string,
})
const cLocal = reactive({
  theme: '' as string,
  claudeDirOverride: '' as string,
  updaterChannel: 'stable' as string,
})
const lastSaved = ref('')
const errorMessage = ref('')
const checkingUpdate = ref(false)

watchEffect(() => {
  const s = settings.value
  if (s) {
    sLocal.defaultModel = s.defaultModel ?? ''
    sLocal.defaultPermissionMode = s.defaultPermissionMode ?? ''
  }
  const c = config.value
  if (c) {
    cLocal.theme = c.theme ?? ''
    cLocal.claudeDirOverride = c.claudeDirOverride ?? ''
    cLocal.updaterChannel = c.updaterChannel ?? 'stable'
  }
})

async function saveSettings() {
  errorMessage.value = ''
  const next: Settings = {
    ...(settings.value ?? { extra: {} }),
    defaultModel: sLocal.defaultModel || null,
    defaultPermissionMode: sLocal.defaultPermissionMode || null,
  } as Settings
  try {
    await settingsMut.mutateAsync(next)
    lastSaved.value = 'settings.json updated'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function saveConfig() {
  errorMessage.value = ''
  const next: AppConfig = {
    ...(config.value ?? { experimentalHooksMetrics: false }),
    theme: cLocal.theme || null,
    claudeDirOverride: cLocal.claudeDirOverride || null,
    updaterChannel: cLocal.updaterChannel || null,
  } as AppConfig
  try {
    await configMut.mutateAsync(next)
    lastSaved.value = 'app config updated'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function checkForUpdates() {
  errorMessage.value = ''
  checkingUpdate.value = true
  try {
    const { check } = await import('@tauri-apps/plugin-updater')
    const update = await check()
    if (update?.available) {
      lastSaved.value = `update available: ${update.version}`
    } else {
      lastSaved.value = 'already up to date'
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  } finally {
    checkingUpdate.value = false
  }
}

async function redoOnboarding() {
  errorMessage.value = ''
  const next: Settings = {
    ...(settings.value ?? { extra: {} }),
    onboardingCompleted: false,
  } as Settings
  try {
    await settingsMut.mutateAsync(next)
    lastSaved.value = 'onboarding will replay on next reload'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="Settings" subtitle="Edit ~/.claude/settings.json and app preferences" />
  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>
  <p
    v-if="lastSaved"
    class="mx-6 mt-4 rounded-md border border-emerald-300 bg-emerald-50 p-3 text-sm text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200"
  >
    {{ lastSaved }}
  </p>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="settings">
    <template #default>
      <section class="p-6 space-y-6">
        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Claude CLI</h3>
          <dl class="mt-2 grid grid-cols-3 gap-x-3 gap-y-2 text-sm">
            <dt class="text-neutral-500">Path</dt>
            <dd class="col-span-2 break-all font-mono text-xs">{{ cli?.path ?? 'not found' }}</dd>
            <dt class="text-neutral-500">Version</dt>
            <dd class="col-span-2">{{ cli?.version ?? '—' }}</dd>
          </dl>
        </div>

        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">~/.claude/settings.json</h3>
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
          <div class="mt-3 flex items-center gap-2">
            <button type="button" class="ccg-btn-primary" :disabled="settingsMut.isPending.value" @click="saveSettings">
              {{ settingsMut.isPending.value ? 'Saving…' : 'Save settings.json' }}
            </button>
            <button type="button" class="ccg-btn-ghost" @click="redoOnboarding">
              Replay onboarding
            </button>
          </div>
        </div>

        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">App preferences</h3>
          <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2">
            <FormField label="Theme">
              <select v-model="cLocal.theme" class="ccg-input">
                <option value="">match system</option>
                <option value="light">light</option>
                <option value="dark">dark</option>
              </select>
            </FormField>
            <FormField label="Claude directory override" hint="Default: ~/.claude">
              <input v-model="cLocal.claudeDirOverride" type="text" class="ccg-input" />
            </FormField>
          </div>
          <button type="button" class="mt-3 ccg-btn-primary" :disabled="configMut.isPending.value" @click="saveConfig">
            {{ configMut.isPending.value ? 'Saving…' : 'Save preferences' }}
          </button>
        </div>

        <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Updater</h3>
          <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2">
            <FormField label="Channel">
              <select v-model="cLocal.updaterChannel" class="ccg-input">
                <option value="stable">stable</option>
                <option value="beta">beta</option>
              </select>
            </FormField>
          </div>
          <div class="mt-3 flex items-center gap-2">
            <button type="button" class="ccg-btn-primary" :disabled="configMut.isPending.value" @click="saveConfig">
              Save channel
            </button>
            <button
              type="button"
              class="ccg-btn-ghost"
              :disabled="checkingUpdate"
              @click="checkForUpdates"
            >
              {{ checkingUpdate ? 'Checking…' : 'Check for updates' }}
            </button>
          </div>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
