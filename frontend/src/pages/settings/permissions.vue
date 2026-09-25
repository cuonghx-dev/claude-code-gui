<script setup lang="ts">
import { computed, inject, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import SourceBadge from '@/components/settings/SourceBadge.vue'
import FormField from '@/components/forms/FormField.vue'
import { SETTINGS_CONTEXT } from '@/composables/settingsContext'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
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
const saved = ref('')
const issues = ref<RuleIssue[]>([])

const check = async () => {
  issues.value = await validate.mutateAsync(draft.value)
}

watch(
  () => current.data.value,
  (p) => {
    if (!p) return
    draft.value = JSON.parse(JSON.stringify(p))
    saved.value = JSON.stringify(draft.value)
    issues.value = []
    // Surface problems in rules that are already on disk, not just new ones.
    if (p.allow.length || p.ask.length || p.deny.length) void check()
  },
  { immediate: true },
)

const dirty = computed(() => !!saved.value && JSON.stringify(draft.value) !== saved.value)
useUnsavedChanges(dirty)

const newRule = ref({ allow: '', ask: '', deny: '', additionalDirectories: '' })
type ListKey = 'allow' | 'ask' | 'deny' | 'additionalDirectories'

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
    saved.value = JSON.stringify(draft.value)
    toast.success('Permissions saved')
  } catch (e) {
    toast.error((e as Error)?.message ?? 'Save failed')
  }
}

const LISTS: { key: ListKey; label: string; hint: string; placeholder: string }[] = [
  { key: 'allow', label: 'Allow', hint: 'Runs without asking', placeholder: 'Add rule, e.g. Bash(npm run test:*)' },
  { key: 'ask', label: 'Ask', hint: 'Always prompts, even in modes that would not', placeholder: 'Add rule, e.g. Bash(git push:*)' },
  { key: 'deny', label: 'Deny', hint: 'Never runs', placeholder: 'Add rule, e.g. Read(./.env)' },
  { key: 'additionalDirectories', label: 'Additional directories', hint: 'Extra paths Claude may read and write', placeholder: 'Add directory, e.g. ../shared-lib' },
]

const issuesFor = (rule: string) => issues.value.filter((i) => i.rule === rule)

const KIND_COLOR = { deny: '#B03A2E', ask: '#9A6A12', allow: '#3F7A4E' } as const
const merged = computed(() => {
  const e = effective.data.value
  if (!e) return []
  return (['deny', 'ask', 'allow'] as const).flatMap((kind) =>
    e[kind].map((r) => ({ kind, rule: r.rule, scope: r.scope })),
  )
})
</script>

