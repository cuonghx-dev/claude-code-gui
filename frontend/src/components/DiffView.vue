<script setup lang="ts">
import type { DiffResult } from '@/types/ipc'

defineProps<{ diff: DiffResult }>()
</script>

<template>
  <div class="rounded-lg border border-neutral-200 dark:border-neutral-800">
    <header
      class="flex items-baseline gap-2 border-b border-neutral-200 px-3 py-2 text-xs dark:border-neutral-800"
    >
      <span class="font-mono text-red-600 dark:text-red-400">− {{ diff.leftLabel }}</span>
      <span class="font-mono text-emerald-600 dark:text-emerald-400">+ {{ diff.rightLabel }}</span>
    </header>

    <p v-if="diff.binary" class="px-3 py-2 text-xs text-neutral-500 dark:text-neutral-400">
      Binary or oversized file — no line diff.
    </p>
    <p
      v-else-if="!diff.hunks.length"
      class="px-3 py-2 text-xs text-neutral-500 dark:text-neutral-400"
    >
      No differences.
    </p>
    <div v-else class="max-h-[32rem] overflow-auto">
      <table class="w-full border-collapse font-mono text-xs">
        <tbody>
          <template v-for="(h, hi) in diff.hunks" :key="hi">
            <tr class="bg-neutral-100 text-neutral-500 dark:bg-neutral-900 dark:text-neutral-400">
              <td colspan="3" class="px-2 py-1">@@ −{{ h.leftStart }} +{{ h.rightStart }} @@</td>
            </tr>
            <tr
              v-for="(l, li) in h.lines"
              :key="`${hi}-${li}`"
              :class="{
                'bg-emerald-500/10': l.tag === 'insert',
                'bg-red-500/10': l.tag === 'delete',
              }"
            >
              <td class="w-12 select-none px-2 text-right text-neutral-400 tabular-nums">
                {{ l.leftNo ?? '' }}
              </td>
              <td class="w-12 select-none px-2 text-right text-neutral-400 tabular-nums">
                {{ l.rightNo ?? '' }}
              </td>
              <td class="whitespace-pre-wrap px-2">
                <span class="select-none text-neutral-400">{{
                  l.tag === 'insert' ? '+' : l.tag === 'delete' ? '−' : ' '
                }}</span>
                {{ l.text }}
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
    <p
      v-if="diff.truncated"
      class="border-t border-neutral-200 px-3 py-2 text-xs text-neutral-500 dark:border-neutral-800 dark:text-neutral-400"
    >
      Diff truncated at 2000 lines.
    </p>
  </div>
</template>
