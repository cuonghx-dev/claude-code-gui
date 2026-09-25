<script setup lang="ts">
import type { DiffResult } from '@/types/ipc'

defineProps<{ diff: DiffResult }>()
</script>

<template>
  <div class="overflow-hidden rounded-lg border bg-white" style="border-color: var(--ccg-hairline);">
    <header
      class="flex items-baseline gap-3 border-b px-3 py-2 font-mono text-[12px]"
      style="border-color: var(--ccg-hairline-soft);"
    >
      <span class="truncate" style="color: var(--ccg-error);">− {{ diff.leftLabel }}</span>
      <span class="truncate" style="color: var(--ccg-success);">+ {{ diff.rightLabel }}</span>
    </header>

    <p v-if="diff.binary" class="px-3 py-2 text-[12px]" style="color: var(--ccg-subtle);">
      Binary or oversized file — no line diff.
    </p>
    <p v-else-if="!diff.hunks.length" class="px-3 py-2 text-[12px]" style="color: var(--ccg-subtle);">
      No differences.
    </p>
    <div v-else class="max-h-[32rem] overflow-auto">
      <table class="w-full border-collapse font-mono text-[12px] leading-[1.6]" style="color: var(--ccg-body);">
        <tbody>
          <template v-for="(h, hi) in diff.hunks" :key="hi">
            <tr style="background: var(--ccg-surface-strong); color: var(--ccg-subtle);">
              <td colspan="3" class="px-2 py-1">@@ −{{ h.leftStart }} +{{ h.rightStart }} @@</td>
            </tr>
            <tr
              v-for="(l, li) in h.lines"
              :key="`${hi}-${li}`"
              :style="{
                background: l.tag === 'insert' ? 'var(--ccg-success-bg)' : l.tag === 'delete' ? 'var(--ccg-error-bg)' : undefined,
              }"
            >
              <td class="w-12 select-none px-2 text-right tabular-nums" style="color: var(--ccg-muted-soft);">
                {{ l.leftNo ?? '' }}
              </td>
              <td class="w-12 select-none px-2 text-right tabular-nums" style="color: var(--ccg-muted-soft);">
                {{ l.rightNo ?? '' }}
              </td>
              <td class="whitespace-pre-wrap px-2">
                <span
                  class="select-none"
                  :style="{
                    color: l.tag === 'insert' ? 'var(--ccg-success)' : l.tag === 'delete' ? 'var(--ccg-error)' : 'var(--ccg-muted-soft)',
                  }"
                >{{ l.tag === 'insert' ? '+' : l.tag === 'delete' ? '−' : ' ' }}</span>
                {{ l.text }}
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
    <p
      v-if="diff.truncated"
      class="border-t px-3 py-2 text-[12px]"
      style="border-color: var(--ccg-hairline-soft); color: var(--ccg-subtle);"
    >
      Diff truncated at 2000 lines.
    </p>
  </div>
</template>
