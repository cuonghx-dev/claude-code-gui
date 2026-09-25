<script setup lang="ts">
import { computed, ref } from 'vue'
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

const cursor = ref({ line: 1, col: 1 })

const format = () => {
  try {
    emit('update:modelValue', `${JSON.stringify(JSON.parse(props.modelValue), null, 2)}\n`)
  } catch {
    // Formatting invalid JSON is not possible; the status line already says so.
  }
}
</script>

<template>
  <div class="flex min-h-0 flex-col gap-2.5" :class="{ 'flex-1': fill }">
    <MarkdownEditor
      :model-value="modelValue"
      language="json"
      :min-height="minHeight"
      :fill="fill"
      :class="{ 'pointer-events-none opacity-60': disabled, 'min-h-0 flex-1': fill }"
      @update:model-value="(v: string) => emit('update:modelValue', v)"
      @cursor="(c) => (cursor = c)"
    />
    <div class="flex items-center gap-3">
      <span
        class="min-w-0 truncate font-mono text-[11.5px]"
        :class="parseError ? 'text-[#B03A2E]' : 'text-[#3F7A4E]'"
        :title="parseError ?? undefined"
      >
        {{ parseError ?? 'Valid JSON' }}
      </span>
      <span class="shrink-0 font-mono text-[11.5px] text-[#A29E94]">
        Ln {{ cursor.line }}, Col {{ cursor.col }}
      </span>
      <span class="flex-1" />
      <button
        type="button"
        class="ccg-btn-ghost ccg-btn-sm"
        :disabled="!!parseError || disabled"
        @click="format"
      >
        Format
      </button>
      <slot name="actions" />
    </div>
  </div>
</template>