<template>
  <section
    class="grid content-start gap-6 px-7 py-[22px]"
    style="grid-template-columns: minmax(0, 1fr) 320px"
  >
    <div class="flex min-w-0 flex-col gap-[18px]">
      <div v-for="list in LISTS" :key="list.key" class="flex flex-col gap-2">
        <div class="flex items-baseline gap-2">
          <span class="text-[13px] font-semibold text-ink">{{ list.label }}</span>
          <span class="text-[12px] text-[#A29E94]">{{ list.hint }}</span>
        </div>
        <div class="overflow-hidden rounded-lg border border-[#E6E2DA] bg-white">
          <div
            v-for="(rule, i) in draft[list.key]"
            :key="`${rule}-${i}`"
            class="flex items-center gap-2.5 border-b border-[#F3F0EA] px-3 py-2 last:border-b-0 font-mono text-[12.5px] text-ink"
          >
            <span class="min-w-0 flex-1 break-all">{{ rule }}</span>
            <span
              v-for="(issue, j) in issuesFor(rule)"
              :key="j"
              class="shrink-0 rounded px-[7px] py-0.5 font-sans text-[11.5px]"
              :class="
                issue.severity === 'error'
                  ? 'bg-[#FBE9E6] text-[#B03A2E]'
                  : 'bg-[#FDF3DC] text-[#9A6A12]'
              "
            >
              {{ issue.message }}
            </span>
            <button
              v-if="!readOnly"
              type="button"
              class="rule-remove"
              :aria-label="`Remove ${rule}`"
              @click="remove(list.key, i)"
            >
              ×
            </button>
          </div>
          <input
            v-if="!readOnly"
            v-model="newRule[list.key]"
            type="text"
            class="rule-add"
            :placeholder="list.placeholder"
            :aria-label="`Add to ${list.label}`"
            @keydown.enter.prevent="add(list.key)"
          />
          <div
            v-else-if="!draft[list.key].length"
            class="px-3 py-2 text-[12.5px] text-[#A29E94]"
          >
            None
          </div>
        </div>
      </div>

      <FormField label="Default mode" hint="Scalar setting: the most specific scope that sets it wins">
        <select
          v-model="draft.defaultMode"
          class="ccg-input w-64 font-mono text-[12.5px]"
          :disabled="readOnly"
          @change="check"
        >
          <option :value="null">— inherit —</option>
          <option value="default">default</option>
          <option value="acceptEdits">acceptEdits</option>
          <option value="plan">plan</option>
          <option value="auto">auto</option>
          <option value="dontAsk">dontAsk</option>
          <option value="bypassPermissions">bypassPermissions</option>
        </select>
      </FormField>

      <div class="flex items-center justify-end gap-2">
        <span
          v-if="errors.length"
          class="rounded bg-[#FBE9E6] px-[7px] py-0.5 text-[11.5px] text-[#B03A2E]"
        >
          {{ errors.length }} malformed {{ errors.length === 1 ? 'rule blocks' : 'rules block' }} saving
        </span>
        <button
          type="button"
          class="ccg-btn-primary"
          :disabled="readOnly || !dirty || !!errors.length || put.isPending.value"
          @click="save"
        >
          {{ put.isPending.value ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </div>

    <aside
      class="flex flex-col gap-2.5 self-start rounded-[10px] bg-[#F3F1EC] p-4"
      title="Rule lists merge across settings files, so these all apply regardless of which scope you are editing."
    >
      <div class="text-[13px] font-semibold text-ink">In force everywhere</div>
      <template v-if="effective.data.value">
        <div class="flex items-center gap-1 text-[12px] text-[#6B6860]">
          Mode:
          <span class="font-mono text-ink">{{ effective.data.value.defaultMode ?? 'default' }}</span>
          <SourceBadge
            v-if="effective.data.value.defaultModeSource"
            :scope="effective.data.value.defaultModeSource"
            active
          />
        </div>
        <div
          v-for="r in merged"
          :key="`${r.kind}-${r.scope}-${r.rule}`"
          class="flex items-center gap-2 font-mono text-[11.5px]"
        >
          <span class="w-[38px] shrink-0 font-medium" :style="{ color: KIND_COLOR[r.kind] }">
            {{ r.kind }}
          </span>
          <span class="min-w-0 flex-1 truncate text-ink" :title="r.rule">{{ r.rule }}</span>
          <SourceBadge :scope="r.scope" on-panel />
        </div>
        <p v-if="!merged.length" class="text-[12px] text-[#8A867C]">No rules set in any scope.</p>
      </template>
      <template v-else-if="effective.isPending.value">
        <div v-for="n in 4" :key="n" class="h-3.5 rounded bg-[#E8E5DE]" />
      </template>
    </aside>
  </section>
</template>

<style scoped>
.rule-remove {
  flex-shrink: 0;
  color: #c2bdb2;
  line-height: 1;
  font-size: 14px;
  transition: color 120ms ease-out;
}
.rule-remove:hover {
  color: var(--ccg-error);
}
.rule-add {
  display: block;
  width: 100%;
  padding: 8px 12px;
  border: 0;
  background: transparent;
  font-family: var(--font-mono);
  font-size: 12.5px;
  color: var(--ccg-ink);
  outline: none;
}
.rule-add::placeholder {
  color: #a29e94;
}
.rule-add:focus {
  background: var(--ccg-canvas-soft);
}
</style>
