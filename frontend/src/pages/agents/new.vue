<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import EditorPage from '@/components/EditorPage.vue'
import { useAgentImport } from '@/composables/useAgents'
import { useDraftRecovery } from '@/composables/useDraftRecovery'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'

const router = useRouter()
const importMut = useAgentImport()
const errorMessage = ref('')

const TEMPLATE = `---
name: my-agent
description: "Describe when this agent should be used."
tools: Read, Write, Edit, Bash, Glob, Grep
model: sonnet
---

You are ...
`

const content = ref(TEMPLATE)
const initial = TEMPLATE
const dirty = computed(() => content.value !== initial)
useUnsavedChanges(dirty)

const draft = useDraftRecovery<string>('agent:new', content)
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

  importMut
    .mutateAsync({ slug, directory: '', content: trimmed })
    .then((agent) => {
      draft.clear()
      content.value = initial
      router.push(`/agents/${encodeURIComponent(agent.slug)}`)
    })
    .catch((e) => {
      errorMessage.value = (e as { message?: string })?.message ?? String(e)
    })
}

function onCancel() {
  draft.clear()
  content.value = initial
  router.push('/agents')
}
</script>

<template>
  <EditorPage
    v-model="content"
    section="Agents"
    section-to="/agents"
    name="new"
    :dirty="dirty"
    file-path="~/.claude/agents/<name>.md"
    :error="errorMessage"
  >
    <template #actions>
      <button type="button" class="ccg-btn-ghost ccg-btn-sm" @click="onCancel">Cancel</button>
      <button
        type="button"
        class="ccg-btn-primary ccg-btn-sm"
        :disabled="importMut.isPending.value"
        @click="onSubmit"
      >
        {{ importMut.isPending.value ? 'Creating…' : 'Create' }}
      </button>
    </template>
    <template #status>
      <span class="truncate font-sans">Slug is derived from frontmatter <span class="font-mono">name:</span></span>
    </template>
  </EditorPage>
</template>
