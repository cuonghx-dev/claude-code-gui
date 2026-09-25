<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import QueryStateBoundary from '@/components/QueryStateBoundary.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  usePlugin,
  usePluginDelete,
  usePluginSetEnabled,
} from '@/composables/usePlugins'
import { describe } from '@/utils/description'

const route = useRoute()
const router = useRouter()
const id = computed(() => (route.params as { id: string }).id)
const { isPending, isError, error, data } = usePlugin(id)

const setEnabled = usePluginSetEnabled()
const remove = usePluginDelete()

const errorMessage = ref('')
const confirmingDelete = ref(false)

async function toggle(next: boolean) {
  errorMessage.value = ''
  try {
    await setEnabled.mutateAsync({ id: id.value, enabled: next })
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}

async function onDelete() {
  errorMessage.value = ''
  try {
    await remove.mutateAsync(id.value)
    router.push('/plugins')
  } catch (e) {
    errorMessage.value = (e as { message?: string })?.message ?? String(e)
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <QueryStateBoundary :is-pending="isPending" :is-error="isError" :error="error" :data="data">
      <template #default="{ data: detail }">
        <div v-if="detail" class="flex max-w-[960px] flex-col gap-[18px] px-7 py-[22px]">
          <div class="flex flex-col gap-1">
            <nav class="text-[13px]" style="color: var(--ccg-subtle);">
              <RouterLink to="/plugins" class="hover:text-ink">Plugins</RouterLink>
              <span v-if="detail.marketplace"> / <span class="font-mono text-[12px]">{{ detail.marketplace }}</span></span>
              /
            </nav>
            <div class="flex items-center gap-2.5">
              <h2 class="min-w-0 truncate font-mono text-[17px] font-semibold text-ink">{{ detail.name }}</h2>
              <span v-if="detail.version" class="ccg-badge">v{{ detail.version }}</span>
              <span class="flex-1" />
              <label class="flex items-center gap-2 text-[12.5px]" style="color: var(--ccg-muted);">
                {{ detail.enabled ? 'Enabled' : 'Disabled' }}
                <button
                  type="button"
                  role="switch"
                  class="ccg-switch disabled:opacity-50"
                  :aria-checked="detail.enabled"
                  :aria-label="detail.enabled ? 'Disable plugin' : 'Enable plugin'"
                  :disabled="setEnabled.isPending.value"
                  @click="toggle(!detail.enabled)"
                />
              </label>
              <button type="button" class="ccg-btn-danger ccg-btn-sm" @click="confirmingDelete = true">
                Uninstall
              </button>
            </div>
          </div>

          <p v-if="errorMessage" class="ccg-alert-error px-3 py-2 text-[12.5px]" role="alert">
            {{ errorMessage }}
          </p>

          <p class="text-[13px] leading-[1.5]" style="color: var(--ccg-body); text-wrap: pretty;">
            {{ describe(detail.description, detail.readme) }}
          </p>

          <div class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Location</span>
            <div class="ccg-code-block break-all" style="white-space: pre-wrap;">{{ detail.dir }}</div>
          </div>

          <div class="flex flex-col gap-1.5">
            <span class="ccg-section-label">Skills {{ detail.skills.length }}</span>
            <div v-if="detail.skills.length" class="flex flex-wrap gap-1">
              <span v-for="sk in detail.skills" :key="sk" class="ccg-chip">{{ sk }}</span>
            </div>
            <p v-else class="text-[13px]" style="color: var(--ccg-subtle);">This plugin contributes no skills.</p>
          </div>

          <div v-if="detail.readme" class="flex flex-col gap-1.5">
            <span class="ccg-section-label">README</span>
            <pre class="ccg-code-block max-h-[60vh]" style="white-space: pre-wrap;">{{ detail.readme }}</pre>
          </div>
        </div>
      </template>
    </QueryStateBoundary>
  </div>
  <ConfirmDialog
    v-model:open="confirmingDelete"
    title="Uninstall plugin?"
    :message="`This permanently removes the plugin directory '${id}/' from disk.`"
    confirm-label="Uninstall"
    danger
    @confirm="onDelete"
  />
</template>
