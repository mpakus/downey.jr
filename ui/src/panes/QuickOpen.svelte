<script lang="ts">
  import { errorMessage, filesSearch, type TreeNode } from '../lib/ipc'

  let {
    projectId,
    onopen,
    onclose,
    onerror,
  }: {
    projectId: string
    onopen: (relPath: string) => void
    onclose: () => void
    onerror: (message: string) => void
  } = $props()

  let query = $state('')
  let items = $state<TreeNode[]>([])
  let index = $state(0)
  let inputEl = $state<HTMLInputElement | undefined>()

  $effect(() => {
    inputEl?.focus()
  })

  $effect(() => {
    const needle = query
    const id = projectId
    void (async () => {
      try {
        items = await filesSearch(id, needle, 40)
        index = 0
      } catch (cause) {
        onerror(errorMessage(cause))
      }
    })()
  })

  function confirm() {
    const item = items[index]
    if (item) {
      onopen(item.relPath)
    }
    onclose()
  }
</script>

<div
  class="scrim"
  role="presentation"
  onclick={onclose}
  onkeydown={(event) => {
    if (event.key === 'Escape') {
      onclose()
    }
  }}
>
  <div
    class="palette"
    role="dialog"
    tabindex="-1"
    aria-label="Open file"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => event.stopPropagation()}
  >
    <input
      bind:this={inputEl}
      bind:value={query}
      placeholder="Open file"
      aria-label="Search files"
      onkeydown={(event) => {
        if (event.key === 'Escape') {
          onclose()
        }
        if (event.key === 'Enter') {
          event.preventDefault()
          confirm()
        }
        if (event.key === 'ArrowDown') {
          event.preventDefault()
          index = Math.min(items.length - 1, index + 1)
        }
        if (event.key === 'ArrowUp') {
          event.preventDefault()
          index = Math.max(0, index - 1)
        }
      }}
    />
    <ul>
      {#each items as item, itemIndex (item.relPath)}
        <li>
          <button
            type="button"
            class:active={itemIndex === index}
            onclick={() => {
              onopen(item.relPath)
              onclose()
            }}>{item.relPath}</button
          >
        </li>
      {/each}
    </ul>
    {#if items.length === 0}
      <p>No Markdown files match.</p>
    {/if}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: start center;
    padding: var(--space-6);
    background: color-mix(in srgb, var(--fg) 20%, transparent);
  }

  .palette {
    width: min(36rem, 100%);
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  input {
    width: 100%;
    padding: var(--space-3);
    border: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--fg);
    font: inherit;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: var(--space-1);
    max-height: 16rem;
    overflow: auto;
  }

  button {
    width: 100%;
    padding: var(--space-2);
    text-align: start;
    border-radius: var(--radius-sm);
    color: var(--fg);
  }

  button.active,
  button:hover {
    background: var(--selection);
  }

  p {
    margin: 0;
    padding: var(--space-3);
    color: var(--fg-muted);
  }
</style>
