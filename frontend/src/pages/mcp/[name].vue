<script setup lang="ts">
// Master-detail MCP page. `/mcp` (index.vue) renders this with no server
// selected; `/mcp/<name>` selects one. Scope/project ride along in the query.
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { Loader2 } from 'lucide-vue-next'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  useMcpCapabilities,
  useMcpDelete,
  useMcpList,
  useMcpScope,
  useMcpServer,
} from '@/composables/useMcp'
import { useProjectsList } from '@/composables/useProjects'
import type { McpServer } from '@/types/ipc'

const route = useRoute()
const router = useRouter()
const name = computed(() => (route.params as { name?: string }).name ?? '')
const { scope, projectName, workingDir, ready, query, fileLabel } = useMcpScope()
const list = useMcpList(scope, workingDir, ready)
const server = useMcpServer(name, scope, workingDir, ready)
const projects = useProjectsList()

// Only projects with a known working dir can hold a .mcp.json.
const projectOptions = computed(() =>
  (projects.data.value ?? []).filter((p) => !!p.workingDir),
)

function setScope(next: 'global' | 'project') {
  if (next === scope.value) return
  // The selected server belongs to the old scope's file, so drop it.
  if (next === 'global') {
    router.replace({ path: '/mcp', query: {} })
  } else {
    router.replace({
      path: '/mcp',
      query: { scope: 'project', project: projectName.value || projectOptions.value[0]?.name || '' },
    })
  }
}

function setProject(p: string) {
  router.replace({ path: '/mcp', query: { scope: 'project', project: p } })
}

const serverLink = (s: string) => ({ path: `/mcp/${encodeURIComponent(s)}`, query: query.value })

// Bare /mcp shows the first server rather than an empty detail pane.
watch(
  () => [name.value, list.data.value] as const,
  ([n, items]) => {
    if (!n && items?.length) router.replace(serverLink(items[0].name))
  },
  { immediate: true },
)

const target = (s: McpServer) =>
  s.transport.kind === 'stdio'
    ? [s.transport.command, ...s.transport.args].join(' ')
    : s.transport.url
const transportLabel = (s: McpServer) => (s.transport.kind === 'stdio' ? 'stdio' : 'http')

// The entry as it would sit in the config file; empty maps are noise.
function configOf(s: McpServer): Record<string, unknown> {
  const t = s.transport
  if (t.kind === 'stdio') {
    const out: Record<string, unknown> = { command: t.command, args: t.args }
    if (Object.keys(t.env).length) out.env = t.env
    return out
  }
  const out: Record<string, unknown> = { url: t.url }
  if (Object.keys(t.headers).length) out.headers = t.headers
  return out
}

const escapeHtml = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

function highlightJson(value: unknown): string {
  return escapeHtml(JSON.stringify(value, null, 2)).replace(
    /("(?:[^"\\]|\\.)*")(\s*:)/g,
    '<span class="mcp-json-key">$1</span>$2',
  )
}

const remove = useMcpDelete()
const confirmingDelete = ref(false)
const errorMessage = ref('')

const probeEnabled = ref(false)
const caps = useMcpCapabilities(name, scope, workingDir, () => probeEnabled.value && ready.value)
const activeTab = ref<'tools' | 'resources' | 'prompts'>('tools')

watch(name, () => {
  probeEnabled.value = false
  errorMessage.value = ''
  activeTab.value = 'tools'
})

const probeError = computed(() => {
  if (!probeEnabled.value) return undefined
  const e = caps.error.value as { message?: string } | undefined
  return e?.message
})
const needsSignIn = computed(() => /requires sign-in|HTTP 40[13]\b/.test(probeError.value ?? ''))

const now = ref(Date.now())
const ticker = setInterval(() => (now.value = Date.now()), 1000)
onBeforeUnmount(() => clearInterval(ticker))
const probedAgo = computed(() => {
  const at = caps.dataUpdatedAt.value
  if (!at) return ''
  const s = Math.max(0, Math.round((now.value - at) / 1000))
  return s < 60 ? `${s}s` : `${Math.round(s / 60)}m`
})

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync({ name: name.value, scope: scope.value, workingDir: workingDir.value })
    router.push({ path: '/mcp', query: query.value })
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function probe() {
  probeEnabled.value = true
  await caps.refetch()
}
</script>

