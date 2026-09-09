<script setup lang="ts">
import { ref } from 'vue'
import type { Message } from '@/types/ipc'

defineProps<{ message: Message }>()

const expanded = ref(false)
</script>

<template>
  <div class="rounded-lg bg-neutral-100/60 dark:bg-neutral-900/60">
    <button
      type="button"
      class="flex w-full items-baseline gap-2 px-3 py-2 text-left"
      @click="expanded = !expanded"
    >
      <span class="text-xs font-medium italic text-neutral-500 dark:text-neutral-400">thinking</span>
      <span class="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400">
        {{ expanded ? '' : (message.thinking ?? '') }}
      </span>
      <span class="text-xs text-neutral-400">{{ expanded ? '−' : '+' }}</span>
    </button>
    <p
      v-if="expanded"
      class="whitespace-pre-wrap px-3 pb-3 text-sm italic text-neutral-600 dark:text-neutral-300"
    >
      {{ message.thinking }}
    </p>
  </div>
</template>
