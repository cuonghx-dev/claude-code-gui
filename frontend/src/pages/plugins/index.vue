<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import FormField from '@/components/forms/FormField.vue'
import { usePluginSetEnabled, usePluginsList } from '@/composables/usePlugins'
import {
  useMarketplaceAvailable,
  useMarketplaceInstall,
  useMarketplaceSourceAdd,
  useMarketplaceSourceRemove,
  useMarketplaceSourceUpdate,
  useMarketplaceSources,
} from '@/composables/useMarketplace'
import { useAsyncRequest } from '@/composables/useAsyncRequest'
import { describePlugin } from '@/utils/description'
import { ChevronRight, Store } from 'lucide-vue-next'
import type { Plugin } from '@/types/ipc'

const route = useRoute()
const router = useRouter()

const tab = ref<'installed' | 'discover'>('installed')

const installed = usePluginsList()
const sources = useMarketplaceSources()
const available = useMarketplaceAvailable()
const sourceAdd = useMarketplaceSourceAdd()
const sourceRemove = useMarketplaceSourceRemove()
const sourceUpdate = useMarketplaceSourceUpdate()
const install = useMarketplaceInstall()
const installFlow = useAsyncRequest()

const showSourceForm = ref(false)
const sourceForm = reactive({
  name: '',
  sourceType: 'github' as 'github' | 'http',
  url: '',
})
const errorMessage = ref('')
const installingPlugin = ref<{ name: string; source: string } | null>(null)
const confirmingSourceRemove = ref<string | null>(null)

