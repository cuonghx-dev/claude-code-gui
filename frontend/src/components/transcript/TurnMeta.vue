<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types/ipc'

const props = defineProps<{ message: Message }>()

const tokens = computed(() => {
  const u = props.message.usage
  if (!u) return ''
  const total = Number(u.input) + Number(u.output)
  const cached = Number(u.cacheRead)
  return cached ? `${total.toLocaleString()} tok · ${cached.toLocaleString()} cached` : `${total.toLocaleString()} tok`
})

const cost = computed(() =>
  props.message.costUsd != null ? `$${props.message.costUsd.toFixed(4)}` : '',
)
</script>

<template>
  <span v-if="tokens" class="text-[11px] text-neutral-400">
    {{ message.model }}<template v-if="message.model"> · </template>{{ tokens }}
    <template v-if="cost"> · {{ cost }}</template>
  </span>
</template>
