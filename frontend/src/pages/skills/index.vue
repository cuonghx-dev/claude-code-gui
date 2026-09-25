<script setup lang="ts">
import { computed, ref } from 'vue'
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
const githubOpen = ref(false)
const githubUrl = ref('')
const search = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const items = data.value ?? []
  if (!q) return items
  return items.filter(
    (s) =>
      s.slug.toLowerCase().includes(q) ||
      (s.frontmatter.name?.toLowerCase().includes(q) ?? false) ||
      (s.frontmatter.description?.toLowerCase().includes(q) ?? false),
  )
})

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

async function importGithub() {
  const url = githubUrl.value.trim()
  if (!url) return
  importError.value = ''
  try {
    const skills = await importMut.mutateAsync({ kind: 'github', url })
    githubOpen.value = false
    githubUrl.value = ''
    if (skills[0]) router.push(`/skills/${encodeURIComponent(skills[0].slug)}`)
  } catch (e) {
    importError.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="Skills" subtitle="~/.claude/skills/*/SKILL.md">
      <template #actions>
        <input v-model="search" placeholder="Filter…" class="ccg-input w-[220px]" />
        <button type="button" class="ccg-btn-ghost" @click="importLocal">Import folder</button>
        <button
          type="button"
          class="ccg-btn-ghost"
          :aria-expanded="githubOpen"
          :aria-pressed="githubOpen"
          @click="githubOpen = !githubOpen"
        >
          Import from GitHub
        </button>
        <RouterLink to="/skills/new" class="ccg-btn-primary">+ New</RouterLink>
      </template>
    </PageHeader>
    <form
      v-if="githubOpen"
      class="ccg-card mx-7 mt-4 flex items-center gap-2 p-3"
      @submit.prevent="importGithub"
    >
      <input
        v-model="githubUrl"
        type="url"
        required
        class="ccg-input flex-1 font-mono text-[12px]"
        placeholder="https://github.com/owner/repo/tree/main/path/to/skill"
        aria-label="GitHub URL of the skill directory"
      />
      <button type="submit" class="ccg-btn-primary" :disabled="importMut.isPending.value">
        {{ importMut.isPending.value ? 'Importing…' : 'Import' }}
      </button>
    </form>
    <p v-if="importError" class="ccg-alert-error mx-7 mt-4 px-3 py-2 text-[12.5px]" role="alert">
      Import failed: {{ importError }}
    </p>

    <QueryStateBoundary
      :is-pending="isPending"
      :is-error="isError"
      :error="error"
      :data="filtered"
      skeleton="cards"
    >
      <template #default="{ data: items }">
        <EmptyState
          v-if="!items?.length"
          :title="search.trim() ? `No skills match “${search.trim()}”.` : 'No skills yet.'"
        >
          <RouterLink v-if="!search.trim()" to="/skills/new" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <ul v-else class="grid grid-cols-3 content-start gap-3 px-7 py-5">
          <li v-for="s in items" :key="s.slug" class="min-w-0">
            <RouterLink
              :to="`/skills/${encodeURIComponent(s.slug)}`"
              class="ccg-card ccg-card-hover flex h-full flex-col gap-2.5 px-4 py-3.5"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">
                  {{ s.frontmatter.name ?? s.slug }}
                </span>
                <span
                  class="ccg-badge"
                  :style="s.source.kind === 'local' ? 'background: var(--ccg-success-bg); color: var(--ccg-success);' : undefined"
                >
                  {{ s.source.kind === 'plugin' ? 'plugin' : 'local' }}
                </span>
              </div>
              <p
                class="line-clamp-3 min-h-[38px] text-[13px] leading-[1.45]"
                style="color: var(--ccg-body); text-wrap: pretty;"
              >
                {{ describe(s.frontmatter.description, s.body) }}
              </p>
              <div
                v-if="s.source.kind === 'plugin' || s.frontmatter.context || s.frontmatter.agent"
                class="flex flex-wrap gap-1"
              >
                <span v-if="s.source.kind === 'plugin'" class="ccg-chip max-w-full truncate">{{ s.source.id }}</span>
                <span v-if="s.frontmatter.context" class="ccg-chip">context: {{ s.frontmatter.context }}</span>
                <span v-if="s.frontmatter.agent" class="ccg-chip">agent: {{ s.frontmatter.agent }}</span>
              </div>
            </RouterLink>
          </li>
        </ul>
      </template>
    </QueryStateBoundary>
  </div>
</template>
