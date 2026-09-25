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
import { RefreshCw, Trash2 } from 'lucide-vue-next'
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

function sourceRepo(s: { sourceType: string; url: string }) {
  if (s.sourceType === 'github') {
    return s.url
      .replace(/^https?:\/\/github\.com\//, '')
      .replace(/\.git$/, '')
  }
  return s.url
}

function isInstallable(installUrl: string | null | undefined): boolean {
  if (!installUrl) return false
  return /^(https?:\/\/|git@)/.test(installUrl.trim())
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
  <div class="flex h-full flex-col">
    <PageHeader title="Plugins" subtitle="~/.claude/plugins/">
      <template #actions>
        <div class="ccg-seg" role="group" aria-label="Plugins view">
          <button type="button" :aria-pressed="tab === 'installed'" @click="tab = 'installed'">Installed</button>
          <button type="button" :aria-pressed="tab === 'discover'" @click="tab = 'discover'">Discover</button>
        </div>
        <button
          v-if="tab === 'discover' && !showSourceForm"
          type="button"
          class="ccg-btn-primary"
          @click="showSourceForm = true"
        >
          + Add marketplace
        </button>
      </template>
    </PageHeader>

    <p v-if="errorMessage" class="ccg-alert-error mx-7 mt-4 px-3 py-2 text-[12.5px]" role="alert">
      {{ errorMessage }}
    </p>

    <!-- Installed tab -->
    <section v-if="tab === 'installed'">
      <QueryStateBoundary
        :is-pending="installed.isPending.value"
        :is-error="installed.isError.value"
        :error="installed.error.value"
        :data="installed.data.value"
        skeleton="cards"
      >
        <template #default="{ data: items }">
          <EmptyState v-if="!items?.length" title="No plugins installed.">
            <button type="button" class="ccg-btn-primary" @click="tab = 'discover'">Browse marketplace</button>
          </EmptyState>
          <div v-else class="flex flex-col gap-6 px-7 py-5">
            <div v-for="group in groupByMarketplace(items)" :key="group.name" class="flex flex-col gap-2">
              <div class="flex items-baseline gap-2">
                <span class="ccg-section-label">{{ group.name }}</span>
                <span class="font-mono text-[11px]" style="color: var(--ccg-muted-soft);">{{ group.plugins.length }}</span>
              </div>
              <ul class="grid grid-cols-3 content-start gap-3">
                <li
                  v-for="p in group.plugins"
                  :key="p.id"
                  class="ccg-card ccg-card-hover relative flex min-w-0 flex-col gap-2.5 px-4 py-3.5"
                >
                  <!-- Stretched link: the whole card opens the detail, the switch sits above it. -->
                  <RouterLink
                    :to="`/plugins/${encodeURIComponent(p.id)}`"
                    class="absolute inset-0 rounded-[9px]"
                    :aria-label="`Open ${p.name}`"
                  />
                  <div class="flex items-center gap-2">
                    <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">{{ p.name }}</span>
                    <span v-if="p.version" class="ccg-badge">v{{ p.version }}</span>
                    <button
                      type="button"
                      role="switch"
                      class="ccg-switch relative z-10 disabled:opacity-50"
                      :aria-checked="p.enabled"
                      :aria-label="`Toggle ${p.name}`"
                      :disabled="setEnabled.isPending.value"
                      @click="togglePlugin(p)"
                    />
                  </div>
                  <p
                    class="line-clamp-2 min-h-[38px] text-[13px] leading-[1.45]"
                    style="color: var(--ccg-body); text-wrap: pretty;"
                  >
                    {{ describePlugin(p.description, p.skills) }}
                  </p>
                  <div class="flex items-center gap-1">
                    <span class="ccg-chip">{{ p.skills.length }} skill{{ p.skills.length === 1 ? '' : 's' }}</span>
                    <span v-if="!p.enabled" class="ccg-chip">disabled</span>
                    <span class="flex-1" />
                    <span class="text-[11.5px]" style="color: var(--ccg-muted-soft);">{{ formatDate(p.installedAt) }}</span>
                  </div>
                </li>
              </ul>
            </div>
          </div>
        </template>
      </QueryStateBoundary>
    </section>

    <!-- Discover tab -->
    <section v-else class="flex flex-col gap-6 px-7 py-5">
      <div class="flex flex-col gap-2">
        <div class="flex items-baseline gap-2">
          <span class="text-[13px] font-semibold text-ink">Marketplaces</span>
          <span class="text-[12px]" style="color: var(--ccg-muted-soft);">catalogs of plugins to discover</span>
        </div>

        <div class="ccg-card overflow-hidden" style="border-radius: 8px;">
          <div
            v-for="s in sources.data.value ?? []"
            :key="s.name"
            class="flex items-center gap-3 border-b px-3 py-2"
            style="border-color: var(--ccg-hairline-faint);"
          >
            <span class="w-48 flex-none truncate font-mono text-[12.5px] font-medium text-ink">{{ s.name }}</span>
            <span class="ccg-badge">{{ s.sourceType }}</span>
            <span class="min-w-0 flex-1 truncate font-mono text-[11.5px]" style="color: var(--ccg-subtle);">
              {{ sourceRepo(s) }}
            </span>
            <span class="text-[11.5px]" style="color: var(--ccg-muted-soft);">{{ formatDate(s.lastUpdated) }}</span>
            <button
              type="button"
              class="ccg-btn-ghost ccg-btn-sm"
              :disabled="sourceUpdate.isPending.value"
              @click="refreshSource(s.name)"
            >
              <RefreshCw :size="14" :stroke-width="1.5" :class="sourceUpdate.isPending.value ? 'animate-spin' : ''" />
              Update
            </button>
            <button
              type="button"
              class="ccg-btn-danger ccg-btn-sm px-2"
              :aria-label="`Remove ${s.name}`"
              @click="confirmingSourceRemove = s.name"
            >
              <Trash2 :size="14" :stroke-width="1.5" />
            </button>
          </div>
          <p
            v-if="!sources.data.value?.length && !showSourceForm"
            class="px-3 py-2 text-[12.5px]"
            style="color: var(--ccg-subtle);"
          >
            No marketplaces configured.
          </p>

          <div
            v-if="showSourceForm"
            class="grid grid-cols-[1fr_200px_2fr] items-end gap-3 px-3 py-3"
            style="background: var(--ccg-canvas-soft);"
          >
            <FormField label="Name">
              <input v-model="sourceForm.name" class="ccg-input w-full font-mono" />
            </FormField>
            <FormField label="Type">
              <select v-model="sourceForm.sourceType" class="ccg-input w-full">
                <option value="github">github (git URL)</option>
                <option value="http">http (JSON manifest)</option>
              </select>
            </FormField>
            <FormField label="URL">
              <input v-model="sourceForm.url" class="ccg-input w-full font-mono" />
            </FormField>
            <div class="col-span-3 flex justify-end gap-2">
              <button type="button" class="ccg-btn-ghost" @click="showSourceForm = false">Cancel</button>
              <button
                type="button"
                class="ccg-btn-primary"
                :disabled="sourceAdd.isPending.value || !sourceForm.name || !sourceForm.url"
                @click="addSource"
              >
                {{ sourceAdd.isPending.value ? 'Adding…' : 'Add' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <span class="ccg-section-label">Available plugins</span>
        <QueryStateBoundary
          :is-pending="available.isPending.value"
          :is-error="available.isError.value"
          :error="available.error.value"
          :data="available.data.value"
        >
          <template #loading>
            <div class="grid grid-cols-3 gap-3">
              <div v-for="i in 6" :key="i" class="ccg-skeleton h-[124px]" />
            </div>
          </template>
          <template #default="{ data: items }">
            <EmptyState
              v-if="!items?.length"
              title="No plugins to install. Update a marketplace to fetch its manifest."
            />
            <ul v-else class="grid grid-cols-3 content-start gap-3">
              <li
                v-for="p in items"
                :key="`${p.source}:${p.id}`"
                class="ccg-card flex min-w-0 flex-col gap-2.5 px-4 py-3.5"
              >
                <div class="flex items-center gap-2">
                  <span class="min-w-0 flex-1 truncate font-mono text-[13.5px] font-medium text-ink">{{ p.name }}</span>
                  <span v-if="p.version" class="ccg-badge">v{{ p.version }}</span>
                </div>
                <p
                  class="line-clamp-2 min-h-[38px] text-[13px] leading-[1.45]"
                  style="color: var(--ccg-body); text-wrap: pretty;"
                >
                  {{ p.description ?? '—' }}
                </p>
                <div class="flex items-center gap-2">
                  <span class="ccg-chip">{{ p.source }}</span>
                  <span class="flex-1" />
                  <button
                    v-if="isInstallable(p.installUrl)"
                    type="button"
                    class="ccg-btn-primary ccg-btn-sm"
                    :disabled="installedIds.has(p.id) || install.isPending.value || installFlow.inFlight.value"
                    @click="startInstall(p.id, p.source)"
                  >
                    {{ installedIds.has(p.id) ? 'Installed' : 'Install' }}
                  </button>
                </div>
                <p
                  v-if="!isInstallable(p.installUrl)"
                  class="truncate font-mono text-[11.5px]"
                  style="color: var(--ccg-subtle);"
                  :title="`claude plugins install ${p.id}`"
                >
                  $ claude plugins install {{ p.id }}
                </p>
              </li>
            </ul>
          </template>
        </QueryStateBoundary>
      </div>
    </section>
  </div>

  <!-- Install progress modal -->
  <Teleport to="body">
    <div
      v-if="installingPlugin"
      class="fixed inset-0 z-50 flex items-start justify-center pt-[128px]"
      style="background: rgba(31, 30, 27, .18);"
    >
      <div
        class="w-[440px] overflow-hidden bg-white"
        style="border-radius: 12px; box-shadow: 0 24px 60px rgba(40,30,20,.3), 0 0 0 1px rgba(0,0,0,.08);"
        role="dialog"
        aria-modal="true"
      >
        <div class="flex flex-col gap-1 border-b px-4 py-3.5" style="border-color: var(--ccg-hairline-soft);">
          <h3 class="text-[14px] font-semibold text-ink">
            Installing <span class="font-mono">{{ installingPlugin.name }}</span>
          </h3>
          <p class="text-[12px]" style="color: var(--ccg-subtle);">
            from <span class="font-mono">{{ installingPlugin.source }}</span>
          </p>
        </div>
        <div class="flex flex-col gap-2 px-4 py-4">
          <p class="font-mono text-[12px]" style="color: var(--ccg-body);">{{ installFlow.step.value || 'starting…' }}</p>
          <div class="h-1.5 overflow-hidden rounded-full" style="background: var(--ccg-surface-strong);">
            <div
              class="h-full rounded-full transition-all"
              style="background: var(--ccg-accent-fill);"
              :style="{ width: `${installFlow.percent.value ?? 0}%` }"
            />
          </div>
          <p v-if="installFlow.errorMessage.value" class="ccg-alert-error mt-1 px-3 py-2 text-[12.5px]">
            {{ installFlow.errorMessage.value }}
          </p>
        </div>
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
