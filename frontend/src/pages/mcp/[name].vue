<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useMcpCapabilities, useMcpDelete, useMcpServer } from '@/composables/useMcp'

const route = useRoute()
const router = useRouter()
const name = computed(() => (route.params as { name: string }).name)
const server = useMcpServer(name, 'global')

const remove = useMcpDelete()
const confirmingDelete = ref(false)
const errorMessage = ref('')

const probeEnabled = ref(false)
const caps = useMcpCapabilities(name, 'global', undefined, probeEnabled)

const probeError = computed(() => {
  const e = caps.error.value as { message?: string } | undefined
  return e?.message
})

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync({ name: name.value, scope: 'global' })
    router.push('/mcp')
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
  <PageHeader :title="name" subtitle="MCP server detail">
    <template #actions>
      <button
        type="button"
        class="ccg-btn-ghost"
        :disabled="caps.isFetching.value"
        @click="probe"
      >
        {{ caps.isFetching.value ? 'Probing…' : 'Probe capabilities' }}
      </button>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <p
    v-if="errorMessage"
    class="mx-6 mt-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
  >
    {{ errorMessage }}
  </p>
  <QueryStateBoundary
    :is-pending="server.isPending.value"
    :is-error="server.isError.value"
    :error="server.error.value"
    :data="server.data.value"
  >
    <template #default="{ data: srv }">
      <section v-if="srv" class="grid grid-cols-1 gap-6 p-6 lg:grid-cols-2">
        <div>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Configuration</h3>
          <pre class="mt-2 overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-xs dark:border-neutral-800 dark:bg-neutral-900">{{ JSON.stringify(srv, null, 2) }}</pre>
        </div>

        <div>
          <h3 class="text-xs font-semibold uppercase tracking-wide text-neutral-500">Capabilities</h3>
          <p
            v-if="!probeEnabled"
            class="mt-2 text-xs text-neutral-500 dark:text-neutral-400"
          >
            Click "Probe capabilities" to spawn the server and run the MCP handshake.
          </p>
          <p
            v-else-if="probeError"
            class="mt-2 rounded-md border border-red-300 bg-red-50 p-2 text-xs text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
          >
            {{ probeError }}
          </p>
          <div v-else-if="caps.data.value" class="mt-2 space-y-3 text-sm">
            <div>
              <p class="text-xs font-semibold text-neutral-700 dark:text-neutral-300">
                Tools ({{ caps.data.value.tools.length }})
              </p>
              <ul class="mt-1 space-y-1 text-xs">
                <li v-for="t in caps.data.value.tools" :key="t.name" class="font-mono">
                  <span class="font-semibold">{{ t.name }}</span>
                  <span v-if="t.description" class="text-neutral-500"> — {{ t.description }}</span>
                </li>
              </ul>
            </div>
            <div>
              <p class="text-xs font-semibold text-neutral-700 dark:text-neutral-300">
                Resources ({{ caps.data.value.resources.length }})
              </p>
              <ul class="mt-1 space-y-1 text-xs">
                <li v-for="r in caps.data.value.resources" :key="r.uri" class="font-mono">
                  <span class="font-semibold">{{ r.name ?? r.uri }}</span>
                  <span class="text-neutral-500">  · {{ r.uri }}</span>
                </li>
              </ul>
            </div>
            <div>
              <p class="text-xs font-semibold text-neutral-700 dark:text-neutral-300">
                Prompts ({{ caps.data.value.prompts.length }})
              </p>
              <ul class="mt-1 space-y-1 text-xs">
                <li v-for="p in caps.data.value.prompts" :key="p.name" class="font-mono">
                  <span class="font-semibold">{{ p.name }}</span>
                  <span v-if="p.description" class="text-neutral-500"> — {{ p.description }}</span>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>
    </template>
  </QueryStateBoundary>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Delete MCP server?"
    :message="`This removes '${name}' from .mcp.json.`"
    confirm-label="Delete"
    danger
    @confirm="onDelete"
  />
</template>