async function addSource() {
  errorMessage.value = ''
  try {
    await sourceAdd.mutateAsync({
      name: sourceForm.name.trim(),
      sourceType: sourceForm.sourceType,
      url: sourceForm.url.trim(),
    })
    sourceForm.name = ''
    sourceForm.url = ''
    showSourceForm.value = false
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function refreshSource(name: string) {
  errorMessage.value = ''
  try {
    await sourceUpdate.mutateAsync(name)
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function removeSource() {
  if (!confirmingSourceRemove.value) return
  errorMessage.value = ''
  try {
    await sourceRemove.mutateAsync(confirmingSourceRemove.value)
    confirmingSourceRemove.value = null
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function startInstall(name: string, source: string) {
  errorMessage.value = ''
  installingPlugin.value = { name, source }
  try {
    const requestId = await install.mutateAsync({ name, source })
    await installFlow.start('marketplace:install', String(requestId))
    installingPlugin.value = null
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
    installingPlugin.value = null
  }
}

const installedIds = computed(
  () => new Set((installed.data.value ?? []).map((p) => p.id)),
)

const setEnabled = usePluginSetEnabled()

function groupByMarketplace(items: Plugin[]): { name: string; plugins: Plugin[] }[] {
  const map = new Map<string, Plugin[]>()
  for (const p of items) {
    const key = p.marketplace ?? '(local)'
    const arr = map.get(key) ?? []
    arr.push(p)
    map.set(key, arr)
  }
  return [...map.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, plugins]) => ({
      name,
      plugins: plugins.slice().sort((a, b) => a.name.localeCompare(b.name)),
    }))
}

const dateFmt = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
function formatDate(iso: string | null) {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  return dateFmt.format(d)
}

async function togglePlugin(p: Plugin) {
  errorMessage.value = ''
  try {
    await setEnabled.mutateAsync({ id: p.id, enabled: !p.enabled })
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

// Deep-link auto-install: `/plugins?autoInstall=foo&source=bar` switches
// to Discover and kicks the install flow exactly once. Removes the
// query params after triggering so a refresh doesn't re-run it.
async function handleAutoInstall() {
  const auto = route.query.autoInstall
  const source = route.query.source
  if (typeof auto !== 'string' || typeof source !== 'string') return
  tab.value = 'discover'
  void router.replace({ query: {} })
  await startInstall(auto, source)
}

onMounted(() => {
  void handleAutoInstall()
})
watch(
  () => [route.query.autoInstall, route.query.source],
  () => void handleAutoInstall(),
)
</script>

<template>
  <PageHeader title="Plugins" subtitle="Installed plugins + marketplace discovery">
    <template #actions>
      <button
        type="button"
        class="rounded-md px-3 py-1.5 text-sm"
        :class="tab === 'installed' ? 'bg-violet-600 text-white' : 'border border-neutral-300 dark:border-neutral-700'"
        @click="tab = 'installed'"
      >
        Installed
      </button>
      <button
        type="button"
        class="rounded-md px-3 py-1.5 text-sm"
        :class="tab === 'discover' ? 'bg-violet-600 text-white' : 'border border-neutral-300 dark:border-neutral-700'"
        @click="tab = 'discover'"
      >
        Discover
      </button>
    </template>
  </PageHeader>

  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>

  <!-- Installed tab -->
  <section v-if="tab === 'installed'" class="p-6">
    <QueryStateBoundary
      :is-pending="installed.isPending.value"
      :is-error="installed.isError.value"
      :error="installed.error.value"
      :data="installed.data.value"
    >
      <template #default="{ data: items }">
        <EmptyState v-if="!items?.length" title="No plugins installed" hint="Switch to Discover to browse." />
        <div v-else class="space-y-6">
          <div v-for="group in groupByMarketplace(items)" :key="group.name">
            <header class="mb-2 flex items-center gap-2 text-xs">
              <Store class="h-4 w-4 text-neutral-500" />
              <code class="font-mono text-sm">{{ group.name }}</code>
              <span class="text-neutral-500">{{ group.plugins.length }}</span>
            </header>
            <ul class="divide-y divide-neutral-200 overflow-hidden rounded-lg border border-neutral-200 bg-white dark:divide-neutral-800 dark:border-neutral-800 dark:bg-neutral-900">
              <li
                v-for="p in group.plugins"
                :key="p.id"
                class="flex items-center gap-4 px-4 py-3 hover:bg-neutral-50 dark:hover:bg-neutral-800/60"
              >
                <button
                  type="button"
                  role="switch"
                  :aria-checked="p.enabled"
                  :aria-label="`Toggle ${p.name}`"
                  :disabled="setEnabled.isPending.value"
                  class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition disabled:opacity-50"
                  :class="p.enabled ? 'bg-amber-500' : 'bg-neutral-300 dark:bg-neutral-700'"
                  @click="togglePlugin(p)"
                >
                  <span
                    class="inline-block h-4 w-4 transform rounded-full bg-white shadow transition"
                    :class="p.enabled ? 'translate-x-[1.125rem]' : 'translate-x-0.5'"
                  />
                </button>
                <RouterLink
                  :to="`/plugins/${encodeURIComponent(p.id)}`"
                  class="flex min-w-0 flex-1 items-center gap-4"
                >
                  <span class="w-44 shrink-0 truncate text-sm font-semibold">{{ p.name }}</span>
                  <span
                    v-if="p.version"
                    class="shrink-0 rounded bg-neutral-100 px-2 py-0.5 font-mono text-[11px] text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300"
                  >v{{ p.version }}</span>
                  <span class="line-clamp-1 flex-1 text-xs text-neutral-500 dark:text-neutral-400">
                    {{ describePlugin(p.description, p.skills) }}
                  </span>
                  <span class="shrink-0 text-[11px] text-neutral-500">
                    {{ p.skills.length }} skill{{ p.skills.length === 1 ? '' : 's' }}
                  </span>
                  <span class="w-24 shrink-0 text-right text-[11px] text-neutral-500">
                    {{ formatDate(p.installedAt) }}
                  </span>
                  <ChevronRight class="h-4 w-4 shrink-0 text-neutral-400" />
                </RouterLink>
              </li>
            </ul>
          </div>
        </div>
      </template>
    </QueryStateBoundary>
  </section>

  <!-- Discover tab -->
  <section v-else class="p-6 space-y-6">
    <div class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Sources</h3>
        <button type="button" class="ccg-btn-ghost text-xs" @click="showSourceForm = !showSourceForm">
          {{ showSourceForm ? 'Cancel' : '+ Add source' }}
        </button>
      </div>

      <div v-if="showSourceForm" class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-3">
        <FormField label="Name">
          <input v-model="sourceForm.name" class="ccg-input" />
        </FormField>
        <FormField label="Type">
          <select v-model="sourceForm.sourceType" class="ccg-input">
            <option value="github">github (git URL)</option>
            <option value="http">http (JSON manifest)</option>
          </select>
        </FormField>
        <FormField label="URL">
          <input v-model="sourceForm.url" class="ccg-input" />
        </FormField>
        <div class="md:col-span-3">
          <button
            type="button"
            class="ccg-btn-primary"
            :disabled="sourceAdd.isPending.value || !sourceForm.name || !sourceForm.url"
            @click="addSource"
          >
            {{ sourceAdd.isPending.value ? 'Adding…' : 'Add source' }}
          </button>
        </div>
      </div>

      <ul v-if="sources.data.value?.length" class="mt-3 divide-y divide-neutral-200 dark:divide-neutral-800">
        <li v-for="s in sources.data.value" :key="s.name" class="flex items-center gap-3 py-2 text-sm">
          <div class="flex-1">
            <div class="font-semibold">{{ s.name }}</div>
            <div class="font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
              {{ s.sourceType }} · {{ s.url }}
            </div>
            <div v-if="s.lastUpdated" class="text-[11px] text-neutral-400">
              Updated {{ s.lastUpdated.slice(0, 16) }} · {{ s.plugins.length }} plugins
            </div>
          </div>
          <button
            type="button"
            class="ccg-btn-ghost text-xs"
            :disabled="sourceUpdate.isPending.value"
            @click="refreshSource(s.name)"
          >
            Refresh
          </button>
          <button
            type="button"
            class="text-xs text-red-600 hover:underline dark:text-red-400"
            @click="confirmingSourceRemove = s.name"
          >
            Remove
          </button>
        </li>
      </ul>
      <p v-else class="mt-3 text-xs text-neutral-500 dark:text-neutral-400">
        No sources configured. Add a github or http source to discover plugins.
      </p>
    </div>

    <div>
      <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide text-neutral-500">Available plugins</h3>
      <QueryStateBoundary
        :is-pending="available.isPending.value"
        :is-error="available.isError.value"
        :error="available.error.value"
        :data="available.data.value"
      >
        <template #default="{ data: items }">
          <EmptyState
            v-if="!items?.length"
            title="No plugins to install"
            hint="Refresh a source to fetch its manifest."
          />
          <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <li
              v-for="p in items"
              :key="`${p.source}:${p.id}`"
              class="rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-900"
            >
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.name }}</span>
                <span v-if="p.version" class="text-xs text-neutral-400">v{{ p.version }}</span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">{{ p.description ?? '—' }}</p>
              <p class="mt-2 text-[11px] text-neutral-400">From source: {{ p.source }}</p>
              <button
                type="button"
                class="mt-3 ccg-btn-primary text-xs"
                :disabled="installedIds.has(p.id) || install.isPending.value || installFlow.inFlight.value"
                @click="startInstall(p.id, p.source)"
              >
                {{ installedIds.has(p.id) ? 'Installed' : 'Install' }}
              </button>
            </li>
          </ul>
        </template>
      </QueryStateBoundary>
    </div>
  </section>

  <!-- Install progress modal -->
  <Teleport to="body">
    <div
      v-if="installingPlugin"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
    >
      <div class="w-[420px] rounded-lg border border-neutral-200 bg-white p-5 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <h3 class="text-sm font-semibold">Installing {{ installingPlugin.name }}</h3>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          From source <code>{{ installingPlugin.source }}</code>
        </p>
        <div class="mt-4">
          <p class="text-xs font-mono">{{ installFlow.step.value || 'starting…' }}</p>
          <div class="mt-2 h-2 overflow-hidden rounded bg-neutral-100 dark:bg-neutral-800">
            <div
              class="h-full bg-violet-600 transition-all"
              :style="{ width: `${installFlow.percent.value ?? 0}%` }"
            />
          </div>
        </div>
        <p
          v-if="installFlow.errorMessage.value"
          class="mt-3 text-xs text-red-600 dark:text-red-400"
        >
          {{ installFlow.errorMessage.value }}
        </p>
      </div>
    </div>
  </Teleport>

  <ConfirmDialog
    :open="!!confirmingSourceRemove"
    title="Remove source?"
    :message="confirmingSourceRemove ? `Remove '${confirmingSourceRemove}' from .marketplaces.json. Installed plugins are not affected.` : ''"
    confirm-label="Remove"
    danger
    @update:open="(v: boolean) => { if (!v) confirmingSourceRemove = null }"
    @confirm="removeSource"
  />
</template>
