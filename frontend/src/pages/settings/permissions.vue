<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import SourceBadge from '@/components/settings/SourceBadge.vue'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import {
  useEffectivePermissions,
  usePermissions,
  usePermissionsPut,
  useValidatePermissions,
} from '@/composables/usePermissions'
import type { Permissions, RuleIssue } from '@/types/ipc'

const ctx = inject(SETTINGS_CONTEXT)!
const current = usePermissions(ctx.scope, ctx.workingDir)
const effective = useEffectivePermissions(ctx.workingDir)
const put = usePermissionsPut()
const validate = useValidatePermissions()

const draft = ref<Permissions>({
  allow: [],
  ask: [],
  deny: [],
  additionalDirectories: [],
  defaultMode: null,
  disableBypassPermissionsMode: null,
  disableAutoMode: null,
})
const issues = ref<RuleIssue[]>([])

watch(
  () => current.data.value,
  (p) => {
    if (p) draft.value = JSON.parse(JSON.stringify(p))
  },
  { immediate: true },
)

const newRule = ref({ allow: '', ask: '', deny: '', additionalDirectories: '' })
type ListKey = 'allow' | 'ask' | 'deny' | 'additionalDirectories'

const check = async () => {
  issues.value = await validate.mutateAsync(draft.value)
}

const add = async (key: ListKey) => {
  const value = newRule.value[key].trim()
  if (!value) return
  draft.value[key] = [...draft.value[key], value]
  newRule.value[key] = ''
  await check()
}

const remove = async (key: ListKey, i: number) => {
  draft.value[key] = draft.value[key].filter((_, idx) => idx !== i)
  await check()
}

const readOnly = computed(() => ctx.scope.value === 'managed')
const errors = computed(() => issues.value.filter((i) => i.severity === 'error'))

const save = async () => {
  await check()
  if (errors.value.length) {
    toast.error('Fix the malformed rules first')
    return
  }
  try {
    await put.mutateAsync({
      scope: ctx.scope.value,
      workingDir: ctx.workingDir.value,
      permissions: draft.value,
    })
    toast.success('Permissions saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}

const LISTS: { key: ListKey; label: string; hint: string }[] = [
  { key: 'allow', label: 'Allow', hint: 'Runs without asking, e.g. Bash(npm run test:*)' },
  { key: 'ask', label: 'Ask', hint: 'Always prompts, even in modes that would not' },
  { key: 'deny', label: 'Deny', hint: 'Never runs, e.g. Read(./.env)' },
  { key: 'additionalDirectories', label: 'Additional directories', hint: 'Extra paths Claude may read and write' },
]

const issuesFor = (rule: string) => issues.value.filter((i) => i.rule === rule)
</script>

<template>
  <section class="space-y-6 p-6">
    <div v-for="list in LISTS" :key="list.key">
      <h3 class="text-sm font-semibold">{{ list.label }}</h3>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">{{ list.hint }}</p>
      <ul class="mt-2 space-y-1">
        <li v-for="(rule, i) in draft[list.key]" :key="`${rule}-${i}`" class="flex items-center gap-2">
          <span class="font-mono text-xs">{{ rule }}</span>
          <span
            v-for="(issue, j) in issuesFor(rule)"
            :key="j"
            class="text-[11px]"
            :class="issue.severity === 'error' ? 'text-red-600 dark:text-red-400' : 'text-amber-600 dark:text-amber-400'"
          >
            {{ issue.message }}
          </span>
          <button
            type="button"
            class="ccg-btn-ghost ml-auto text-xs"
            :disabled="readOnly"
            @click="remove(list.key, i)"
          >
            Remove
          </button>
        </li>
      </ul>
      <div class="mt-2 flex gap-2">
        <input
          v-model="newRule[list.key]"
          type="text"
          class="ccg-input font-mono text-xs"
          :placeholder="list.key === 'additionalDirectories' ? '../shared-lib' : 'Tool(specifier)'"
          :disabled="readOnly"
          @keyup.enter="add(list.key)"
        />
        <button type="button" class="ccg-btn-ghost" :disabled="readOnly" @click="add(list.key)">
          Add
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
      <label class="flex flex-col gap-1 text-xs text-neutral-500 dark:text-neutral-400">
        Default mode
        <select v-model="draft.defaultMode" class="ccg-input" :disabled="readOnly" @change="check">
          <option :value="null">— inherit —</option>
          <option value="default">default</option>
          <option value="acceptEdits">acceptEdits</option>
          <option value="plan">plan</option>
          <option value="auto">auto</option>
          <option value="dontAsk">dontAsk</option>
          <option value="bypassPermissions">bypassPermissions</option>
        </select>
      </label>
    </div>

    <button type="button" class="ccg-btn-primary" :disabled="readOnly || put.isPending.value" @click="save">
      {{ put.isPending.value ? 'Saving…' : 'Save permissions' }}
    </button>

    <div v-if="effective.data.value">
      <h3 class="text-sm font-semibold">In force everywhere</h3>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        Rule lists merge across settings files, so these all apply regardless of which scope you are
        editing.
      </p>
      <ul class="mt-2 space-y-1 text-xs">
        <li
          v-for="r in [
            ...effective.data.value.deny,
            ...effective.data.value.ask,
            ...effective.data.value.allow,
          ]"
          :key="`${r.scope}-${r.rule}`"
          class="flex items-center gap-2"
        >
          <SourceBadge :scope="r.scope" />
          <span class="font-mono">{{ r.rule }}</span>
        </li>
      </ul>
      <p class="mt-2 text-xs text-neutral-500 dark:text-neutral-400">
        Mode:
        <span class="font-mono">{{ effective.data.value.defaultMode ?? 'default' }}</span>
        <template v-if="effective.data.value.defaultModeSource">
          from
          <SourceBadge :scope="effective.data.value.defaultModeSource" />
        </template>
      </p>
    </div>
  </section>
</template>