<template>
  <div class="flex h-full flex-col">
    <PageHeader title="MCP servers" :subtitle="fileLabel">
      <template #actions>
        <select
          v-if="scope === 'project'"
          :value="projectName"
          class="ccg-input max-w-[260px]"
          aria-label="Project"
          @change="setProject(($event.target as HTMLSelectElement).value)"
        >
          <option value="" disabled>Choose a project…</option>
          <option v-for="p in projectOptions" :key="p.name" :value="p.name">
            {{ p.workingDir }}
          </option>
        </select>
        <div class="ccg-seg" role="group" aria-label="MCP scope">
          <button
            v-for="opt in [{ v: 'global', label: 'User' }, { v: 'project', label: 'Project' }] as const"
            :key="opt.v"
            type="button"
            class="px-3!"
            :aria-pressed="scope === opt.v"
            @click="setScope(opt.v)"
          >
            {{ opt.label }}
          </button>
        </div>
        <RouterLink
          :to="{ path: '/mcp/new', query }"
          class="ccg-btn-primary"
          :class="!ready ? 'pointer-events-none opacity-45' : ''"
          :aria-disabled="!ready"
        >
          + New
        </RouterLink>
      </template>
    </PageHeader>

    <EmptyState
      v-if="!ready"
      title="Choose a project — project-scoped servers live in its .mcp.json."
    />
    <QueryStateBoundary
      v-else
      :is-pending="list.isPending.value"
      :is-error="list.isError.value"
      :error="list.error.value"
      :data="list.data.value"
    >
      <template #loading>
        <div class="flex min-h-0 flex-1">
          <div class="flex w-[300px] flex-none flex-col gap-1 border-r border-hairline-soft p-3">
            <div v-for="i in 4" :key="i" class="ccg-skeleton h-[58px]" style="border-radius: 8px;" />
          </div>
        </div>
      </template>
      <template #default="{ data: items }">
        <EmptyState v-if="!items?.length" :title="`No MCP servers in ${fileLabel}.`">
          <RouterLink :to="{ path: '/mcp/new', query }" class="ccg-btn-primary">+ New</RouterLink>
        </EmptyState>
        <div v-else class="flex min-h-0 flex-1">
          <nav
            class="flex w-[300px] flex-none flex-col gap-1 overflow-auto border-r border-hairline-soft p-3"
            aria-label="MCP servers"
          >
            <RouterLink
              v-for="s in items"
              :key="s.name"
              :to="serverLink(s.name)"
              class="mcp-item flex flex-col gap-1 rounded-lg border px-3 py-2.5"
              :class="s.name === name ? 'mcp-item-active' : ''"
              :aria-current="s.name === name ? 'page' : undefined"
            >
              <div class="flex items-center gap-2">
                <span class="min-w-0 flex-1 truncate font-mono text-[13px] font-medium text-ink">{{ s.name }}</span>
                <span class="ccg-badge">{{ transportLabel(s) }}</span>
              </div>
              <div class="truncate font-mono text-[11.5px] text-neutral-500">{{ target(s) }}</div>
            </RouterLink>
          </nav>

          <section class="flex min-w-0 flex-1 flex-col gap-[18px] overflow-auto px-7 py-[22px]">
            <template v-if="name">
              <div class="flex items-center gap-2.5">
                <h3 class="min-w-0 flex-1 truncate font-mono text-[17px] font-semibold text-ink">{{ name }}</h3>
                <button
                  type="button"
                  class="ccg-btn-ghost ccg-btn-sm"
                  :disabled="caps.isFetching.value"
                  @click="probe"
                >
                  <Loader2 v-if="caps.isFetching.value" :size="14" :stroke-width="1.5" class="animate-spin" />
                  <span v-else aria-hidden="true">↻</span>
                  {{ caps.isFetching.value ? 'Probing…' : 'Probe capabilities' }}
                </button>
                <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirmingDelete = true">
                  Delete
                </button>
              </div>

              <p v-if="errorMessage" class="ccg-alert-error px-3 py-2 text-[12.5px]" role="alert">
                {{ errorMessage }}
              </p>

              <QueryStateBoundary
                :is-pending="server.isPending.value"
                :is-error="server.isError.value"
                :error="server.error.value"
                :data="server.data.value"
              >
                <template #loading>
                  <div class="ccg-skeleton h-[96px]" style="border-radius: 8px;" />
                </template>
                <template #default="{ data: srv }">
                  <div v-if="srv" class="flex flex-col gap-1.5">
                    <div class="ccg-section-label">Configuration</div>
                    <!-- eslint-disable-next-line vue/no-v-html -- escaped in highlightJson -->
                    <pre class="ccg-code-block overflow-auto" v-html="highlightJson(configOf(srv))" />
                  </div>
                </template>
              </QueryStateBoundary>

              <div
                v-if="caps.data.value?.isChannel"
                class="ccg-alert-purple flex flex-col gap-1 px-3.5 py-3 text-[13px] leading-normal"
              >
                <div class="flex gap-2.5">
                  <span class="font-mono font-semibold">claude/channel</span>
                  <span>This server can push events into sessions<template v-if="caps.data.value.relaysPermissions"> and answer permission prompts externally — tool approvals can happen outside this machine</template>.</span>
                </div>
                <p v-if="caps.data.value.instructions" class="whitespace-pre-wrap opacity-80">
                  {{ caps.data.value.instructions }}
                </p>
              </div>

              <div
                v-if="needsSignIn"
                class="ccg-alert-warn px-3.5 py-3 text-[13px]"
                role="alert"
              >
                server requires sign-in; authenticate it with <span class="font-mono">/mcp</span> in claude
              </div>
              <p v-else-if="probeError" class="ccg-alert-error px-3.5 py-3 text-[13px]" role="alert">
                {{ probeError }}
              </p>

              <div v-if="caps.data.value" class="flex flex-col gap-2">
                <div class="ccg-tabs gap-3.5! border-b border-hairline-soft" role="tablist">
                  <button
                    v-for="t in [
                      { v: 'tools', label: 'Tools', n: caps.data.value.tools.length },
                      { v: 'resources', label: 'Resources', n: caps.data.value.resources.length },
                      { v: 'prompts', label: 'Prompts', n: caps.data.value.prompts.length },
                    ] as const"
                    :key="t.v"
                    type="button"
                    role="tab"
                    class="ccg-tab pb-2!"
                    :aria-selected="activeTab === t.v"
                    @click="activeTab = t.v"
                  >
                    {{ t.label }} {{ t.n }}
                  </button>
                  <span class="flex-1" />
                  <span v-if="probedAgo" class="text-[11.5px] text-muted-soft">
                    probed {{ probedAgo }} ago · cached 60s
                  </span>
                </div>
                <template v-if="activeTab === 'tools'">
                  <div v-for="t in caps.data.value.tools" :key="t.name" class="mcp-row">
                    <span class="truncate font-mono text-[12.5px] text-ink">{{ t.name }}</span>
                    <span class="text-muted">{{ t.description }}</span>
                  </div>
                  <p v-if="!caps.data.value.tools.length" class="py-2 text-[13px] text-neutral-500">No tools.</p>
                </template>
                <template v-else-if="activeTab === 'resources'">
                  <div v-for="r in caps.data.value.resources" :key="r.uri" class="mcp-row">
                    <span class="truncate font-mono text-[12.5px] text-ink">{{ r.name ?? r.uri }}</span>
                    <span class="truncate font-mono text-[12px] text-muted">{{ r.uri }}</span>
                  </div>
                  <p v-if="!caps.data.value.resources.length" class="py-2 text-[13px] text-neutral-500">No resources.</p>
                </template>
                <template v-else>
                  <div v-for="p in caps.data.value.prompts" :key="p.name" class="mcp-row">
                    <span class="truncate font-mono text-[12.5px] text-ink">{{ p.name }}</span>
                    <span class="text-muted">{{ p.description }}</span>
                  </div>
                  <p v-if="!caps.data.value.prompts.length" class="py-2 text-[13px] text-neutral-500">No prompts.</p>
                </template>
              </div>
              <div v-else-if="caps.isFetching.value" class="flex flex-col gap-2">
                <div v-for="i in 4" :key="i" class="ccg-skeleton h-8" style="border-radius: 6px;" />
              </div>
              <p v-else-if="!probeError" class="text-[13px] text-neutral-500">
                Probe to connect (stdio servers are spawned) and list its tools, resources and prompts.
              </p>
            </template>
          </section>
        </div>
      </template>
    </QueryStateBoundary>

    <ConfirmDialog
      v-model:open="confirmingDelete"
      title="Delete MCP server?"
      :message="`This removes '${name}' from ${fileLabel}.`"
      confirm-label="Delete"
      danger
      @confirm="onDelete"
    />
  </div>
</template>

<style scoped>
.mcp-item {
  border-color: transparent;
  transition: background-color 120ms ease-out, border-color 120ms ease-out;
}
.mcp-item:hover {
  background: var(--ccg-surface-strong);
}
.mcp-item-active,
.mcp-item-active:hover {
  background: var(--ccg-surface-card);
  border-color: var(--ccg-hairline-strong);
}
.mcp-row {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: 12px;
  padding: 8px 2px;
  border-bottom: 1px solid var(--ccg-hairline-faint);
  font-size: 13px;
}
.ccg-code-block :deep(.mcp-json-key) {
  color: var(--ccg-accent);
}
</style>
