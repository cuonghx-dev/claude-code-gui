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
  <QueryStateBoundary
    :is-pending="isPending"
    :is-error="isError"
    :error="error"
    :data="data"
    skeleton="rows"
  >
    <template #default="{ data: rows }">
      <section class="flex flex-col gap-2 px-7 py-[22px]">
        <div class="flex items-baseline gap-2">
          <span class="text-[13px] font-semibold text-ink">Effective settings</span>
          <span class="text-[12px] text-[#A29E94]">
            Highest-precedence file first; permission lists merge across files instead of
            overriding
          </span>
        </div>
        <div class="overflow-hidden rounded-lg border border-[#E6E2DA] bg-white">
          <div v-if="!rows?.length" class="px-3 py-6 text-center text-[13px] text-[#8A867C]">
            No settings are set anywhere.
          </div>
          <table v-else class="w-full text-[12.5px]">
            <thead>
              <tr class="border-b border-[#ECE9E2] bg-[#FAF9F6] text-left text-[11px] uppercase tracking-[.08em] text-[#A29E94]">
                <th class="px-3 py-1.5 font-semibold">Key</th>
                <th class="px-3 py-1.5 font-semibold">Value</th>
                <th class="px-3 py-1.5 font-semibold">From</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="r in rows"
                :key="r.key"
                class="border-b border-[#F3F0EA] align-top last:border-b-0"
              >
                <td class="whitespace-nowrap px-3 py-2 font-mono text-ink">{{ r.key }}</td>
                <td class="w-full max-w-0 px-3 py-2">
                  <pre class="overflow-x-auto whitespace-pre-wrap break-all font-mono text-[12px] text-[#5E5B54]">{{ render(r.value) }}</pre>
                </td>
                <td class="whitespace-nowrap px-3 py-2">
                  <span class="inline-flex items-center gap-1">
                    <SourceBadge :scope="r.source" active />
                    <SourceBadge
                      v-for="o in r.overridden"
                      :key="o"
                      :scope="o"
                      :overridden="!r.merged"
                    />
                    <span v-if="r.merged" class="text-[11px] text-[#A29E94]">merged</span>
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
</template>
