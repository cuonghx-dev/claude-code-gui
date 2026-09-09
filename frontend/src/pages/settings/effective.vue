<script setup lang="ts">
import { inject } from 'vue'
import SourceBadge from '@/components/settings/SourceBadge.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import { useSettingsEffective } from '@/composables/useSettings'

const ctx = inject(SETTINGS_CONTEXT)!
const { isPending, isError, error, data } = useSettingsEffective(ctx.workingDir)

const render = (v: unknown) => (typeof v === 'string' ? v : JSON.stringify(v))
</script>

<template>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: rows }">
      <section class="p-6">
        <p class="mb-3 text-xs text-neutral-500 dark:text-neutral-400">
          What Claude Code actually uses, highest-precedence file first. Permission lists merge
          across files rather than overriding, so a merged row shows every scope's rules.
        </p>
        <table class="w-full text-sm">
          <thead class="text-left text-xs text-neutral-500 dark:text-neutral-400">
            <tr class="border-b border-neutral-200 dark:border-neutral-800">
              <th class="py-2 pr-4 font-medium">Key</th>
              <th class="py-2 pr-4 font-medium">Value</th>
              <th class="py-2 font-medium">From</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="r in rows"
              :key="r.key"
              class="border-b border-neutral-100 align-top dark:border-neutral-900"
            >
              <td class="py-2 pr-4 font-mono text-xs">{{ r.key }}</td>
              <td class="max-w-xl py-2 pr-4">
                <pre class="overflow-x-auto whitespace-pre-wrap font-mono text-xs">{{ render(r.value) }}</pre>
              </td>
              <td class="whitespace-nowrap py-2">
                <SourceBadge :scope="r.source" />
                <SourceBadge
                  v-for="o in r.overridden"
                  :key="o"
                  :scope="o"
                  :overridden="!r.merged"
                  class="ml-1"
                />
                <span v-if="r.merged" class="ml-1 text-[11px] text-neutral-400">merged</span>
              </td>
            </tr>
          </tbody>
        </table>
        <p v-if="!rows?.length" class="text-sm text-neutral-500">No settings are set anywhere.</p>
      </section>
    </template>
  </QueryStateBoundary>
</template>
