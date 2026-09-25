<script setup lang="ts">
import { ref } from 'vue'
import type { Message } from '@/types/ipc'

defineProps<{ message: Message }>()

const expanded = ref(false)
</script>

<template>
  <div>
    <button
      type="button"
      class="text-left text-[12.5px] italic transition-colors duration-[120ms] ease-out hover:text-body"
      style="color: var(--ccg-subtle);"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      {{ expanded ? '▾' : '▸' }} Thinking
    </button>
    <p
      v-if="expanded"
      class="mt-1.5 whitespace-pre-wrap border-l-2 pl-3 text-[12.5px] italic leading-[1.55]"
      style="color: var(--ccg-muted); border-color: var(--ccg-hairline);"
    >
      {{ message.thinking }}
    </p>
  </div>
</template>
