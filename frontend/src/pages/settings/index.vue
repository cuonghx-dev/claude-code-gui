<script setup lang="ts">
import { inject, reactive, ref, watchEffect } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import FormField from '@/components/forms/FormField.vue'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import {
  useClaudeCliInfo,
  useConfig,
  useConfigSet,
  useSettings,
  useSettingsPatch,
} from '@/composables/useSettings'
import type { AppConfig } from '@/types/ipc'

const ctx = inject(SETTINGS_CONTEXT)!
const { data: settings } = useSettings()
const { data: cli } = useClaudeCliInfo()
const { data: config } = useConfig()
const patch = useSettingsPatch()
const configMut = useConfigSet()

const sLocal = reactive({ defaultModel: '', defaultPermissionMode: '' })
const cLocal = reactive({ theme: '', claudeDirOverride: '', updaterChannel: 'stable' })
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

// A merge patch of just these two keys: everything else in the file is left
// exactly as the CLI (or a person) wrote it.
async function saveSettings() {
  errorMessage.value = ''
  try {
    await patch.mutateAsync({
      scope: ctx.scope.value,
      workingDir: ctx.workingDir.value,
      patch: {
        defaultModel: sLocal.defaultModel || null,
        defaultPermissionMode: sLocal.defaultPermissionMode || null,
      },
    })
    lastSaved.value = 'settings saved'
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
    lastSaved.value = 'app preferences saved'
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
    lastSaved.value = update?.available ? `update available: ${update.version}` : 'already up to date'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  } finally {
    checkingUpdate.value = false
  }
}

async function redoOnboarding() {
  errorMessage.value = ''
  try {
    await patch.mutateAsync({ scope: 'user', patch: { onboardingCompleted: false } })
    lastSaved.value = 'onboarding will replay on next reload'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <section class="space-y-6 p-6">
    <p v-if="errorMessage" class="rounded-md bg-red-500/10 p-3 text-sm text-red-700 dark:text-red-300">
      {{ errorMessage }}
    </p>
    <p v-if="lastSaved" class="rounded-md bg-emerald-500/10 p-3 text-sm text-emerald-700 dark:text-emerald-300">
      {{ lastSaved }}
    </p>

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
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">
        Session defaults
      </h3>
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
            <option value="plan">plan</option>
            <option value="auto">auto</option>
            <option value="dontAsk">dontAsk</option>
            <option value="bypassPermissions">bypassPermissions</option>
          </select>
        </FormField>
      </div>
      <div class="mt-3 flex items-center gap-2">
        <button
          type="button"
          class="ccg-btn-primary inline-flex items-center gap-1.5"
          :disabled="patch.isPending.value"
          @click="saveSettings"
        >
          <Loader2 v-if="patch.isPending.value" class="h-3.5 w-3.5 animate-spin" aria-hidden="true" />
          Save
        </button>
        <button type="button" class="ccg-btn-ghost" @click="redoOnboarding">
          Replay onboarding
        </button>
      </div>
    </div>

    <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
      <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">
        App preferences
      </h3>
      <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
        Stored by this app, not by Claude Code.
      </p>
      <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-3">
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
        <FormField label="Updater channel">
          <select v-model="cLocal.updaterChannel" class="ccg-input">
            <option value="stable">stable</option>
            <option value="beta">beta</option>
          </select>
        </FormField>
      </div>
      <div class="mt-3 flex items-center gap-2">
        <button
          type="button"
          class="ccg-btn-primary inline-flex items-center gap-1.5"
          :disabled="configMut.isPending.value"
          @click="saveConfig"
        >
          <Loader2 v-if="configMut.isPending.value" class="h-3.5 w-3.5 animate-spin" aria-hidden="true" />
          Save preferences
        </button>
        <button
          type="button"
          class="ccg-btn-ghost inline-flex items-center gap-1.5"
          :disabled="checkingUpdate"
          @click="checkForUpdates"
        >
          <Loader2 v-if="checkingUpdate" class="h-3.5 w-3.5 animate-spin" aria-hidden="true" />
          {{ checkingUpdate ? 'Checking…' : 'Check for updates' }}
        </button>
      </div>
    </div>
  </section>
</template>
