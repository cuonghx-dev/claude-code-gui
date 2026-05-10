<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Toaster } from 'vue-sonner'
import AppShell from '@/components/AppShell.vue'
import { attachFsListener, detachFsListener } from '@/lib/fsListener'
import { attachDeepLinkListener } from '@/lib/deepLink'

let detachFs: (() => void) | undefined
let detachDeep: (() => void) | undefined
const router = useRouter()

onMounted(async () => {
  detachFs = await attachFsListener()
  detachDeep = await attachDeepLinkListener(router)
})

onBeforeUnmount(() => {
  detachFs?.()
  detachDeep?.()
  detachFsListener()
})
</script>

<template>
  <AppShell>
    <RouterView />
  </AppShell>
  <Toaster position="bottom-right" rich-colors close-button />
</template>
