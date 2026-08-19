<script lang="ts">
  import { tabTitle, type DocTab } from '../lib/tabs'

  let {
    tabs,
    activeRelPath = null,
    onselect,
    onclose,
  }: {
    tabs: DocTab[]
    activeRelPath?: string | null
    onselect: (relPath: string) => void
    onclose: (relPath: string) => void
  } = $props()
</script>

{#if tabs.length > 0}
  <div class="tabs" role="tablist" aria-label="Open documents">
    {#each tabs as tab (tab.relPath)}
      {@const selected = tab.relPath === activeRelPath}
      <div class="tab" class:selected>
        <button
          type="button"
          role="tab"
          aria-selected={selected}
          title={tab.relPath}
          onclick={() => onselect(tab.relPath)}>{tab.title || tabTitle(tab.relPath)}</button
        >
        <button
          type="button"
          class="close"
          title="Close"
          aria-label="Close {tab.title}"
          onclick={() => onclose(tab.relPath)}>×</button
        >
      </div>
    {/each}
  </div>
{/if}

<style>
  .tabs {
    display: flex;
    flex: none;
    min-width: 0;
    overflow: auto;
    border-block-end: 1px solid var(--border);
    background: color-mix(in srgb, var(--sidebar) 40%, var(--bg));
  }

  .tab {
    display: flex;
    align-items: stretch;
    flex: none;
    min-width: 0;
    max-width: 16rem;
    border-inline-end: 1px solid var(--border);
  }

  .tab.selected {
    background: var(--bg);
  }

  .tab > button[role='tab'] {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: var(--space-2) var(--space-3);
    color: var(--fg-muted);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .tab.selected > button[role='tab'] {
    color: var(--fg);
  }

  .close {
    width: 28px;
    flex: none;
    color: var(--fg-muted);
    font-size: 1rem;
    line-height: 1;
  }

  .close:hover,
  .tab > button[role='tab']:hover {
    color: var(--fg);
    background: var(--selection);
  }
</style>
