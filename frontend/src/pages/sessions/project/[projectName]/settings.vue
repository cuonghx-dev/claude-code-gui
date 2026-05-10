<script setup lang="ts">
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'

const route = useRoute()
const projectName = route.params.projectName as string
</script>

<template>
  <PageHeader :title="`${projectName} — settings`" subtitle="Project settings + CLAUDE.md editor" />
  <section class="p-6">
    <p class="text-sm text-neutral-500 dark:text-neutral-400">TODO (Phase 1 read; Phase 2 write).</p>
  </section>
</template>
