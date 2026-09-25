<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { checkFrontmatter } from '@/utils/frontmatter'

/**
 * Editor screen from the design handoff: breadcrumb toolbar, full-bleed
 * CodeMirror, optional right panel, and a 26px status bar. Pages own the
 * actions (Save, Delete, …) via the `actions` slot.
 */
const props = withDefaults(
  defineProps<{
    modelValue: string
    /** Breadcrumb root, e.g. "Agents". */
    section: string
    sectionTo: string
    /** Slug / name shown after the breadcrumb. */
    name: string
    /** Dot color before the name (agents). */
    color?: string
    dirty?: boolean
    filePath?: string
    language?: 'markdown' | 'yaml' | 'javascript' | 'json'
    /** Validate the leading YAML frontmatter in the status bar. */
    frontmatter?: boolean
    error?: string
  }>(),
  { language: 'markdown', frontmatter: true, dirty: false },
)
const emit = defineEmits<{ 'update:modelValue': [string] }>()

const cursor = ref({ line: 1, col: 1 })
const status = computed(() =>
  props.frontmatter && props.language === 'markdown' ? checkFrontmatter(props.modelValue) : null,
)
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div
      class="flex flex-none items-center gap-2.5 border-b px-5 py-3.5"
      style="border-color: var(--ccg-hairline-soft);"
    >
      <RouterLink
        :to="sectionTo"
        class="text-[13px] transition-colors hover:text-ink"
        style="color: var(--ccg-subtle);"
      >
        {{ section }}
      </RouterLink>
      <span style="color: #C9C3B7;">/</span>
      <span v-if="color" class="h-[9px] w-[9px] flex-none rounded-full" :style="{ background: color }" />
      <span class="truncate font-mono text-[14px] font-medium text-ink">{{ name }}</span>
      <span
        v-if="dirty"
        class="flex flex-none items-center gap-[5px] text-[12px]"
        style="color: var(--ccg-accent);"
      >
        <span class="h-1.5 w-1.5 rounded-full" style="background: var(--ccg-accent);" />Unsaved
      </span>
      <div class="flex-1" />
      <slot name="actions" />
    </div>
    <p v-if="error" class="ccg-alert-error mx-5 mt-3 flex-none px-3 py-2 text-[12.5px]" role="alert">
      {{ error }}
    </p>
    <slot name="banner" />
    <div class="flex min-h-0 flex-1">
      <MarkdownEditor
        :model-value="modelValue"
        :language="language"
        fill
        flush
        class="min-h-0 min-w-0 flex-1"
        @update:model-value="emit('update:modelValue', $event)"
        @cursor="cursor = $event"
      />
      <slot name="panel" />
    </div>
    <div
      class="flex h-[26px] flex-none items-center gap-4 border-t px-4 font-mono text-[11px]"
      style="border-color: var(--ccg-hairline-soft); background: #F7F5F0; color: var(--ccg-subtle);"
    >
      <template v-if="status">
        <span v-if="status.kind === 'valid'" style="color: var(--ccg-success);">✓ Valid YAML</span>
        <span
          v-else-if="status.kind === 'error'"
          class="truncate"
          style="color: var(--ccg-error);"
          :title="status.message"
        >✕ {{ status.line ? `Line ${status.line}: ` : '' }}{{ status.message }}</span>
        <span v-else>No frontmatter</span>
      </template>
      <span class="flex-none">Ln {{ cursor.line }}, Col {{ cursor.col }}</span>
      <slot name="status" />
      <span class="flex-1" />
      <span v-if="filePath" class="truncate">{{ filePath }}</span>
    </div>
  </div>
</template>
