<script setup lang="ts">
defineProps<{ label: string; hint?: string; error?: string; required?: boolean }>()
</script>

<template>
  <label class="flex flex-col gap-1.5">
    <span class="text-[12.5px] font-medium text-ink">
      {{ label }}<span v-if="required" class="ml-0.5 text-[#B03A2E]">*</span>
    </span>
    <slot />
    <span v-if="error" class="text-[12px] text-[#B03A2E]">
      {{ error }}
    </span>
    <span v-else-if="hint" class="text-[12px] text-[#A29E94]">
      {{ hint }}
    </span>
  </label>
</template>
