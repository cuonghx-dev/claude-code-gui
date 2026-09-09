<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    /** Bars, in display order. */
    data: { key: string; label: string; value: number }[]
    height?: number
    /** Formats the tooltip value. */
    format?: (v: number) => string
  }>(),
  { height: 140 },
)

// Hand-rolled rather than pulling in a chart library: three chart shapes over a
// few hundred points does not justify 60-500 kB plus a second theming system
// fighting the app's CSS tokens.
const max = computed(() => Math.max(1, ...props.data.map((d) => d.value)))
const barWidth = computed(() => 100 / Math.max(1, props.data.length))

const fmt = (v: number) => (props.format ? props.format(v) : v.toLocaleString())
</script>

<template>
  <svg
    :viewBox="`0 0 100 ${height}`"
    preserveAspectRatio="none"
    class="w-full"
    :style="{ height: `${height}px` }"
    role="img"
  >
    <g v-for="(d, i) in data" :key="d.key">
      <rect
        :x="i * barWidth + barWidth * 0.15"
        :width="barWidth * 0.7"
        :y="height - (d.value / max) * height"
        :height="Math.max(1, (d.value / max) * height)"
        class="fill-blue-500/70 hover:fill-blue-500"
      >
        <title>{{ d.label }}: {{ fmt(d.value) }}</title>
      </rect>
    </g>
  </svg>
</template>
