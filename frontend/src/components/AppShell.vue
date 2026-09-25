<script setup lang="ts">
import { computed } from 'vue'
import Sidebar from './Sidebar.vue'
import TitleBar from './TitleBar.vue'
import OnboardingFlow from './OnboardingFlow.vue'
import { useSettings } from '@/composables/useSettings'

const { data, isPending } = useSettings()
const showOnboarding = computed(
  () => !isPending.value && data.value && data.value.onboardingCompleted !== true,
)
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden" style="background: var(--ccg-canvas);">
    <TitleBar />
    <div class="flex min-h-0 flex-1">
      <Sidebar />
      <main class="min-w-0 flex-1 overflow-auto" style="background: var(--ccg-canvas);">
        <slot />
      </main>
    </div>
    <OnboardingFlow v-if="showOnboarding" />
  </div>
</template>
