<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
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
  <PageHeader :title="data?.job.name ?? jobId" :subtitle="`~/.claude/jobs/${jobId}`">
    <template #actions>
      <RouterLink to="/jobs" class="ccg-btn-ghost">Back</RouterLink>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: d }">
      <section v-if="d" class="space-y-6 p-6">
        <dl class="grid grid-cols-2 gap-x-6 gap-y-3 text-sm md:grid-cols-4">
          <div>
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">State</dt>
            <dd class="mt-1"><JobStateBadge :state="d.job.state" /></dd>
          </div>
          <div>
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Created</dt>
            <dd class="mt-1">{{ fmt(d.job.createdAt) }}</dd>
          </div>
          <div>
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Updated</dt>
            <dd class="mt-1">{{ fmt(d.job.updatedAt) }}</dd>
          </div>
          <div>
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Tokens</dt>
            <dd class="mt-1">{{ d.job.tokens?.toLocaleString() ?? '—' }}</dd>
          </div>
          <div class="col-span-2 md:col-span-4">
            <dt class="text-xs text-neutral-500 dark:text-neutral-400">Working directory</dt>
            <dd class="mt-1 font-mono text-xs">{{ d.job.cwd ?? '—' }}</dd>
          </div>
        </dl>

        <div v-if="d.job.intent">
          <h3 class="text-sm font-semibold">Prompt</h3>
          <p class="mt-1 whitespace-pre-wrap text-sm text-neutral-600 dark:text-neutral-300">
            {{ d.job.intent }}
          </p>
        </div>

        <div v-if="d.job.needs" class="rounded-lg bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300">
          <span class="font-semibold">Waiting on:</span> {{ d.job.needs }}
        </div>

        <div v-if="d.job.children.length">
          <h3 class="text-sm font-semibold">Artifacts</h3>
          <ul class="mt-1 space-y-1 text-sm">
            <li v-for="c in d.job.children" :key="c.id">
              <span class="text-xs text-neutral-500 dark:text-neutral-400">{{ c.kind ?? 'link' }}</span>
              <a v-if="c.href" :href="c.href" target="_blank" rel="noreferrer" class="ml-2 underline">
                {{ c.href }}
              </a>
              <span v-else class="ml-2">{{ c.id }}</span>
            </li>
          </ul>
        </div>

        <div v-if="d.job.output">
          <h3 class="text-sm font-semibold">Output</h3>
          <pre class="mt-1 overflow-x-auto rounded-lg bg-neutral-100 p-3 text-xs dark:bg-neutral-900">{{ d.job.output }}</pre>
        </div>

        <div v-if="d.job.resumeSessionId">
          <RouterLink
            v-if="d.job.cwd"
            :to="`/sessions`"
            class="text-sm underline"
          >
            Session {{ d.job.resumeSessionId.slice(0, 8) }}…
          </RouterLink>
        </div>

        <div>
          <h3 class="text-sm font-semibold">Timeline</h3>
          <ol
            v-if="d.timeline.length"
            class="mt-2 space-y-3 border-l border-neutral-200 pl-4 dark:border-neutral-800"
          >
            <li v-for="(e, i) in d.timeline" :key="i">
              <div class="flex items-baseline gap-2">
                <JobStateBadge v-if="e.state" :state="e.state" />
                <span class="text-xs text-neutral-400">{{ fmt(e.at) }}</span>
              </div>
              <p v-if="e.detail" class="mt-1 text-sm">{{ e.detail }}</p>
              <p v-if="e.text" class="mt-1 whitespace-pre-wrap text-xs text-neutral-500 dark:text-neutral-400">
                {{ e.text }}
              </p>
            </li>
          </ol>
          <p v-else class="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
            No timeline recorded.
          </p>
          <p
            v-if="d.malformedLines"
            class="mt-2 text-xs text-neutral-500 dark:text-neutral-400"
          >
            {{ d.malformedLines }} unreadable event{{ d.malformedLines === 1 ? '' : 's' }} skipped.
          </p>
        </div>

        <p
          v-if="d.job.providerEnvKeys.length"
          class="text-xs text-neutral-500 dark:text-neutral-400"
        >
          Provider environment: {{ d.job.providerEnvKeys.join(', ') }} (values not read)
        </p>
      </section>
    </template>
  </QueryStateBoundary>
</template>
