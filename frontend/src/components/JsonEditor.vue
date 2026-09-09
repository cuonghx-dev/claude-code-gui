<script setup lang="ts">
import { computed } from 'vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'

const props = withDefaults(
  defineProps<{
    modelValue: string
    disabled?: boolean
    minHeight?: string
    fill?: boolean
  }>(),
  { disabled: false, minHeight: '320px', fill: false },
)
const emit = defineEmits<{ 'update:modelValue': [string] }>()

// Claude Code reads these files as strict JSON — a comment or trailing comma is
// a syntax error it reports at startup — so the parse status is worth showing
// before a save rather than after.
const parseError = computed(() => {
  if (!props.modelValue.trim()) return null
  try {
    JSON.parse(props.modelValue)
    return null
  } catch (e) {
    return (e as Error).message
  }
})

const format = () => {
  try {
    emit('update:modelValue', `${JSON.stringify(JSON.parse(props.modelValue), null, 2)}\n`)
  } catch {
    // Formatting invalid JSON is not possible; the status line already says so.
  }
}
</script>

<template>
  <div class="flex min-h-0 flex-col gap-2" :class="{ 'flex-1': fill }">
    <MarkdownEditor
      :model-value="modelValue"
      language="json"
      :min-height="minHeight"
      :fill="fill"
      :class="{ 'pointer-events-none opacity-60': disabled, 'min-h-0 flex-1': fill }"
      @update:model-value="(v: string) => emit('update:modelValue', v)"
    />
    <div class="flex items-center gap-3 text-xs">
      <span v-if="parseError" class="text-red-600 dark:text-red-400">{{ parseError }}</span>
      <span v-else class="text-emerald-600 dark:text-emerald-400">Valid JSON</span>
      <button
        type="button"
        class="ccg-btn-ghost ml-auto text-xs"
        :disabled="!!parseError || disabled"
        @click="format"
      >
        Format
      </button>
    </div>
  </div>
</template>
