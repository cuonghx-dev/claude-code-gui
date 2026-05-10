<script setup lang="ts">
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
    <div class="fixed inset-0 z-40 flex items-center justify-center bg-neutral-100/95 backdrop-blur-sm dark:bg-neutral-950/95">
      <div class="w-[520px] rounded-xl border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <header class="mb-4">
          <h2 class="text-lg font-semibold">Welcome to Claude Code GUI</h2>
          <p class="mt-1 text-sm text-neutral-600 dark:text-neutral-400">
            A few questions to set up your defaults. You can change everything later in Settings.
          </p>
          <div class="mt-3 flex gap-1">
            <span
              v-for="i in totalSteps"
              :key="i"
              class="h-1 flex-1 rounded"
              :class="i - 1 <= step ? 'bg-violet-600' : 'bg-neutral-200 dark:bg-neutral-700'"
            />
          </div>
        </header>

        <p
          v-if="errorMessage"
          class="mb-3 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>

        <section v-if="step === 0" class="space-y-3">
          <h3 class="text-sm font-semibold">Default model</h3>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">
            Used when an agent does not pin a model.
          </p>
          <div class="flex flex-col gap-2">
            <label v-for="m in ['opus', 'sonnet', 'haiku']" :key="m" class="flex items-center gap-2">
              <input v-model="state.defaultModel" type="radio" :value="m" />
              <span class="text-sm capitalize">{{ m }}</span>
            </label>
          </div>
        </section>

        <section v-else-if="step === 1" class="space-y-3">
          <h3 class="text-sm font-semibold">Default permission mode</h3>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">
            Applied when launching a new terminal.
          </p>
          <select v-model="state.defaultPermissionMode" class="ccg-input">
            <option value="default">default</option>
            <option value="acceptEdits">acceptEdits</option>
            <option value="bypassPermissions">bypassPermissions</option>
            <option value="plan">plan</option>
          </select>
          <h3 class="mt-4 text-sm font-semibold">Theme</h3>
          <select v-model="state.theme" class="ccg-input">
            <option value="system">match system</option>
            <option value="light">light</option>
            <option value="dark">dark</option>
          </select>
        </section>

        <section v-else class="space-y-3">
          <h3 class="text-sm font-semibold">Claude directory (advanced)</h3>
          <p class="text-xs text-neutral-500 dark:text-neutral-400">
            Default: <code>~/.claude</code>. Override only if you keep agents elsewhere.
          </p>
          <div class="flex gap-2">
            <input v-model="state.claudeDirOverride" class="ccg-input flex-1" placeholder="~/.claude" />
            <button type="button" class="ccg-btn-ghost" @click="pickDir">Browse…</button>
          </div>
        </section>

        <footer class="mt-6 flex items-center justify-between">
          <button
            v-if="step > 0"
            type="button"
            class="ccg-btn-ghost"
            @click="step -= 1"
          >
            Back
          </button>
          <span v-else />
          <div class="flex items-center gap-2">
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
          </div>
        </footer>
      </div>
    </div>
  </Teleport>
</template>
