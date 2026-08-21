<script lang="ts">
  import type { MarkdownEditor } from '../editor/types'

  let {
    value = $bindable(''),
    api = $bindable(null),
    writable = true,
    spellcheck = true,
    lineNumbers = false,
    softWrap = true,
    indentUnit = 2,
    hidden = false,
  }: {
    value: string
    api?: MarkdownEditor | null
    writable?: boolean
    spellcheck?: boolean
    lineNumbers?: boolean
    softWrap?: boolean
    indentUnit?: number
    hidden?: boolean
  } = $props()

  let host = $state<HTMLDivElement | undefined>()

  $effect(() => {
    const el = host
    if (!el) {
      return
    }
    let cancelled = false
    let instance: MarkdownEditor | undefined

    void import('../editor/setup').then(({ createMarkdownEditor }) => {
      if (cancelled) {
        return
      }
      instance = createMarkdownEditor(el, {
        doc: value,
        writable,
        spellcheck,
        lineNumbers,
        softWrap,
        indentUnit,
        onChange(text) {
          value = text
        },
      })
      api = instance
    })

    return () => {
      cancelled = true
      instance?.destroy()
      api = null
    }
  })

  $effect(() => {
    api?.setDoc(value)
  })
  $effect(() => {
    api?.setWritable(writable)
  })
  $effect(() => {
    api?.setSpellcheck(spellcheck)
  })
  $effect(() => {
    api?.setLineNumbers(lineNumbers)
  })
  $effect(() => {
    api?.setSoftWrap(softWrap)
  })
  $effect(() => {
    api?.setIndentUnit(indentUnit)
  })
  $effect(() => {
    if (!hidden) {
      api?.refresh()
    }
  })
</script>

<div class="editor" class:is-hidden={hidden}>
  <div bind:this={host} class="cm-host"></div>
</div>

<style>
  .editor {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .editor.is-hidden {
    display: none;
  }

  .cm-host {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  .cm-host :global(.cm-editor) {
    flex: 1;
    min-width: 0;
    height: 100%;
  }
</style>
