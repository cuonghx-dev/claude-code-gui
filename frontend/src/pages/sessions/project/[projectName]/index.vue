<script setup lang="ts">
import EmptyState from '@/components/EmptyState.vue'
</script>

<template>
  <section class="flex h-full items-center justify-center p-6">
    <EmptyState
      title="No session selected"
      hint="Pick a session from the list, or start a new chat."
    />
  </section>
</template>
