<script lang="ts">
  let {
    disabled = false,
    oncommand,
  }: {
    disabled?: boolean
    oncommand: (id: string) => void
  } = $props()

  const textCommands = [
    { id: 'edit-bold', label: 'B', name: 'Bold' },
    { id: 'edit-italic', label: 'I', name: 'Italic' },
    { id: 'edit-inline-code', label: '`', name: 'Inline code' },
    { id: 'edit-heading-1', label: 'H1', name: 'Heading 1' },
    { id: 'edit-heading-2', label: 'H2', name: 'Heading 2' },
    { id: 'edit-list', label: 'List', name: 'List' },
    { id: 'edit-task', label: 'Task', name: 'Task list' },
    { id: 'edit-quote', label: 'Quote', name: 'Quote' },
  ]
</script>

<div class="toolbar" role="toolbar" aria-label="Editor">
  <div class="group" role="group" aria-label="Text">
    <span class="legend">Text</span>
    {#each textCommands as command (command.id)}
      <button
        type="button"
        {disabled}
        aria-label={command.name}
        title={command.name}
        onclick={() => oncommand(command.id)}>{command.label}</button
      >
    {/each}
  </div>
  <div class="group" role="group" aria-label="Links">
    <span class="legend">Links</span>
    <button
      type="button"
      {disabled}
      aria-label="Link"
      title="Link"
      onclick={() => oncommand('edit-link')}>Link</button
    >
    <button
      type="button"
      {disabled}
      aria-label="Wiki link"
      title="Wiki link"
      onclick={() => oncommand('edit-wiki-link')}>Wiki</button
    >
  </div>
  <div class="group" role="group" aria-label="Media">
    <span class="legend">Media</span>
    <button
      type="button"
      {disabled}
      aria-label="Image"
      title="Image"
      onclick={() => oncommand('edit-image')}>Image</button
    >
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    height: 32px;
    padding-inline: 0;
    overflow: auto;
    -webkit-app-region: no-drag;
  }

  .group {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex: none;
  }

  .legend {
    padding-inline-end: var(--space-1);
    color: var(--fg-muted);
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  button {
    padding: 0 var(--space-2);
    height: 24px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-elev);
    color: var(--fg);
    font-size: 0.75rem;
    line-height: 1;
    transition-property: background-color, transform;
    transition-duration: var(--duration);
  }

  button:hover:not(:disabled) {
    background: var(--selection);
  }

  button:active:not(:disabled) {
    transform: scale(0.96);
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  @media (prefers-reduced-motion: reduce) {
    button:active:not(:disabled) {
      transform: none;
    }
  }
</style>
