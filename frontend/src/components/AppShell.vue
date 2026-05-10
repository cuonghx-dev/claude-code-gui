<script setup lang="ts">
import { computed } from 'vue'
import Sidebar from './Sidebar.vue'
import OnboardingFlow from './OnboardingFlow.vue'
import { useSettings } from '@/composables/useSettings'

const { data, isPending } = useSettings()
const showOnboarding = computed(
  () => !isPending.value && data.value && data.value.onboardingCompleted !== true,
)
</script>

<template>
  <div class="flex h-full w-full overflow-hidden bg-neutral-50 dark:bg-neutral-950">
    <Sidebar />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
    <OnboardingFlow v-if="showOnboarding" />
  </div>
</template>
