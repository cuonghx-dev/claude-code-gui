<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/components/PageHeader.vue'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useMcpDelete, useMcpServer } from '@/composables/useMcp'

const route = useRoute()
const router = useRouter()
const name = computed(() => (route.params as { name: string }).name)
const { isPending, isError, error, data } = useMcpServer(name, 'global')

const remove = useMcpDelete()
const confirmingDelete = ref(false)
const errorMessage = ref('')

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync({ name: name.value, scope: 'global' })
    router.push('/mcp')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <PageHeader :title="name" subtitle="Global ~/.claude/.mcp.json (Phase 5 adds capability probe)">
    <template #actions>
      <button type="button" class="ccg-btn-danger" @click="confirmingDelete = true">Delete</button>
    </template>
  </PageHeader>
  <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
    <template #default="{ data: srv }">
      <section v-if="srv" class="p-6">
        <p
          v-if="errorMessage"
          class="mb-4 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200"
        >
          {{ errorMessage }}
        </p>
        <pre class="overflow-auto rounded-lg border border-neutral-200 bg-white p-4 text-sm dark:border-neutral-800 dark:bg-neutral-900">{{ JSON.stringify(srv, null, 2) }}</pre>
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
