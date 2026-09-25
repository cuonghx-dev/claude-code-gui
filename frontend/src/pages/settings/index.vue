<script setup lang="ts">
import { PERMISSION_MODES } from '@/lib/permissionModes'
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
import type { AppConfig, UpdateInfo } from '@/types/ipc'
import { updaterCheck, updaterInstall } from '@/utils/ipc'

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
const installingUpdate = ref(false)
const availableUpdate = ref<UpdateInfo | null>(null)

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
  availableUpdate.value = null
  try {
    // Checks the saved channel's endpoint; save the channel first to switch.
    availableUpdate.value = await updaterCheck()
    lastSaved.value = availableUpdate.value
      ? `update available: ${availableUpdate.value.version}`
      : 'already up to date'
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  } finally {
    checkingUpdate.value = false
  }
}

async function installUpdate() {
  errorMessage.value = ''
  installingUpdate.value = true
  try {
    await updaterInstall()
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
    installingUpdate.value = false
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
  <section class="flex max-w-[820px] flex-col gap-[18px] px-7 py-[22px]">
    <p v-if="errorMessage" class="ccg-alert-error px-3 py-2 text-[12.5px]">
      {{ errorMessage }}
    </p>
    <p v-if="lastSaved" class="rounded-[7px] bg-[#DDEBDF] px-3 py-2 text-[12.5px] text-[#3F7A4E]">
      {{ lastSaved }}
    </p>

    <div class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">Claude CLI</span>
        <span class="text-[12px] text-[#A29E94]">The binary sessions and terminals launch</span>
      </div>
      <dl class="rounded-lg border border-[#E6E2DA] bg-white text-[12.5px]">
        <div class="flex items-center gap-3 border-b border-[#F3F0EA] px-3 py-2">
          <dt class="w-20 shrink-0 text-[#8A867C]">Path</dt>
          <dd class="min-w-0 flex-1 break-all font-mono text-ink">{{ cli?.path ?? 'not found' }}</dd>
        </div>
        <div class="flex items-center gap-3 px-3 py-2">
          <dt class="w-20 shrink-0 text-[#8A867C]">Version</dt>
          <dd class="font-mono text-ink">{{ cli?.version ?? '—' }}</dd>
        </div>
      </dl>
    </div>

    <div class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">Session defaults</span>
        <span class="text-[12px] text-[#A29E94]">Written to the selected scope's settings file</span>
      </div>
      <div class="flex flex-col gap-4 rounded-lg border border-[#E6E2DA] bg-white p-4">
        <div class="grid grid-cols-2 gap-3">
          <FormField label="Default model">
            <select v-model="sLocal.defaultModel" class="ccg-input font-mono text-[12.5px]">
              <option value="">— inherit —</option>
              <option value="opus">opus</option>
              <option value="sonnet">sonnet</option>
              <option value="haiku">haiku</option>
            </select>
          </FormField>
          <FormField label="Default permission mode">
            <select v-model="sLocal.defaultPermissionMode" class="ccg-input font-mono text-[12.5px]">
              <option value="">— inherit —</option>
              <option v-for="m in PERMISSION_MODES" :key="m" :value="m">{{ m }}</option>
            </select>
          </FormField>
        </div>
        <div class="flex items-center justify-end gap-2">
          <button type="button" class="ccg-btn-ghost" @click="redoOnboarding">
            Replay onboarding
          </button>
          <button
            type="button"
            class="ccg-btn-primary"
            :disabled="patch.isPending.value"
            @click="saveSettings"
          >
            <Loader2 v-if="patch.isPending.value" :size="14" class="animate-spin" aria-hidden="true" />
            Save
          </button>
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-2">
      <div class="flex items-baseline gap-2">
        <span class="text-[13px] font-semibold text-ink">App preferences</span>
        <span class="text-[12px] text-[#A29E94]">Stored by this app, not by Claude Code</span>
      </div>
      <div class="flex flex-col gap-4 rounded-lg border border-[#E6E2DA] bg-white p-4">
        <div class="grid grid-cols-3 gap-3">
          <FormField label="Theme">
            <select v-model="cLocal.theme" class="ccg-input">
              <option value="">match system</option>
              <option value="light">light</option>
              <option value="dark">dark</option>
            </select>
          </FormField>
          <FormField label="Claude directory override" hint="Default: ~/.claude">
            <input v-model="cLocal.claudeDirOverride" type="text" class="ccg-input font-mono text-[12.5px]" />
          </FormField>
          <FormField label="Updater channel">
            <select v-model="cLocal.updaterChannel" class="ccg-input">
              <option value="stable">stable</option>
              <option value="beta">beta</option>
            </select>
          </FormField>
        </div>
        <p
          v-if="availableUpdate?.notes"
          class="whitespace-pre-wrap text-[12px] text-[#8A867C]"
        >
          {{ availableUpdate.notes }}
        </p>
        <div class="flex items-center justify-end gap-2">
          <button
            type="button"
            class="ccg-btn-ghost"
            :disabled="checkingUpdate"
            @click="checkForUpdates"
          >
            <Loader2 v-if="checkingUpdate" :size="14" class="animate-spin" aria-hidden="true" />
            {{ checkingUpdate ? 'Checking…' : 'Check for updates' }}
          </button>
          <button
            v-if="availableUpdate"
            type="button"
            class="ccg-btn-ghost"
            :disabled="installingUpdate"
            @click="installUpdate"
          >
            <Loader2 v-if="installingUpdate" :size="14" class="animate-spin" aria-hidden="true" />
            {{ installingUpdate ? 'Installing…' : `Install ${availableUpdate.version} and restart` }}
          </button>
          <button
            type="button"
            class="ccg-btn-primary"
            :disabled="configMut.isPending.value"
            @click="saveConfig"
          >
            <Loader2 v-if="configMut.isPending.value" :size="14" class="animate-spin" aria-hidden="true" />
            Save preferences
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
