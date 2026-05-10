<script setup lang="ts">
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'

const route = useRoute()
const projectName = route.params.projectName as string
const sessionId = route.params.sessionId as string
</script>

<template>
  <PageHeader :title="`Session ${sessionId}`" :subtitle="projectName" />
  <section class="p-6">
    <p class="text-sm text-neutral-500 dark:text-neutral-400">
      TODO (Phase 1 viewer; Phase 4 Resume terminal).
    </p>
  </section>
</template>
