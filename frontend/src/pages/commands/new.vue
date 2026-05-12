<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import MarkdownEditor from '@/components/MarkdownEditor.vue'
import { useCommandImportRaw } from '@/composables/useCommands'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const importMut = useCommandImportRaw()
const errorMessage = ref('')

const TEMPLATE = `---
name: my-command
description: "Describe what this command does."
argument-hint: "[args]"
allowed-tools: [Read, Bash]
---

Run the command using {{args}}.
`

const content = ref(TEMPLATE)
const initial = TEMPLATE
const dirty = computed(() => content.value !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<string>('command:new', content)
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
  if (!/^---\r?\n/.test(trimmed)) {
    errorMessage.value = 'Markdown must start with YAML frontmatter (---).'
    return
  }
  const name = sniffName(trimmed)
  if (!name) {
    errorMessage.value = 'Add `name:` to the frontmatter; it becomes the slug.'
    return
  }
  const slug = slugify(name)
  if (!/^[a-z0-9_]+(?:-[a-z0-9_]+)*$/.test(slug)) {
    errorMessage.value = `Derived slug "${slug}" is invalid. Use lowercase letters, digits, "-" or "_" in name.`
    return
  }

  importMut
    .mutateAsync({ slug, directory: '', content: trimmed })
    .then((c) => {
      draft.clear()
      content.value = initial
      router.push(`/commands/${encodeURIComponent(c.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  content.value = initial
  router.push('/commands')
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <PageHeader title="New command" subtitle="Paste a slash-command markdown. Slug is derived from frontmatter `name:`.">
      <template #actions>
        <button type="button" class="ccg-btn-ghost" @click="onCancel">Cancel</button>
        <button
          type="button"
          class="ccg-btn-primary"
          :disabled="importMut.isPending.value"
          @click="onSubmit"
        >
          {{ importMut.isPending.value ? 'Creating…' : 'Create' }}
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
