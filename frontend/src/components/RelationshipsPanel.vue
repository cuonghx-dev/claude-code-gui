<script setup lang="ts">
import RelationshipGraph from '@/components/RelationshipGraph.vue'

defineProps<{
  agentSlug?: string
  commandSlug?: string
  skillSlug?: string
}>()
</script>

<template>
  <aside
    class="flex w-80 flex-none flex-col overflow-y-auto border-l"
    style="border-color: var(--ccg-hairline-soft); background: #F7F5F0;"
    aria-label="Relationships"
  >
    <div class="border-b px-4 py-3.5 text-[13px] font-semibold text-ink" style="border-color: var(--ccg-hairline-soft);">
      Relationships
    </div>
    <RelationshipGraph :agent-slug="agentSlug" :command-slug="commandSlug" :skill-slug="skillSlug" class="m-3" />
    <div class="flex gap-3.5 px-4 text-[11.5px]" style="color: var(--ccg-muted);">
      <span class="flex items-center gap-[5px]"><span class="h-2 w-2 rounded-[2px]" style="background: #1F1E1B;" />Agents</span>
      <span class="flex items-center gap-[5px]"><span class="h-2 w-2 rounded-[2px]" style="background: #3F7A4E;" />Skills</span>
      <span class="flex items-center gap-[5px]"><span class="h-2 w-2 rounded-[2px]" style="background: #7A4FA0;" />Commands</span>
    </div>
    <div
      class="m-4 rounded-[7px] border bg-white px-3 py-2.5 text-[12px] leading-[1.5]"
      style="border-color: var(--ccg-hairline-soft); color: var(--ccg-muted);"
    >
      Click a node to open it. Renaming <span class="font-mono">name:</span> won’t rename the file — slug is fixed at creation.
    </div>
  </aside>
</template>
