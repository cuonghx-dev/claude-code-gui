<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { useSkillCreateRaw } from '@/composables/useSkills'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const createMut = useSkillCreateRaw()
const errorMessage = ref('')

const TEMPLATE = `---
name: my-skill
description: "Describe when this skill should be used."
context: when
---

Body of the skill goes here.
`

const content = ref(TEMPLATE)
const initial = TEMPLATE
const dirty = computed(() => content.value !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<string>('skill:new', content)
const recovered = draft.load()
if (typeof recovered === 'string') content.value = recovered

function sniffName(c: string): string | undefined {
  const fmMatch = c.match(/^---\r?\n([\s\S]*?)\r?\n---/)
  if (!fmMatch) return undefined
  const nameMatch = fmMatch[1].match(/^name:\s*["']?([^\r\n"']+)["']?\s*$/m)
  return nameMatch?.[1]?.trim()
}

function slugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64)
}

function onSubmit() {
  errorMessage.value = ''
  const trimmed = content.value.replace(/^﻿/, '').trimStart()
  const name = sniffName(trimmed)
  const slug = name ? slugify(name) : `untitled-${Date.now()}`

  createMut
    .mutateAsync({ slug, content: trimmed })
    .then((s) => {
      draft.clear()
      content.value = initial
      router.push(`/skills/${encodeURIComponent(s.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  content.value = initial
  router.push('/skills')
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <PageHeader title="New skill" subtitle="SKILL.md under ~/.claude/skills/<slug>/. Slug is derived from frontmatter `name:`.">
      <template #actions>
        <button type="button" class="ccg-btn-ghost" @click="onCancel">Cancel</button>
        <button
          type="button"
          class="ccg-btn-primary"
          :disabled="createMut.isPending.value"
          @click="onSubmit"
        >
          {{ createMut.isPending.value ? 'Creating…' : 'Create' }}
        </button>
      </template>
    </PageHeader>
    <section class="flex min-h-0 flex-1 flex-col p-6">
      <p
        v-if="errorMessage"
        class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
      >
        {{ errorMessage }}
      </p>
      <MarkdownEditor v-model="content" fill class="min-h-0 flex-1" />
    </section>
  </div>
</template>
