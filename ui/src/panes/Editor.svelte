<script lang="ts">
  let {
    value = $bindable(''),
    writable = true,
    spellcheck = true,
    textareaEl = $bindable(),
  }: {
    value: string
    writable?: boolean
    spellcheck?: boolean
    textareaEl?: HTMLTextAreaElement | undefined
  } = $props()
</script>

<label class="editor">
  <span class="sr">Markdown source</span>
  <textarea
    bind:this={textareaEl}
    bind:value
    readonly={!writable}
    {spellcheck}
    aria-label="Markdown source"></textarea>
</label>

<style>
  .editor {
    display: flex;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }

  textarea {
    flex: 1;
    min-width: 0;
    min-height: 0;
    width: 100%;
    resize: none;
    border: 0;
    padding: var(--space-6) var(--space-4);
    background: var(--bg);
    color: var(--fg);
    font-family: var(--font-body);
    font-size: var(--font-size);
    line-height: var(--line-height);
    max-width: none;
    tab-size: 2;
  }

  textarea:focus-visible {
    outline: none;
  }
</style>
