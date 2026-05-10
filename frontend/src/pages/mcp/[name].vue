<script setup lang="ts">
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'

const route = useRoute()
const name = route.params.name as string
</script>

<template>
  <PageHeader :title="name" subtitle="MCP server detail + capability probe" />
  <section class="p-6">
    <p class="text-sm text-neutral-500 dark:text-neutral-400">TODO (Phase 1 detail; Phase 5 probe).</p>
  </section>
</template>
