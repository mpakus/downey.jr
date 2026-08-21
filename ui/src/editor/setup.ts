import { markdown } from '@codemirror/lang-markdown'
import { indentUnit, syntaxHighlighting } from '@codemirror/language'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { Compartment, EditorState } from '@codemirror/state'
import {
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from '@codemirror/view'

import { markdownHighlightStyle } from './highlight'
import type { MarkdownEditor, MarkdownEditorOptions } from './types'

export type { MarkdownEditor, MarkdownEditorOptions }

const editorTheme = EditorView.theme({
  '&': {
    height: '100%',
    backgroundColor: 'var(--bg)',
    color: 'var(--fg)',
    fontFamily: 'var(--font-body)',
    fontSize: 'var(--font-size)',
    lineHeight: 'var(--line-height)',
  },
  '&.cm-focused': {
    outline: 'none',
  },
  '.cm-scroller': {
    fontFamily: 'var(--font-body)',
    lineHeight: 'inherit',
  },
  '.cm-content': {
    caretColor: 'var(--ed-cursor)',
    padding: 'var(--space-6) var(--space-4)',
  },
  '.cm-line': {
    padding: '0',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--ed-cursor)',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
    {
      backgroundColor: 'var(--ed-sel)',
    },
  '.cm-activeLine': {
    backgroundColor: 'var(--ed-active-line)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--bg)',
    color: 'var(--ed-syntax)',
    border: 'none',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--ed-active-line)',
    color: 'var(--fg-muted)',
  },
})

function clamp(value: number, max: number): number {
  return Math.max(0, Math.min(max, value))
}

function indentExtension(spaces: number) {
  const count = Math.max(1, Math.min(8, Math.round(spaces) || 2))
  return [EditorState.tabSize.of(count), indentUnit.of(' '.repeat(count))]
}

function lineNumberExtension(on: boolean) {
  return on ? [lineNumbers(), highlightActiveLineGutter()] : []
}

function wrapExtension(on: boolean) {
  return on ? EditorView.lineWrapping : []
}

function spellcheckAttributes(on: boolean) {
  return EditorView.contentAttributes.of({
    'aria-label': 'Markdown source',
    spellcheck: on ? 'true' : 'false',
  })
}

/**
 * Creates a CodeMirror Markdown editor. Call only after a dynamic import so
 * Preview-only sessions never load the editor chunk.
 */
export function createMarkdownEditor(
  parent: HTMLElement,
  options: MarkdownEditorOptions,
): MarkdownEditor {
  const writable = new Compartment()
  const numbers = new Compartment()
  const wrap = new Compartment()
  const indent = new Compartment()
  const attrs = new Compartment()

  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: options.doc,
      extensions: [
        history(),
        drawSelection(),
        dropCursor(),
        highlightActiveLine(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown({ addKeymap: true }),
        syntaxHighlighting(markdownHighlightStyle, { fallback: false }),
        editorTheme,
        writable.of(EditorState.readOnly.of(!options.writable)),
        numbers.of(lineNumberExtension(options.lineNumbers)),
        wrap.of(wrapExtension(options.softWrap)),
        indent.of(indentExtension(options.indentUnit)),
        attrs.of(spellcheckAttributes(options.spellcheck)),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            options.onChange(update.state.doc.toString())
          }
        }),
      ],
    }),
  })

  function setDoc(text: string) {
    if (view.state.doc.toString() === text) {
      return
    }
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: text },
    })
  }

  return {
    setDoc,
    setWritable(on) {
      view.dispatch({
        effects: writable.reconfigure(EditorState.readOnly.of(!on)),
      })
    },
    setSpellcheck(on) {
      view.dispatch({
        effects: attrs.reconfigure(spellcheckAttributes(on)),
      })
    },
    setLineNumbers(on) {
      view.dispatch({
        effects: numbers.reconfigure(lineNumberExtension(on)),
      })
    },
    setSoftWrap(on) {
      view.dispatch({
        effects: wrap.reconfigure(wrapExtension(on)),
      })
    },
    setIndentUnit(spaces) {
      view.dispatch({
        effects: indent.reconfigure(indentExtension(spaces)),
      })
    },
    selection() {
      const range = view.state.selection.main
      return { start: range.from, end: range.to }
    },
    setTextAndSelection(text, start, end) {
      const length = text.length
      const from = clamp(start, length)
      const to = clamp(end, length)
      const current = view.state.doc.toString()
      view.dispatch({
        ...(current === text
          ? {}
          : { changes: { from: 0, to: view.state.doc.length, insert: text } }),
        selection: { anchor: from, head: to },
        scrollIntoView: true,
      })
    },
    focus() {
      view.focus()
    },
    refresh() {
      view.requestMeasure()
    },
    destroy() {
      view.destroy()
    },
  }
}
