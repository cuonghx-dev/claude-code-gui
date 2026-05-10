<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { Toaster } from 'vue-sonner'
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
  <Toaster position="bottom-right" rich-colors close-button />
</template>
