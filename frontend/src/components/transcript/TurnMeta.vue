<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types/ipc'

const props = defineProps<{ message: Message }>()

const compact = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k` : String(n))

const tokens = computed(() => {
  const u = props.message.usage
  if (!u) return ''
  const total = Number(u.input) + Number(u.output)
  const cached = Number(u.cacheRead)
  return cached ? `${compact(total)} tok · ${compact(cached)} cached` : `${compact(total)} tok`
})

const cost = computed(() => {
  const c = props.message.costUsd
  if (c == null) return ''
  return `$${c < 1 ? c.toFixed(3) : c.toFixed(2)}`
})
</script>

<template>
  <span v-if="tokens" class="font-mono" :title="message.costUsd != null ? `$${message.costUsd.toFixed(6)}` : undefined">
    {{ message.model }}<template v-if="message.model"> · </template>{{ tokens }}<template v-if="cost"> · {{ cost }}</template>
  </span>
</template>
