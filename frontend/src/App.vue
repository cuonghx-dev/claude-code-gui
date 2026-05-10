<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import AppShell from '@/components/AppShell.vue'
import { attachFsListener, detachFsListener } from '@/lib/fsListener'

let detach: (() => void) | undefined

onMounted(async () => {
  detach = await attachFsListener()
})

onBeforeUnmount(() => {
  detach?.()
  detachFsListener()
})
</script>

<template>
  <AppShell>
    <RouterView />
  </AppShell>
</template>
