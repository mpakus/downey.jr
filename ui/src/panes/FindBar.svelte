<script lang="ts">
  import { findMatchOffsets } from '../lib/text'

  let {
    root = null,
    onclose,
  }: {
    root?: HTMLElement | null
    onclose: () => void
  } = $props()

  let query = $state('')
  let index = $state(0)
  let matches = $state<Range[]>([])
  let inputEl = $state<HTMLInputElement | undefined>()

  $effect(() => {
    inputEl?.focus()
  })

  $effect(() => {
    const needle = query
    const host = root
    if (!host || !needle) {
      matches = []
      index = 0
      return
    }
    matches = collectRanges(host, needle)
    index = 0
    reveal(0)
  })

  function collectRanges(host: HTMLElement, needle: string): Range[] {
    const found: Range[] = []
    const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT)
    let node = walker.nextNode()
    while (node) {
      const text = node.textContent ?? ''
      for (const at of findMatchOffsets(text, needle)) {
        const range = document.createRange()
        range.setStart(node, at)
        range.setEnd(node, at + needle.length)
        found.push(range)
      }
      node = walker.nextNode()
    }
    return found
  }

  function reveal(next: number) {
    const range = matches[next]
    if (!range) {
      return
    }
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)
    const element =
      range.startContainer instanceof Element
        ? range.startContainer
        : range.startContainer.parentElement
    element?.scrollIntoView({ block: 'center' })
  }

  function step(delta: number) {
    if (matches.length === 0) {
      return
    }
    index = (index + delta + matches.length) % matches.length
    reveal(index)
  }
</script>

<div class="find" role="search">
  <input
    bind:this={inputEl}
    bind:value={query}
    placeholder="Find in document"
    aria-label="Find in document"
    onkeydown={(event) => {
      if (event.key === 'Escape') {
        onclose()
      }
      if (event.key === 'Enter' && event.shiftKey) {
        event.preventDefault()
        step(-1)
      } else if (event.key === 'Enter') {
        event.preventDefault()
        step(1)
      }
    }}
  />
  <span>{matches.length === 0 ? '0' : `${index + 1} of ${matches.length}`}</span
  >
  <button type="button" onclick={() => step(-1)}>Previous</button>
  <button type="button" onclick={() => step(1)}>Next</button>
  <button type="button" onclick={onclose}>Close</button>
</div>

<style>
  .find {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
  }

  input {
    flex: 1;
    min-width: 0;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }

  span {
    color: var(--fg-muted);
    font-size: 0.8125rem;
    white-space: nowrap;
  }

  button {
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
  }
</style>
