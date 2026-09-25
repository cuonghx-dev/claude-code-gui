<script setup lang="ts">
import { computed, ref } from 'vue'
import { ChevronRight, FileText, Folder, Link2 } from 'lucide-vue-next'
import { useClaudeDirectoryChildren } from '@/composables/useClaudeDirectory'
import type { ClaudeDirEntry } from '@/types/ipc'

const props = withDefaults(
  defineProps<{
    entry: ClaudeDirEntry
    projectPath?: string
    depth?: number
    /** Label to show instead of the entry's own, for catalog rows. */
    label?: string
    selectedPath?: string | null
  }>(),
  { depth: 0, selectedPath: null },
)

const emit = defineEmits<{ select: [ClaudeDirEntry] }>()

// Children load when the node opens, so the tree can go arbitrarily deep
// without any single request walking a whole subtree.
const open = ref(false)
const children = useClaudeDirectoryChildren(
  computed(() => (open.value ? props.entry.path : null)),
  () => props.projectPath,
)

const expandable = computed(
  () => props.entry.kind === 'dir' && props.entry.exists && !props.entry.isSymlink,
)

const onClick = () => {
  if (expandable.value) open.value = !open.value
  else emit('select', props.entry)
}

const isSelected = computed(() => !!props.selectedPath && props.selectedPath === props.entry.path)

const formatSize = (bytes: bigint | number | null) => {
  if (bytes === null) return ''
  const n = Number(bytes)
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <li>
    <button
      type="button"
      class="ccg-dir-row flex w-full items-center gap-1.5 border py-[5px] pr-2 text-left"
      :style="{ paddingLeft: `${8 + depth * 14}px` }"
      :class="[entry.exists ? '' : 'opacity-50', isSelected ? 'ccg-dir-row-selected' : '']"
      :aria-expanded="expandable ? open : undefined"
      :aria-current="isSelected ? 'true' : undefined"
      :title="entry.oneLiner || entry.path"
      @click="onClick"
    >
      <ChevronRight
        v-if="expandable"
        :size="14"
        :stroke-width="1.5"
        class="flex-none transition-transform"
        :class="open ? 'rotate-90' : ''"
        style="color: var(--ccg-muted-soft);"
      />
      <span v-else class="w-[14px] flex-none" />
      <component
        :is="entry.isSymlink ? Link2 : entry.kind === 'dir' ? Folder : FileText"
        :size="14"
        :stroke-width="1.5"
        class="flex-none"
        style="color: var(--ccg-subtle);"
      />
      <span class="min-w-0 flex-1 truncate font-mono text-[12.5px] text-ink">{{ label ?? entry.label }}</span>
      <span v-if="!entry.known" class="ccg-badge" style="font-size: 9.5px; padding: 0 4px;">extra</span>
      <span class="flex-none font-mono text-[10.5px]" style="color: var(--ccg-muted-soft);">
        <template v-if="!entry.exists">—</template>
        <template v-else-if="entry.kind === 'dir'">{{ entry.childCount }}</template>
        <template v-else>{{ formatSize(entry.sizeBytes) }}</template>
      </span>
    </button>

    <ul v-if="open" class="flex flex-col gap-px">
      <li
        v-if="children.isPending.value"
        class="py-1 text-[11.5px]"
        :style="{ paddingLeft: `${30 + (depth + 1) * 14}px`, color: 'var(--ccg-muted-soft)' }"
      >
        Loading…
      </li>
      <ClaudeDirNode
        v-for="child in children.data.value ?? []"
        :key="child.id"
        :entry="child"
        :project-path="projectPath"
        :selected-path="selectedPath"
        :depth="depth + 1"
        @select="(e: ClaudeDirEntry) => emit('select', e)"
      />
      <li
        v-if="children.data.value?.some((c) => c.truncated)"
        class="py-1 text-[11.5px]"
        :style="{ paddingLeft: `${30 + (depth + 1) * 14}px`, color: 'var(--ccg-muted-soft)' }"
      >
        Listing truncated.
      </li>
    </ul>
  </li>
</template>

<style scoped>
.ccg-dir-row {
  border-color: transparent;
  border-radius: 7px;
  transition: background-color 120ms ease-out, border-color 120ms ease-out;
}

.ccg-dir-row:hover {
  background: #efece5;
}

.ccg-dir-row-selected,
.ccg-dir-row-selected:hover {
  background: #fff;
  border-color: var(--ccg-hairline-strong);
}
</style>
