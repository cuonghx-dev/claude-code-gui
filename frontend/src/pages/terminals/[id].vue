<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import TerminalReplay from '@/components/TerminalReplay.vue'
import { useTerminalReplay } from '@/composables/useCliHistory'

const route = useRoute()
const id = computed(() => (route.params as { id: string }).id)

const { isPending, isError, error, data } = useTerminalReplay(id)

// xterm renders colour and box drawing correctly but is awkward to select
// from, so plain text stays one click away for copying and searching.
const plainText = ref(false)

const ANSI = /\u001b\[[0-9;?]*[ -/]*[@-~]|\u001b[@-Z\\-_]/g
const stripped = computed(() => (data.value?.lines ?? []).map((l) => l.replace(ANSI, '')).join('\n'))

const fmt = (iso: string | null) =>
  iso ? new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(iso)) : '—'
</script>

<template>
  <PageHeader
    :title="data?.entry.workingDir ?? `Session ${id.slice(0, 8)}…`"
    :subtitle="`${data?.entry.lineCount.toLocaleString() ?? 0} lines · ended ${fmt(data?.entry.endedAt ?? null)}`"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="plainText = !plainText">
        {{ plainText ? 'Show colors' : 'Plain text' }}
      </button>
      <RouterLink to="/terminals" class="ccg-btn-ghost">Back</RouterLink>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: d }">
      <section v-if="d" class="flex h-[calc(100vh-65px)] min-h-0 flex-col p-6">
        <pre
          v-if="plainText"
          class="min-h-0 flex-1 overflow-auto rounded-lg bg-neutral-100 p-3 font-mono text-xs dark:bg-neutral-900"
        >{{ stripped }}</pre>
        <TerminalReplay
          v-else
          :lines="d.lines"
          :cols="d.entry.cols"
          :rows="d.entry.rows"
          class="min-h-0 flex-1"
        />
      </section>
    </template>
  </QueryStateBoundary>
</template>
