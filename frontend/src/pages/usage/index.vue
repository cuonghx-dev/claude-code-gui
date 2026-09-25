<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import BarChart, { type Bar } from '@/components/charts/BarChart.vue'
import { useUsageActivity, useUsageRefresh, useUsageRollup } from '@/composables/useUsage'
import type { GroupBy, UsageBucket, UsageQuery } from '@/types/ipc'

// Range and grouping live in the URL (?range=7d&group=project) so views are linkable.
const RANGES = [
  { v: '7d', label: '7d', days: 7 },
  { v: '30d', label: '30d', days: 30 },
  { v: '90d', label: '90d', days: 90 },
  { v: 'all', label: 'All', days: 0 },
] as const
const GROUPS = [
  { v: 'day', label: 'Day' },
  { v: 'project', label: 'Project' },
  { v: 'model', label: 'Model' },
] as const

const route = useRoute()
const router = useRouter()
const range = computed(() => RANGES.find((r) => r.v === route.query.range) ?? RANGES[1])
const groupBy = computed<GroupBy>(
  () => GROUPS.find((g) => g.v === route.query.group)?.v ?? 'day',
)
const windowDays = computed(() => range.value.days)

function setQuery(patch: { range?: string; group?: string }) {
  const next = { range: range.value.v as string, group: groupBy.value as string, ...patch }
  router.replace({
    query: {
      ...route.query,
      range: next.range === '30d' ? undefined : next.range,
      group: next.group === 'day' ? undefined : next.group,
    },
  })
}

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
function tokens(v: number) {
  if (v >= 100_000_000) return `${Math.round(v / 1_000_000)}M`
  if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`
  if (v >= 1000) return `${Math.round(v / 1000)}k`
  return `${v}`
}
const count = (v: number) => Number(v).toLocaleString()

// Stack by model family so point releases (opus-4-1, opus-4-5) share a colour.
const FAMILY_COLORS: Record<string, string> = {
  opus: '#C2552D',
  sonnet: '#E7B9A3',
  haiku: '#D9A15E',
}
const OTHER_COLORS = ['#A89A86', '#C9BBA6', '#8A7A66', '#DCCFBE']
function family(model: string) {
  const m = model.toLowerCase()
  return Object.keys(FAMILY_COLORS).find((f) => m.includes(f)) ?? model
}

const families = computed(() => {
  const seen = new Map<string, number>()
  for (const b of report.data.value?.buckets ?? []) {
    for (const m of b.byModel) {
      const f = family(m.model)
      seen.set(f, (seen.get(f) ?? 0) + m.totals.costUsd)
    }
  }
  const known = Object.keys(FAMILY_COLORS).filter((f) => seen.has(f))
  const others = [...seen.keys()]
    .filter((f) => !(f in FAMILY_COLORS))
    .sort((a, b) => (seen.get(b) ?? 0) - (seen.get(a) ?? 0))
  const colors = new Map<string, string>()
  known.forEach((f) => colors.set(f, FAMILY_COLORS[f]))
  others.forEach((f, i) => colors.set(f, OTHER_COLORS[i % OTHER_COLORS.length]))
  return colors
})

function toBar(key: string, label: string, b?: UsageBucket): Bar {
  const byFamily = new Map<string, number>()
  for (const m of b?.byModel ?? []) {
    const f = family(m.model)
    byFamily.set(f, (byFamily.get(f) ?? 0) + m.totals.costUsd)
  }
  return {
    key,
    label,
    parts: [...families.value].map(([name, color]) => ({
      name,
      color,
      value: byFamily.get(name) ?? 0,
    })),
  }
}

const isoDay = (d: Date) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
const shortDate = (iso: string) => {
  const [y, m, d] = iso.split('-').map(Number)
  if (!y || !m || !d) return iso
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

// Day buckets are sparse; fill empty days so the x-axis reads as time.
const chart = computed<Bar[]>(() => {
  const buckets = report.data.value?.buckets ?? []
  if (groupBy.value !== 'day') return buckets.map((b) => toBar(b.key, b.label, b))
  const byKey = new Map(buckets.map((b) => [b.key, b]))
  const keys = new Set(buckets.map((b) => b.key))
  if (windowDays.value) {
    const today = new Date()
    for (let i = windowDays.value - 1; i >= 0; i--) {
      keys.add(isoDay(new Date(today.getFullYear(), today.getMonth(), today.getDate() - i)))
    }
  }
  return [...keys].sort().map((k) => toBar(k, shortDate(k), byKey.get(k)))
})

const rangeLabel = computed(() => {
  if (groupBy.value === 'day' && chart.value.length) {
    return `${chart.value[0].label} → ${chart.value[chart.value.length - 1].label}`
  }
  return range.value.days ? `Last ${range.value.days} days` : 'All time'
})

const groupLabel = computed(() => GROUPS.find((g) => g.v === groupBy.value)?.label ?? '')

const activityChart = computed<Bar[]>(() =>
  (activity.data.value ?? []).map((d) => ({
    key: d.date,
    label: shortDate(d.date),
    parts: [{ name: 'prompts', value: d.prompts, color: '#C9BBA6' }],
  })),
)
</script>

<template>
  <div class="flex h-full flex-col overflow-auto">
    <PageHeader title="Usage" class="border-b-0!">
      <template #actions>
        <div class="ccg-seg" role="group" aria-label="Range">
          <button
            v-for="r in RANGES"
            :key="r.v"
            type="button"
            :aria-pressed="range.v === r.v"
            @click="setQuery({ range: r.v })"
          >
            {{ r.label }}
          </button>
        </div>
        <div class="ccg-seg" role="group" aria-label="Group by">
          <button
            v-for="g in GROUPS"
            :key="g.v"
            type="button"
            :aria-pressed="groupBy === g.v"
            @click="setQuery({ group: g.v })"
          >
            {{ g.label }}
          </button>
        </div>
        <button
          type="button"
          class="ccg-btn-ghost ccg-btn-sm"
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
      <template #loading>
        <div class="flex flex-col gap-4 px-7">
          <div class="ccg-skeleton h-[78px]" style="border-radius: 10px;" />
          <div class="ccg-skeleton h-[262px]" style="border-radius: 10px;" />
          <div class="ccg-skeleton h-[160px]" style="border-radius: 10px;" />
        </div>
      </template>
      <template #default="{ data: r }">
        <div v-if="r" class="flex flex-col gap-4 px-7 pb-6">
          <dl class="usage-card grid grid-cols-5">
            <div class="kpi">
              <dt>Est. cost</dt>
              <dd>{{ money(r.total.costUsd) }}</dd>
            </div>
            <div class="kpi">
              <dt>Turns</dt>
              <dd>{{ count(r.total.turns) }}</dd>
            </div>
            <div class="kpi">
              <dt>Input / Output</dt>
              <dd>
                {{ tokens(Number(r.total.input)) }}
                <span class="font-normal text-neutral-400">/</span>
                {{ tokens(Number(r.total.output)) }}
              </dd>
            </div>
            <div class="kpi">
              <dt>Cache read</dt>
              <dd>{{ tokens(Number(r.total.cacheRead)) }}</dd>
            </div>
            <div class="kpi">
              <dt>Cache write</dt>
              <dd>{{ tokens(Number(r.total.cacheWrite)) }}</dd>
            </div>
          </dl>

          <p v-if="r.unpricedTurns" class="ccg-alert-warn px-3 py-2 text-[12.5px]" role="status">
            {{ count(r.unpricedTurns) }} turns are not priced — no rate is known for
            <span class="font-mono">{{ r.unpricedModels.join(', ') }}</span>. The cost above excludes them.
          </p>

          <section class="usage-card flex flex-col gap-3 px-[18px] py-4">
            <div class="flex items-baseline">
              <h3 class="flex-1 text-[13px] font-semibold text-ink">Cost by {{ groupLabel.toLowerCase() }}</h3>
              <span class="text-[11.5px] text-neutral-400">Estimate at public API prices · not a bill</span>
            </div>
            <BarChart v-if="chart.length" :data="chart" :format="money" />
            <p v-else class="py-10 text-center text-[13px] text-neutral-500">No usage in this range.</p>
            <div class="flex flex-wrap items-center gap-4 text-[11.5px] text-muted">
              <span v-for="[name, color] in families" :key="name" class="flex items-center gap-[5px]">
                <span class="inline-block size-2 rounded-[2px]" :style="{ background: color }" />
                <span class="font-mono">{{ name }}</span>
              </span>
              <span class="flex-1" />
              <span>{{ rangeLabel }}</span>
            </div>
          </section>

          <section class="usage-card overflow-hidden">
            <div class="usage-row usage-head">
              <span>{{ groupLabel }}</span>
              <span>Turns</span>
              <span>Input</span>
              <span>Output</span>
              <span>Cost</span>
            </div>
            <div
              v-for="b in r.buckets"
              :key="b.key"
              class="usage-row"
              :title="`Cache read ${tokens(Number(b.totals.cacheRead))} · Cache write ${tokens(Number(b.totals.cacheWrite))}`"
            >
              <span class="truncate font-mono text-[12.5px] text-ink">
                {{ b.label }}
                <span v-if="b.byModel.length > 1" class="ml-2 font-sans text-[11px] text-neutral-400">
                  {{ b.byModel.length }} models
                </span>
              </span>
              <span>{{ count(b.totals.turns) }}</span>
              <span>{{ tokens(Number(b.totals.input)) }}</span>
              <span>{{ tokens(Number(b.totals.output)) }}</span>
              <span class="font-medium text-ink">{{ money(b.totals.costUsd) }}</span>
            </div>
            <p v-if="!r.buckets.length" class="px-4 py-3 text-[13px] text-neutral-500">No usage in this range.</p>
          </section>

          <section v-if="activityChart.length" class="usage-card flex flex-col gap-3 px-[18px] py-4">
            <h3 class="text-[13px] font-semibold text-ink">Prompts per day</h3>
            <BarChart :data="activityChart" :height="80" />
          </section>

          <p class="text-[12px] leading-relaxed text-neutral-500">
            {{ count(r.scannedFiles) }} transcripts indexed. Cost is what these tokens would
            cost at current published API rates — an estimate, not a bill: on a subscription plan you
            are not charged per token. A turn is one API response; days follow your local time zone.
          </p>
        </div>
      </template>
    </QueryStateBoundary>
  </div>
</template>

<style scoped>
.usage-card {
  border: 1px solid var(--ccg-hairline);
  border-radius: 10px;
  background: var(--ccg-surface-card);
}
.kpi {
  padding: 14px 16px;
  border-right: 1px solid #f0ede7;
}
.kpi:last-child {
  border-right: 0;
}
.kpi dt {
  font-size: 12px;
  color: var(--ccg-subtle);
}
.kpi dd {
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -.02em;
  color: var(--ccg-ink);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.usage-row {
  display: grid;
  grid-template-columns: minmax(0, 2fr) repeat(4, minmax(0, 1fr));
  align-items: baseline;
  padding: 9px 16px;
  font-size: 13px;
  color: var(--ccg-body);
  border-bottom: 1px solid var(--ccg-hairline-faint);
  font-variant-numeric: tabular-nums;
}
.usage-row:last-child {
  border-bottom: 0;
}
.usage-row > span:not(:first-child) {
  text-align: right;
}
.usage-head {
  font-size: 11.5px;
  color: var(--ccg-subtle);
  background: var(--ccg-canvas-soft);
  border-bottom-color: var(--ccg-hairline-soft);
}
</style>
