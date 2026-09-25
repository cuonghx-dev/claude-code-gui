<script setup lang="ts">
import { PERMISSION_MODES } from '@/lib/permissionModes'
import { reactive, ref, computed } from 'vue'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useSetupFinalize } from '@/composables/useSettings'
import type { SetupPayload } from '@/types/ipc'

const finalize = useSetupFinalize()
const errorMessage = ref('')
const step = ref(0)

const state = reactive({
  defaultModel: 'sonnet',
  defaultPermissionMode: 'default',
  theme: 'system',
  claudeDirOverride: '',
})

const totalSteps = 3
const canAdvance = computed(() => true)

async function pickDir() {
  const picked = await openDialog({ directory: true, multiple: false })
  if (typeof picked === 'string') state.claudeDirOverride = picked
}

async function complete() {
  errorMessage.value = ''
  const payload: SetupPayload = {
    defaultModel: state.defaultModel || null,
    defaultPermissionMode: state.defaultPermissionMode || null,
    theme: state.theme || null,
    claudeDirOverride: state.claudeDirOverride.trim() || null,
  } as SetupPayload
  try {
    await finalize.mutateAsync(payload)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-40 flex items-start justify-center pt-[90px]"
      style="background: rgba(31, 30, 27, .18);"
    >
      <div
        class="flex w-[560px] flex-col overflow-hidden bg-white"
        style="border-radius: 12px; box-shadow: 0 24px 60px rgba(40,30,20,.3), 0 0 0 1px rgba(0,0,0,.08);"
        role="dialog"
        aria-modal="true"
        aria-labelledby="onboarding-title"
      >
        <header class="flex flex-col gap-1 border-b px-5 pb-4 pt-5" style="border-color: var(--ccg-hairline-soft);">
          <div class="flex items-baseline gap-2">
            <h2 id="onboarding-title" class="flex-1 text-[17px] font-semibold text-ink" style="letter-spacing: -.01em;">
              Welcome to Claude Code GUI
            </h2>
            <span class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">
              {{ step + 1 }} / {{ totalSteps }}
            </span>
          </div>
          <p class="text-[13px]" style="color: var(--ccg-body);">
            A few questions to set up your defaults. You can change everything later in Settings.
          </p>
          <div
            class="mt-2 flex gap-1"
            role="progressbar"
            :aria-valuenow="step + 1"
            aria-valuemin="1"
            :aria-valuemax="totalSteps"
            :aria-label="`Step ${step + 1} of ${totalSteps}`"
          >
            <span
              v-for="i in totalSteps"
              :key="i"
              class="h-[3px] flex-1 rounded-full"
              :style="{ background: i - 1 <= step ? 'var(--ccg-ink)' : 'var(--ccg-surface-strong)' }"
            />
          </div>
        </header>

        <div class="flex min-h-[180px] flex-col gap-5 px-5 py-5">
          <p v-if="errorMessage" class="ccg-alert-error px-3 py-2 text-[12.5px]" role="alert">
            {{ errorMessage }}
          </p>

          <section v-if="step === 0" class="flex flex-col gap-2">
            <div class="flex items-baseline gap-2">
              <h3 class="text-[13px] font-semibold text-ink">Default model</h3>
              <span class="text-[12px]" style="color: var(--ccg-muted-soft);">used when an agent does not pin a model</span>
            </div>
            <div class="ccg-seg self-start" role="group" aria-label="Default model">
              <button
                v-for="m in ['opus', 'sonnet', 'haiku']"
                :key="m"
                type="button"
                class="font-mono"
                :aria-pressed="state.defaultModel === m"
                @click="state.defaultModel = m"
              >
                {{ m }}
              </button>
            </div>
          </section>

          <template v-else-if="step === 1">
            <section class="flex flex-col gap-2">
              <div class="flex items-baseline gap-2">
                <h3 class="text-[13px] font-semibold text-ink">Default permission mode</h3>
                <span class="text-[12px]" style="color: var(--ccg-muted-soft);">applied when launching a new terminal</span>
              </div>
              <select v-model="state.defaultPermissionMode" class="ccg-input w-[240px] font-mono text-[12px]">
                <option v-for="m in PERMISSION_MODES" :key="m" :value="m">{{ m }}</option>
              </select>
            </section>
            <section class="flex flex-col gap-2">
              <h3 class="text-[13px] font-semibold text-ink">Theme</h3>
              <div class="ccg-seg self-start" role="group" aria-label="Theme">
                <button
                  v-for="t in [
                    { v: 'system', label: 'Match system' },
                    { v: 'light', label: 'Light' },
                    { v: 'dark', label: 'Dark' },
                  ]"
                  :key="t.v"
                  type="button"
                  :aria-pressed="state.theme === t.v"
                  @click="state.theme = t.v"
                >
                  {{ t.label }}
                </button>
              </div>
            </section>
          </template>

          <section v-else class="flex flex-col gap-2">
            <div class="flex items-baseline gap-2">
              <h3 class="text-[13px] font-semibold text-ink">Claude directory</h3>
              <span class="text-[12px]" style="color: var(--ccg-muted-soft);">advanced</span>
            </div>
            <p class="text-[12.5px]" style="color: var(--ccg-muted);">
              Default: <code class="font-mono text-[12px] text-ink">~/.claude</code>. Override only if you keep agents elsewhere.
            </p>
            <div class="flex gap-2">
              <input v-model="state.claudeDirOverride" class="ccg-input flex-1 font-mono text-[12.5px]" placeholder="~/.claude" />
              <button type="button" class="ccg-btn-ghost" @click="pickDir">Browse…</button>
            </div>
          </section>
        </div>

        <footer
          class="flex items-center justify-between border-t px-5 py-3"
          style="border-color: var(--ccg-hairline-soft); background: var(--ccg-canvas-soft);"
        >
          <button v-if="step > 0" type="button" class="ccg-btn-ghost" @click="step -= 1">Back</button>
          <span v-else />
          <button
            v-if="step < totalSteps - 1"
            type="button"
            class="ccg-btn-primary"
            :disabled="!canAdvance"
            @click="step += 1"
          >
            Next
          </button>
          <button
            v-else
            type="button"
            class="ccg-btn-primary"
            :disabled="finalize.isPending.value"
            @click="complete"
          >
            {{ finalize.isPending.value ? 'Saving…' : 'Finish' }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>
