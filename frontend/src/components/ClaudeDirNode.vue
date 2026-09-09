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
  }>(),
  { depth: 0 },
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
      class="flex w-full items-start gap-2 py-2 pr-4 text-left hover:bg-neutral-50 dark:hover:bg-neutral-800"
      :style="{ paddingLeft: `${16 + depth * 16}px` }"
      :class="entry.exists ? '' : 'opacity-50'"
      :aria-expanded="expandable ? open : undefined"
      @click="onClick"
    >
      <component
        :is="entry.isSymlink ? Link2 : entry.kind === 'dir' ? Folder : FileText"
        class="mt-0.5 h-4 w-4 shrink-0 text-neutral-500 dark:text-neutral-400"
      />
      <span class="min-w-0 flex-1">
        <span class="flex flex-wrap items-baseline gap-2">
          <span class="font-mono text-sm text-neutral-900 dark:text-neutral-100">
            {{ label ?? entry.label }}
          </span>
          <span
            v-if="!entry.known"
            class="rounded bg-neutral-500/10 px-1.5 py-0.5 text-[10px] text-neutral-500 dark:text-neutral-400"
          >
            undocumented
          </span>
          <span v-if="!entry.exists" class="text-[11px] text-neutral-500 dark:text-neutral-400">
            not present
          </span>
          <span v-else-if="entry.kind === 'dir'" class="text-[11px] text-neutral-500 dark:text-neutral-400">
            {{ entry.childCount }} item{{ entry.childCount === 1 ? '' : 's' }}
          </span>
          <span v-else class="text-[11px] text-neutral-500 dark:text-neutral-400">
            {{ formatSize(entry.sizeBytes) }}
          </span>
        </span>
        <span v-if="entry.oneLiner" class="mt-0.5 block text-xs text-neutral-500 dark:text-neutral-400">
          {{ entry.oneLiner }}
        </span>
      </span>
      <ChevronRight
        v-if="expandable"
        class="mt-0.5 h-4 w-4 shrink-0 text-neutral-400 transition-transform"
        :class="open ? 'rotate-90' : ''"
      />
    </button>

    <ul v-if="open" class="bg-neutral-50/60 dark:bg-neutral-950/40">
      <li v-if="children.isPending.value" class="py-1.5 pl-12 text-xs text-neutral-500">
        Loading…
      </li>
      <ClaudeDirNode
        v-for="child in children.data.value ?? []"
        :key="child.id"
        :entry="child"
        :project-path="projectPath"
        :depth="depth + 1"
        @select="(e: ClaudeDirEntry) => emit('select', e)"
      />
      <li
        v-if="children.data.value?.some((c) => c.truncated)"
        class="py-1.5 pl-12 text-xs text-neutral-500 dark:text-neutral-400"
      >
        Listing truncated.
      </li>
    </ul>
  </li>
</template>
