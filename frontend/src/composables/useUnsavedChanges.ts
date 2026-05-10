import { onBeforeRouteLeave } from 'vue-router'
import { onBeforeUnmount, type Ref, watch } from 'vue'

/**
 * Wire a beforeRouteLeave guard + window beforeunload handler that prompts
 * the user when `dirty.value` is true. The prompt is plain `confirm(...)`
 * for now; replace with a Radix-Vue dialog once the design system lands.
 */
export function useUnsavedChanges(dirty: Ref<boolean>, message = 'Discard unsaved changes?') {
  onBeforeRouteLeave(() => {
    if (!dirty.value) return true
    return window.confirm(message)
  })

  const handler = (e: BeforeUnloadEvent) => {
    if (!dirty.value) return
    e.preventDefault()
    // Spec-required for browsers that still gate on returnValue.
    e.returnValue = ''
  }
  watch(
    dirty,
    (d) => {
      if (d) window.addEventListener('beforeunload', handler)
      else window.removeEventListener('beforeunload', handler)
    },
    { immediate: true },
  )

  onBeforeUnmount(() => window.removeEventListener('beforeunload', handler))
}
