<script setup lang="ts">
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'

const route = useRoute()
const id = route.params.id as string
</script>

<template>
  <PageHeader :title="id" subtitle="Plugin detail" />
  <section class="p-6">
    <p class="text-sm text-neutral-500 dark:text-neutral-400">TODO (Phase 1 read; Phase 3 enable/disable).</p>
  </section>
</template>
