<script setup lang="ts">
import { computed, ref } from 'vue'

export interface BarPart {
  name: string
  value: number
  color: string
}
export interface Bar {
  key: string
  label: string
  /** Stacked bottom-up in this order. */
  parts: BarPart[]
}

const props = withDefaults(
  defineProps<{
    data: Bar[]
    height?: number
    /** Formats tooltip values. */
    format?: (v: number) => string
  }>(),
  { height: 170, format: undefined },
)

// Hand-rolled rather than pulling in a chart library: stacked bars over a few
// hundred points does not justify 60-500 kB plus a second theming system
// fighting the app's CSS tokens.
const totals = computed(() => props.data.map((d) => d.parts.reduce((a, p) => a + p.value, 0)))
const max = computed(() => Math.max(1e-9, ...totals.value))
// The design's 5px gap only fits month-scale ranges; thin it for long ones.
const gap = computed(() => (props.data.length > 120 ? 0 : props.data.length > 60 ? 2 : 5))

const hovered = ref<number | null>(null)
const fmt = (v: number) => (props.format ? props.format(v) : v.toLocaleString())
const pct = (v: number) => `${(v / max.value) * 100}%`

const tip = computed(() => {
  const i = hovered.value
  if (i === null || !props.data[i]) return null
  const d = props.data[i]
  const center = ((i + 0.5) / props.data.length) * 100
  return {
    d,
    total: totals.value[i],
    parts: d.parts.filter((p) => p.value > 0).slice().reverse(),
    left: `${Math.min(88, Math.max(12, center))}%`,
  }
})
</script>

<template>
  <div class="relative" @mouseleave="hovered = null">
    <div
      class="flex items-end border-b border-hairline-soft"
      :style="{ height: `${height}px`, gap: `${gap}px` }"
      role="img"
    >
      <div
        v-for="(d, i) in data"
        :key="d.key"
        class="bar flex h-full min-w-0 flex-1 flex-col-reverse"
        :class="hovered !== null && hovered !== i ? 'opacity-60' : ''"
        @mouseenter="hovered = i"
      >
        <div
          v-for="(p, j) in d.parts"
          :key="p.name"
          :style="{
            height: pct(p.value),
            background: p.color,
            borderRadius: j === d.parts.length - 1 ? '2px 2px 0 0' : undefined,
          }"
        />
        <span class="sr-only">{{ d.label }}: {{ fmt(totals[i]) }}</span>
      </div>
    </div>
    <div
      v-if="tip"
      class="chart-tip pointer-events-none absolute bottom-full z-10 mb-1.5 -translate-x-1/2 whitespace-nowrap"
      :style="{ left: tip.left }"
    >
      <div class="font-medium text-ink">{{ tip.d.label }} · {{ fmt(tip.total) }}</div>
      <div v-for="p in tip.parts" :key="p.name" class="flex items-center gap-1.5">
        <span class="inline-block size-2 rounded-[2px]" :style="{ background: p.color }" />
        <span class="flex-1">{{ p.name }}</span>
        <span class="ml-3 font-mono tabular-nums">{{ fmt(p.value) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bar {
  transition: opacity 120ms ease-out;
}
.chart-tip {
  padding: 6px 9px;
  border-radius: 7px;
  background: var(--ccg-surface-card);
  border: 1px solid var(--ccg-hairline);
  box-shadow: 0 4px 14px rgba(40, 30, 20, .1);
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--ccg-muted);
}
</style>
