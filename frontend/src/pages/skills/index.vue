<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import { useSkillImport, useSkillsList } from '@/composables/useSkills'
import { describe } from '@/utils/description'

const { isPending, isError, error, data } = useSkillsList()
const importMut = useSkillImport()
const router = useRouter()
const importError = ref('')

async function importLocal() {
  importError.value = ''
  try {
    const picked = await openDialog({ directory: true, multiple: false, title: 'Pick a skill directory' })
    if (typeof picked !== 'string') return
    const skills = await importMut.mutateAsync({ kind: 'local', path: picked })
    if (skills[0]) router.push(`/skills/${encodeURIComponent(skills[0].slug)}`)
  } catch (e) {
    importError.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader title="Skills" :subtitle="`${data?.length ?? 0} skills (local + plugin)`">
    <template #actions>
      <button type="button" class="ccg-btn-ghost" @click="importLocal">Import folder</button>
      <RouterLink to="/skills/new" class="ccg-btn-primary">+ New</RouterLink>
    </template>
  </PageHeader>
  <p
    v-if="importError"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    Import failed: {{ importError }}
  </p>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: items }">
      <section class="p-6">
        <EmptyState v-if="!items?.length" title="No skills" />
        <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <li v-for="s in items" :key="s.slug">
            <RouterLink :to="`/skills/${s.slug}`" class="block rounded-lg border border-neutral-200 bg-white p-4 hover:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
              <div class="flex items-center gap-2">
                <span class="text-sm font-semibold">{{ s.frontmatter.name ?? s.slug }}</span>
                <span class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400">
                  {{ s.source.kind === 'plugin' ? `plugin: ${s.source.id}` : 'local' }}
                </span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">{{ describe(s.frontmatter.description, s.body) }}</p>
            </RouterLink>
          </li>
        </ul>
      </section>
    </template>
  </QueryStateBoundary>
</template>
