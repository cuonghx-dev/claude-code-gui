<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { usePlan } from '@/composables/usePlans'

const route = useRoute()
const slug = computed(() => (route.params as { slug: string }).slug)
const { isPending, isError, error, data } = usePlan(slug)
</script>

<template>
  <PageHeader :title="data?.title ?? slug" :subtitle="data?.filename" />
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: plan }">
      <section v-if="plan" class="p-6">
        <pre class="max-h-[80vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm leading-relaxed dark:border-neutral-800 dark:bg-neutral-900">{{ plan.body }}</pre>
      </section>
    </template>
  </QueryStateBoundary>
</template>
