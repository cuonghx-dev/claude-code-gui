<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import SkillForm from '@/components/forms/SkillForm.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useSkill,
  useSkillDelete,
  useSkillExport,
  useSkillUpdate,
} from '@/composables/useSkills'
import type { SkillInput } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const slug = computed(() => (route.params as { slug: string }).slug)

const { isPending, isError, error, data } = useSkill(slug)
const update = useSkillUpdate()
const remove = useSkillDelete()
const exportMut = useSkillExport()

const errorMessage = ref('')
const confirmingDelete = ref(false)

const isLocal = computed(() => data.value?.source.kind === 'local')

async function onSubmit(input: SkillInput) {
  errorMessage.value = ''
  try {
    const next = await update.mutateAsync({ slug: slug.value, input })
    if (next.slug !== slug.value) {
      router.replace(`/skills/${encodeURIComponent(next.slug)}`)
    }
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(slug.value)
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
  <PageHeader
    :title="data?.frontmatter?.name ?? slug"
    :subtitle="data?.source.kind === 'plugin' ? `From plugin ${data.source.id} (read-only)` : data?.directory"
  >
    <template #actions>
      <button v-if="isLocal" type="button" class="ccg-btn-ghost" @click="onExport">Export</button>
      <button
        v-if="isLocal"
        type="button"
        class="ccg-btn-danger"
        @click="confirmingDelete = true"
      >
        Delete
      </button>
    </template>
  </PageHeader>

  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: skill }">
      <section v-if="skill" class="p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <p
          v-if="!isLocal"
          class="mb-4 rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200"
        >
          Plugin-bundled skills are read-only. Edit the source plugin to change them.
        </p>
        <template v-if="isLocal">
          <SkillForm
            :draft-key="`skill:${skill.slug}`"
            :initial="{
              slug: skill.slug,
              frontmatter: skill.frontmatter,
              body: skill.body,
            }"
            :submitting="update.isPending.value"
            submit-label="Save changes"
            @submit="onSubmit"
            @cancel="router.push('/skills')"
          />
        </template>
        <template v-else>
          <pre class="max-h-[70vh] overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ skill.body }}</pre>
        </template>
      </section>
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
