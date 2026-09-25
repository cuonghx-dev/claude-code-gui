<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { EditorState, Compartment } from '@codemirror/state'
import { EditorView, highlightActiveLine, highlightActiveLineGutter, keymap, lineNumbers } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { markdown } from '@codemirror/lang-markdown'
import { yaml, yamlFrontmatter } from '@codemirror/lang-yaml'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'

const props = withDefaults(
  defineProps<{
    modelValue: string
    language?: 'markdown' | 'yaml' | 'javascript' | 'json'
    minHeight?: string
    placeholder?: string
    fill?: boolean
    /** Full-bleed inside an editor page: no border or radius. */
    flush?: boolean
  }>(),
  { language: 'markdown', minHeight: '240px', fill: false, flush: false },
)
const emit = defineEmits<{
  'update:modelValue': [string]
  cursor: [{ line: number; col: number }]
}>()

const host = ref<HTMLDivElement>()
let view: EditorView | undefined
const langCompartment = new Compartment()

function makeLang() {
  if (props.language === 'yaml') return yaml()
  if (props.language === 'javascript') return javascript()
  if (props.language === 'json') return json()
  // Agent / command / skill files open with a YAML frontmatter block.
  return yamlFrontmatter({ content: markdown() })
}

/** Syntax colors from the design handoff. */
const highlight = HighlightStyle.define([
  { tag: [t.propertyName, t.definition(t.propertyName)], color: '#B4502B' },
  { tag: [t.string, t.special(t.string)], color: '#3F7A4E' },
  { tag: [t.meta, t.processingInstruction, t.contentSeparator], color: '#A29E94' },
  { tag: t.heading, color: '#2F5E9E', fontWeight: '500' },
  { tag: t.monospace, color: '#7A4FA0' },
  { tag: t.strong, fontWeight: '600' },
  { tag: t.emphasis, fontStyle: 'italic' },
  { tag: [t.link, t.url], color: '#2F5E9E' },
  { tag: [t.number, t.bool, t.null, t.atom], color: '#7A4FA0' },
  { tag: [t.keyword, t.operatorKeyword, t.controlKeyword], color: '#B4502B' },
  { tag: t.comment, color: '#A29E94', fontStyle: 'italic' },
  { tag: [t.punctuation, t.bracket, t.separator], color: '#8A867C' },
  { tag: [t.variableName, t.function(t.variableName)], color: '#1F1E1B' },
  { tag: t.typeName, color: '#2F5E9E' },
])

function emitCursor(state: EditorState) {
  const head = state.selection.main.head
  const line = state.doc.lineAt(head)
  emit('cursor', { line: line.number, col: head - line.from + 1 })
}

onMounted(() => {
  const startState = EditorState.create({
    doc: props.modelValue ?? '',
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      langCompartment.of(makeLang()),
      syntaxHighlighting(highlight),
      EditorView.lineWrapping,
      EditorView.updateListener.of((u) => {
        if (u.docChanged) {
          emit('update:modelValue', u.state.doc.toString())
        }
        if (u.docChanged || u.selectionSet) emitCursor(u.state)
      }),
      EditorView.theme({
        '&': props.fill
          ? { fontSize: '13px', height: '100%', backgroundColor: '#FFFFFF', color: '#1F1E1B' }
          : { fontSize: '13px', minHeight: props.minHeight, backgroundColor: '#FFFFFF', color: '#1F1E1B' },
        '&.cm-focused': { outline: 'none' },
        '.cm-scroller': {
          fontFamily: "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace",
          lineHeight: '22px',
          overflow: 'auto',
        },
        '.cm-content': { padding: '14px 0', caretColor: '#1F1E1B' },
        '.cm-line': { padding: '0 18px' },
        '.cm-cursor, .cm-dropCursor': { borderLeftColor: '#1F1E1B', borderLeftWidth: '2px' },
        '.cm-gutters': {
          backgroundColor: '#FAF9F6',
          color: '#C2BDB2',
          borderRight: '1px solid #F0EDE7',
        },
        '.cm-lineNumbers .cm-gutterElement': {
          minWidth: '36px',
          padding: '0 12px 0 0',
          textAlign: 'right',
        },
        '.cm-gutters .cm-gutter:first-child': { paddingTop: '14px' },
        '.cm-activeLine': { backgroundColor: '#FFF6E8' },
        '.cm-activeLineGutter': { backgroundColor: '#FAF9F6', color: '#8A867C' },
        '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': {
          backgroundColor: '#EDE5D6',
        },
      }),
    ],
  })
  view = new EditorView({ state: startState, parent: host.value! })
  emitCursor(startState)
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
    class="overflow-hidden bg-white"
    :class="flush ? '' : 'rounded-[8px] border border-[#E6E2DA]'"
  />
</template>
