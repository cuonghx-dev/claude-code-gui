<script setup lang="ts">
import RelationshipsPanel from '@/components/RelationshipsPanel.vue'
import EditorPage from '@/components/EditorPage.vue'
import { useRelatedCount } from '@/composables/useRelationships'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useSkill,
  useSkillDelete,
  useSkillExport,
  useSkillReadRaw,
  useSkillUpdateRaw,
} from '@/composables/useSkills'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)
const relatedCount = useRelatedCount('skill', slug)
const showRelated = ref(false)

const { isPending, isError, error, data } = useSkill(slug)
const update = useSkillUpdateRaw()
const remove = useSkillDelete()
const exportMut = useSkillExport()
const readRaw = useSkillReadRaw()

const errorMessage = ref('')
const confirmingDelete = ref(false)
const content = ref('')
const initial = ref('')

const isLocal = computed(() => data.value?.source.kind === 'local')
const dirty = computed(() => content.value !== initial.value)
useUnsavedChanges(dirty)

watch(
  data,
  async (s) => {
    if (!s) return
    try {
      const raw = await readRaw.mutateAsync(s.slug)
      content.value = raw
      initial.value = raw
    } catch (e) {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    }
  },
  { immediate: true },
)

async function onSave() {
  errorMessage.value = ''
  try {
    await update.mutateAsync({ slug: slug.value, content: content.value })
    initial.value = content.value
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
    initial.value = content.value
    router.push('/skills')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onExport() {
  errorMessage.value = ''
  try {
    const bytes = await exportMut.mutateAsync(slug.value)
    const blob = new Blob([new Uint8Array(bytes)], { type: 'application/x-tar' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${slug.value}.tar`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: skill }">
      <EditorPage
        v-if="skill"
        v-model="content"
        section="Skills"
        section-to="/skills"
        :name="skill.frontmatter.name ?? slug"
        :dirty="dirty"
        :file-path="skill.filePath"
        :error="errorMessage"
      >
        <template #actions>
          <span v-if="skill.source.kind === 'plugin'" class="ccg-badge">plugin: {{ skill.source.id }}</span>
          <button
            type="button"
            class="ccg-btn-ghost ccg-btn-sm"
            :aria-pressed="showRelated"
            :disabled="relatedCount === 0"
            @click="showRelated = !showRelated"
          >
            Relationships ({{ relatedCount }})
          </button>
          <template v-if="isLocal">
            <button type="button" class="ccg-btn-ghost ccg-btn-sm" @click="onExport">Export</button>
            <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirmingDelete = true">Delete</button>
            <button
              type="button"
              class="ccg-btn-primary ccg-btn-sm"
              :disabled="!dirty || update.isPending.value"
              @click="onSave"
            >
              {{ update.isPending.value ? 'Saving…' : 'Save' }}
            </button>
          </template>
        </template>
        <template #banner>
          <p v-if="!isLocal" class="ccg-alert-warn mx-5 mt-3 flex-none px-3 py-2 text-[12.5px]">
            Plugin-bundled skills are read-only. Edit the source plugin to change them.
          </p>
        </template>
        <template #panel>
          <RelationshipsPanel v-if="showRelated && relatedCount > 0" :skill-slug="slug" />
        </template>
      </EditorPage>
    </template>
  </QueryStateBoundary>

  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete skill?"
    :message="`This permanently removes the skill directory '${slug}/' from disk.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
