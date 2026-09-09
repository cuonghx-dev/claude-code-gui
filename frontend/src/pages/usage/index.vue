<script setup lang="ts">
import { computed, ref } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import BarChart from '@/components/charts/BarChart.vue'
import { useUsageActivity, useUsageRefresh, useUsageRollup } from '@/composables/useUsage'
import type { GroupBy, UsageQuery } from '@/types/ipc'

const groupBy = ref<GroupBy>('day')
const windowDays = ref(30)

const query = computed<UsageQuery>(() => ({
  fromMs: windowDays.value ? Date.now() - windowDays.value * 86_400_000 : null,
  toMs: null,
  project: null,
  groupBy: groupBy.value,
}))

const report = useUsageRollup(query)
const activity = useUsageActivity(windowDays)
const refresh = useUsageRefresh()

const money = (v: number) => `$${v.toFixed(2)}`
const tokens = (v: number) =>
  v >= 1_000_000 ? `${(v / 1_000_000).toFixed(1)}M` : v >= 1000 ? `${Math.round(v / 1000)}k` : `${v}`

const chart = computed(() =>
  (report.data.value?.buckets ?? []).map((b) => ({
    key: b.key,
    label: b.label,
    value: b.totals.costUsd,
  })),
)

const activityChart = computed(() =>
  (activity.data.value ?? []).map((d) => ({ key: d.date, label: d.date, value: d.prompts })),
)
</script>

<template>
  <PageHeader title="Usage" subtitle="Tokens and cost from session transcripts">
    <template #actions>
      <select v-model.number="windowDays" class="ccg-input w-auto">
        <option :value="7">7 days</option>
        <option :value="30">30 days</option>
        <option :value="90">90 days</option>
        <option :value="0">All time</option>
      </select>
      <select v-model="groupBy" class="ccg-input w-auto">
        <option value="day">By day</option>
        <option value="project">By project</option>
        <option value="model">By model</option>
      </select>
      <button
        type="button"
        class="ccg-btn-ghost"
        :disabled="refresh.isPending.value"
        @click="refresh.mutate()"
      >
        {{ refresh.isPending.value ? 'Rescanning…' : 'Rescan' }}
      </button>
    </template>
  </PageHeader>

  <QueryStateBoundary
    :is-pending="report.isPending.value"
    :is-error="report.isError.value"
    :error="report.error.value"
    :data="report.data.value"
  >
    <template #default="{ data: r }">
      <section v-if="r" class="space-y-6 p-6">
        <dl class="grid grid-cols-2 gap-4 md:grid-cols-4">
          <div class="rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Cost</dt>
            <dd class="mt-1 text-2xl font-semibold tabular-nums">{{ money(r.total.costUsd) }}</dd>
          </div>
          <div class="rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Turns</dt>
            <dd class="mt-1 text-2xl font-semibold tabular-nums">
              {{ Number(r.total.turns).toLocaleString() }}
            </dd>
          </div>
          <div class="rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Input / output</dt>
            <dd class="mt-1 text-2xl font-semibold tabular-nums">
              {{ tokens(Number(r.total.input)) }} / {{ tokens(Number(r.total.output)) }}
            </dd>
          </div>
          <div class="rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Cache read</dt>
            <dd class="mt-1 text-2xl font-semibold tabular-nums">
              {{ tokens(Number(r.total.cacheRead)) }}
            </dd>
          </div>
        </dl>

        <p
          v-if="r.unpricedTurns"
          class="rounded-lg bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300"
        >
          {{ Number(r.unpricedTurns).toLocaleString() }} turns are not priced — no rate is known for
          {{ r.unpricedModels.join(', ') }}. The cost above excludes them.
        </p>

        <div>
          <h3 class="mb-2 text-sm font-semibold">Cost by {{ groupBy }}</h3>
          <BarChart :data="chart" :format="money" />
        </div>

        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="text-left text-xs text-neutral-500 dark:text-neutral-400">
              <tr class="border-b border-neutral-200 dark:border-neutral-800">
                <th class="py-2 pr-4 font-medium">{{ groupBy }}</th>
                <th class="py-2 pr-4 text-right font-medium">Turns</th>
                <th class="py-2 pr-4 text-right font-medium">Input</th>
                <th class="py-2 pr-4 text-right font-medium">Output</th>
                <th class="py-2 pr-4 text-right font-medium">Cache read</th>
                <th class="py-2 text-right font-medium">Cost</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="b in r.buckets"
                :key="b.key"
                class="border-b border-neutral-100 dark:border-neutral-900"
              >
                <td class="py-2 pr-4">
                  {{ b.label }}
                  <span
                    v-if="b.byModel.length > 1"
                    class="ml-2 text-[11px] text-neutral-400"
                  >
                    {{ b.byModel.length }} models
                  </span>
                </td>
                <td class="py-2 pr-4 text-right tabular-nums">
                  {{ Number(b.totals.turns).toLocaleString() }}
                </td>
                <td class="py-2 pr-4 text-right tabular-nums">{{ tokens(Number(b.totals.input)) }}</td>
                <td class="py-2 pr-4 text-right tabular-nums">{{ tokens(Number(b.totals.output)) }}</td>
                <td class="py-2 pr-4 text-right tabular-nums">
                  {{ tokens(Number(b.totals.cacheRead)) }}
                </td>
                <td class="py-2 text-right tabular-nums">{{ money(b.totals.costUsd) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-if="activityChart.length">
          <h3 class="mb-2 text-sm font-semibold">Prompts per day</h3>
          <BarChart :data="activityChart" :height="80" />
        </div>

        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          {{ r.scannedFiles.toLocaleString() }} transcripts indexed. Cost is computed from each
          turn's recorded token counts at current published rates.
        </p>
      </section>
    </template>
  </QueryStateBoundary>
</template>
