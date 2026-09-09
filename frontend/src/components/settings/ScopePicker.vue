<script setup lang="ts">
import { computed } from 'vue'
import type { ScopeInfo, SettingsScope } from '@/types/ipc'

const props = defineProps<{
  modelValue: SettingsScope
  scopes: ScopeInfo[]
  /** Project and local scopes need a project selected. */
  hasWorkingDir: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [SettingsScope] }>()

const LABEL: Record<SettingsScope, string> = {
  managed: 'Managed',
  local: 'Project local',
  project: 'Project',
  user: 'User',
}

const disabled = (s: SettingsScope) =>
  s === 'managed' || (s === 'project' && !props.hasWorkingDir)

const current = computed(() => props.scopes.find((s) => s.scope === props.modelValue))
</script>

<template>
  <div class="flex flex-col gap-1">
    <div class="inline-flex rounded-lg border border-neutral-200 p-0.5 dark:border-neutral-800">
      <button
        v-for="s in scopes"
        :key="s.scope"
        type="button"
        class="rounded px-3 py-1 text-xs"
        :class="[
          s.scope === modelValue
            ? 'bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900'
            : 'text-neutral-600 dark:text-neutral-300',
          disabled(s.scope) && s.scope !== modelValue ? 'opacity-40' : '',
        ]"
        :disabled="disabled(s.scope) && s.scope !== 'managed'"
        @click="emit('update:modelValue', s.scope)"
      >
        {{ LABEL[s.scope] }}
        <span v-if="!s.writable" class="ml-1 opacity-70">(read-only)</span>
      </button>
    </div>
    <p v-if="current" class="font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
      {{ current.path }}
      <span v-if="!current.exists"> · not created yet</span>
    </p>
  </div>
</template>
