<script lang="ts">
  import type { ConflictStrategy } from '../lib/ipc'

  let {
    names,
    onchoose,
    oncancel,
  }: {
    names: string[]
    onchoose: (strategy: ConflictStrategy, applyAll: boolean) => void
    oncancel: () => void
  } = $props()

  let applyAll = $state(true)
</script>

<div class="scrim" role="presentation" onclick={oncancel}>
  <div
    class="sheet"
    role="dialog"
    tabindex="-1"
    aria-labelledby="conflict-title"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => event.stopPropagation()}
  >
    <h2 id="conflict-title">Items already exist at the destination</h2>
    <ul>
      {#each names as name (name)}
        <li>{name}</li>
      {/each}
    </ul>
    <label>
      <input type="checkbox" bind:checked={applyAll} />
      Apply to all
    </label>
    <div class="actions">
      <button type="button" onclick={oncancel}>Cancel</button>
      <button type="button" onclick={() => onchoose('skip', applyAll)}
        >Skip</button
      >
      <button type="button" onclick={() => onchoose('keepBoth', applyAll)}
        >Keep Both</button
      >
      <button type="button" onclick={() => onchoose('replace', applyAll)}
        >Replace</button
      >
    </div>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: color-mix(in srgb, var(--fg) 20%, transparent);
  }

  .sheet {
    width: min(28rem, 100%);
    padding: var(--space-4);
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: 1rem;
  }

  ul {
    margin: 0 0 var(--space-3);
    padding-inline-start: var(--space-4);
    color: var(--fg);
  }

  label {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .actions {
    display: flex;
    justify-content: end;
    gap: var(--space-2);
  }

  button {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
  }
</style>
