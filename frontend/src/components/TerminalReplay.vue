<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { useDebounceFn } from '@vueuse/core'

const props = defineProps<{
  /** Raw scrollback with ANSI escapes intact. */
  lines: string[]
  cols?: number | null
  rows?: number | null
}>()

const host = ref<HTMLDivElement>()

let term: Terminal | undefined
let fit: FitAddon | undefined

const render = () => {
  if (!term) return
  term.clear()
  // Replaying a recorded buffer, so no PTY and no input: write once and stop.
  term.write(props.lines.join('\r\n'))
}

const refit = useDebounceFn(() => fit?.fit(), 100)

onMounted(() => {
  term = new Terminal({
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
    fontSize: 13,
    cursorBlink: false,
    disableStdin: true,
    scrollback: 10_000,
    theme: { background: '#0a0a0a' },
  })
  fit = new FitAddon()
  term.loadAddon(fit)
  if (host.value) {
    term.open(host.value)
    fit.fit()
  }
  render()
  window.addEventListener('resize', refit)
})

watch(() => props.lines, render)

onBeforeUnmount(() => {
  window.removeEventListener('resize', refit)
  term?.dispose()
  term = undefined
})
</script>

<template>
  <div ref="host" class="h-full w-full overflow-hidden rounded-lg bg-[#0a0a0a] p-2" />
</template>
