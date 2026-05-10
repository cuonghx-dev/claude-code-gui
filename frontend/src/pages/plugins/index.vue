<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import FormField from '@/components/forms/FormField.vue'
import { usePluginsList } from '@/composables/usePlugins'
import {
  useMarketplaceAvailable,
  useMarketplaceInstall,
  useMarketplaceSourceAdd,
  useMarketplaceSourceRemove,
  useMarketplaceSourceUpdate,
  useMarketplaceSources,
} from '@/composables/useMarketplace'
import { useAsyncRequest } from '@/composables/useAsyncRequest'

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
        <ul v-else class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <li v-for="p in items" :key="p.id">
            <RouterLink :to="`/plugins/${p.id}`" class="block rounded-lg border border-neutral-200 bg-white p-4 hover:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-sm font-semibold">{{ p.name }}</span>
                <span v-if="p.version" class="text-xs text-neutral-400">v{{ p.version }}</span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs text-neutral-500 dark:text-neutral-400">{{ p.description ?? '—' }}</p>
              <p class="mt-2 text-[11px] text-neutral-400">
                {{ p.skills.length }} skill{{ p.skills.length === 1 ? '' : 's' }} ·
                <span :class="p.enabled ? 'text-emerald-600 dark:text-emerald-400' : 'text-neutral-500'">
                  {{ p.enabled ? 'enabled' : 'disabled' }}
                </span>
              </p>
            </RouterLink>
          </li>
        </ul>
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
