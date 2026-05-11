<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { EditorState, Compartment } from '@codemirror/state'
import { EditorView, keymap, lineNumbers } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { yaml } from '@codemirror/lang-yaml'

const props = withDefaults(
  defineProps<{
    modelValue: string
    language?: 'markdown' | 'yaml'
    minHeight?: string
    placeholder?: string
    fill?: boolean
  }>(),
  { language: 'markdown', minHeight: '240px', fill: false },
)
const emit = defineEmits<{ 'update:modelValue': [string] }>()

const host = ref<HTMLDivElement>()
let view: EditorView | undefined
const langCompartment = new Compartment()

function makeLang() {
  return props.language === 'yaml' ? yaml() : markdown()
}

onMounted(() => {
  const startState = EditorState.create({
    doc: props.modelValue ?? '',
    extensions: [
      lineNumbers(),
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      langCompartment.of(makeLang()),
      EditorView.lineWrapping,
      EditorView.updateListener.of((u) => {
        if (u.docChanged) {
          emit('update:modelValue', u.state.doc.toString())
        }
      }),
      EditorView.theme({
        '&': props.fill
          ? { fontSize: '13px', height: '100%' }
          : { fontSize: '13px', minHeight: props.minHeight },
        '.cm-scroller': {
          fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
          overflow: 'auto',
        },
        '.cm-content': { padding: '8px 0' },
      }),
    ],
  })
  view = new EditorView({ state: startState, parent: host.value! })
})

onBeforeUnmount(() => view?.destroy())

watch(
  () => props.modelValue,
  (next) => {
    if (!view) return
    const current = view.state.doc.toString()
    if (next === current) return
    view.dispatch({
      changes: { from: 0, to: current.length, insert: next ?? '' },
    })
  },
)

watch(
  () => props.language,
  () => {
    if (!view) return
    view.dispatch({ effects: langCompartment.reconfigure(makeLang()) })
  },
)
</script>

<template>
  <div
    ref="host"
    class="overflow-hidden rounded-md border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900"
  />
</template>
