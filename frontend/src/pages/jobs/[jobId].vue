<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import JobStateBadge from '@/components/JobStateBadge.vue'
import { useJob } from '@/composables/useJobs'

const route = useRoute()
const jobId = computed(() => (route.params as { jobId: string }).jobId)

const { isPending, isError, error, data } = useJob(jobId)

const fmt = (iso: string | null) =>
  iso ? new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(iso)) : '—'
</script>

<template>
  <div class="flex h-full flex-col">
    <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
      <template #default="{ data: d }">
        <div v-if="d" class="flex max-w-[960px] flex-col gap-[18px] px-7 py-[22px]">
          <div class="flex flex-col gap-1">
            <nav class="text-[13px]" style="color: var(--ccg-subtle);">
              <RouterLink to="/jobs" class="hover:text-ink">Jobs</RouterLink> /
            </nav>
            <div class="flex items-center gap-2.5">
              <h2 class="min-w-0 truncate text-[17px] font-semibold text-ink">{{ d.job.name }}</h2>
              <JobStateBadge :state="d.job.state" />
              <span class="flex-1" />
              <RouterLink
                v-if="d.job.resumeSessionId && d.job.cwd"
                to="/sessions"
                class="ccg-btn-ghost ccg-btn-sm"
              >
                Session <span class="font-mono">{{ d.job.resumeSessionId.slice(0, 8) }}</span>
              </RouterLink>
            </div>
            <p class="ccg-path">~/.claude/jobs/{{ jobId }}</p>
          </div>

          <div class="ccg-card grid grid-cols-4" style="border-radius: 10px;">
            <div
              v-for="(cell, ci) in [
                { label: 'Created', value: fmt(d.job.createdAt) },
                { label: 'Updated', value: fmt(d.job.updatedAt) },
                { label: 'Tokens', value: d.job.tokens?.toLocaleString() ?? '—' },
                { label: 'Working directory', value: d.job.cwd ?? '—', mono: true },
              ]"
              :key="cell.label"
              class="flex min-w-0 flex-col gap-1 px-4 py-3"
              :class="ci > 0 ? 'border-l' : ''"
              style="border-color: #F0EDE7;"
            >
              <span class="text-[12px]" style="color: var(--ccg-subtle);">{{ cell.label }}</span>
              <span
                class="truncate text-[13px] text-ink tabular-nums"
                :class="cell.mono ? 'font-mono text-[12px]' : ''"
                :title="cell.value"
              >{{ cell.value }}</span>
            </div>
          </div>

          <div v-if="d.job.needs" class="ccg-alert-warn px-3.5 py-2.5 text-[13px]">
            <span class="font-semibold">Waiting on:</span> {{ d.job.needs }}
          </div>

          <div v-if="d.job.intent" class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Prompt</span>
            <p class="whitespace-pre-wrap text-[13px] leading-[1.5]" style="color: var(--ccg-body);">
              {{ d.job.intent }}
            </p>
          </div>

          <div v-if="d.job.children.length" class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Artifacts</span>
            <div class="ccg-card overflow-hidden" style="border-radius: 8px;">
              <div
                v-for="(c, i) in d.job.children"
                :key="c.id"
                class="flex items-center gap-3 px-3 py-2 font-mono text-[12.5px]"
                :class="i > 0 ? 'border-t' : ''"
                style="border-color: var(--ccg-hairline-faint);"
              >
                <span class="ccg-badge">{{ c.kind ?? 'link' }}</span>
                <a
                  v-if="c.href"
                  :href="c.href"
                  target="_blank"
                  rel="noreferrer"
                  class="min-w-0 truncate underline"
                  style="color: var(--ccg-blue);"
                >{{ c.href }}</a>
                <span v-else class="min-w-0 truncate text-ink">{{ c.id }}</span>
              </div>
            </div>
          </div>

          <div v-if="d.job.output" class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Output</span>
            <pre class="ccg-code-block max-h-[50vh]">{{ d.job.output }}</pre>
          </div>

          <div class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Timeline</span>
            <ol
              v-if="d.timeline.length"
              class="flex flex-col gap-3 border-l pl-4"
              style="border-color: var(--ccg-hairline);"
            >
              <li v-for="(e, i) in d.timeline" :key="i" class="flex flex-col gap-1">
                <div class="flex items-center gap-2">
                  <JobStateBadge v-if="e.state" :state="e.state" />
                  <span class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">{{ fmt(e.at) }}</span>
                </div>
                <p v-if="e.detail" class="text-[13px] text-ink">{{ e.detail }}</p>
                <p v-if="e.text" class="whitespace-pre-wrap text-[12.5px]" style="color: var(--ccg-muted);">
                  {{ e.text }}
                </p>
              </li>
            </ol>
            <p v-else class="text-[13px]" style="color: var(--ccg-subtle);">No timeline recorded.</p>
            <p v-if="d.malformedLines" class="text-[12px]" style="color: var(--ccg-muted-soft);">
              {{ d.malformedLines }} unreadable event{{ d.malformedLines === 1 ? '' : 's' }} skipped.
            </p>
          </div>

          <p v-if="d.job.providerEnvKeys.length" class="text-[12px]" style="color: var(--ccg-subtle);">
            Provider environment:
            <span class="font-mono">{{ d.job.providerEnvKeys.join(', ') }}</span>
            (values not read)
          </p>
        </div>
      </template>
    </QueryStateBoundary>
  </div>
</template>
